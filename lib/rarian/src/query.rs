use std::str::Chars;
use std::collections::HashSet;
use std::ops::Bound;
use std::convert::TryInto;

pub use lmdb::Transaction;

use nom::{
    IResult,
    sequence::delimited,
    character::complete::char,
};

use crate::error::*;
use crate::db::meta::Metakey;

use crate::db::{
    Database,
    Index,
    EntryDB,
    RangeDB,
    entry::EntryT,
};

use crate::uuid::UUID;

use crate::db::dbm::{
    DBManager,
    RoTransaction,
};

// Query: Takes specific index, gives a Set of valid UUIDs
//   Range
//   Text match
//
// Combiner: Take multiple sets of UUIDs, combines into one
//   AND
//   OR
//
// Transformers (get a single set)
//   Sorting: Sort Set of UUIDs
//   Filter: filter by something

#[derive(Clone,Debug,PartialEq,Eq)]
pub enum Filter {
    TermExists(String),
    IntInRange(Bound<i64>, Bound<i64>),
}

pub type Target = Metakey;

#[derive(Clone,Debug,PartialEq,Eq)]
pub enum QueryT {
    F(Filter, Target),
    OR(Box<QueryT>, Box<QueryT>),
    AND(Box<QueryT>, Box<QueryT>),
    NOT(Box<QueryT>)
}

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Query {
    root: QueryT,
}

pub struct Querier<'env, T> {
    txn: &'env T,
    db: &'env Database,
}

impl<'env, T: Transaction> Querier<'env, T> {
    pub fn new(txn: &'env T, db: &'env Database) -> Self {
        Self { txn, db }
    }
    pub fn run(&mut self, query: Query) -> Result<HashSet<UUID>> {
        self.runT(query.root)
    }
    pub fn runT(&mut self, query: QueryT) -> Result<HashSet<UUID>> {
        // TODO: Keep the error path instead of bubbling it up throwing away that info
        match query {
            QueryT::F(filter, target) => {
                self.filter(filter, target)
            }
            QueryT::OR(a, b) => {
                let a_set = self.runT(*a)?;
                let b_set = self.runT(*b)?;
                Ok(HashSet::union(&a_set, &b_set).map(|u| *u).collect())
            }
            QueryT::AND(a, b) => {
                let a_set = self.runT(*a)?;
                let b_set = self.runT(*b)?;
                Ok(HashSet::intersection(&a_set, &b_set).map(|u| *u).collect())
            }
            QueryT::NOT(a) => {
                let a_set = self.runT(*a)?;
                let all_set = self.all()?;
                Ok(HashSet::difference(&all_set, &a_set).map(|u| *u).collect())
            }
        }
    }

    pub fn filter(&mut self, filter: Filter, target: Target) -> Result<HashSet<UUID>> {
        // 1: Check schema if that filter is valid for that target (typecheck)
        // 2: Figure out where the index for that target is (if any!)
        // 3a: If there is an index, do a fast indexed search
        // 3b: If there is no index, do a slow iterating search
        // 4: Collate, Return
        // 5: ...
        // 6: PROFIT!

        if let Some(i) = self.db.indices.get(&target) {
            match (i,filter) {
                (Index::IntMap(db), Filter::IntInRange(lower,upper)) => {
                    Ok(db.map.range((lower,upper)).map(|(_,u)| *u).collect())
                }
                (Index::Term(db), Filter::TermExists(ref term)) => {
                    db.lookup(self.txn, &term).map(|m| m.into_set())
                }
                _ => Err(Error::QueryType),
            }
        } else {
            // Iterative search
            // TODO: Actually do the search.
            Err(Error::QueryIterating)
        }
    }

    pub fn all(&mut self) -> Result<HashSet<UUID>> {
        let i = self.db.entries.iter_start(self.txn)?;
        i.map(|r| {
            let (b,_) = r?;
            let (int_bytes, _rest) = b.split_at(std::mem::size_of::<u128>());
            let u = u128::from_le_bytes(int_bytes.try_into().unwrap());
            Ok(UUID::from_u128(u))
        }).collect()
    }
}

// 'python OR raspberry or pi' => "title:python OR title:raspberry OR title:pi"
// 'python AND raspberry description:pi' => "title:python AND title:raspberry OR description:pi"
// 'date:[2019..2020]' for range query

pub fn parse(query: &str) -> Result<Query> {
    enum C { OR, AND };

    let mut a: Option<QueryT> = None;
    let mut comb = C::OR;
    for word in query.split_whitespace() {
        let mut step = None;
        if let Some(i) = word.find(':') {
            let (target, rest) = word.split_at(i);
            let filter = &rest[1..];

            let f = parse_f(filter)?;
            step.replace(Box::new(QueryT::F(f, Metakey::from_str(target)?)));
        } else {
            match word {
                "OR" | "or" => comb = C::OR,
                "AND" | "and" => comb = C::AND,
                _ => {
                    let f = parse_f(word)?;
                    step.replace(Box::new(QueryT::F(f, Metakey::Title)));
                }
            }
        }

        if a.is_none() {
            if let Some(s) = step {
                a.replace(*s);
            }
        } else {
            if let Some(s) = step {
                let a2 = a.take().unwrap();
                a.replace(match comb {
                    C::OR => QueryT::OR(Box::new(a2), s),
                    C::AND => QueryT::AND(Box::new(a2), s),
                });
                comb = C::OR;
            }
        }
    }

    Ok(Query {
        root: a.unwrap(),
    })
}

pub fn parse_f(filter: &str) -> Result<Filter> {
    if filter.starts_with('[') {
        // Range query
        let m: &[_] = &['[', ']'];
        let inner = filter.trim_matches(m);
        let mut i = inner.split("..");
        let lower = i.next().ok_or(Error::QueryUnexpectedEOS)?;
        let upper = i.next().ok_or(Error::QueryUnexpectedEOS)?;
        let lower_b = if lower.is_empty() {
                Bound::Unbounded
            } else {
                Bound::Included(lower.parse()?)
            };
        let upper_b = if upper.is_empty() {
                Bound::Unbounded
            } else {
                Bound::Included(upper.parse()?)
            };

        Ok(Filter::IntInRange(lower_b, upper_b))
    } else {
        // Term query
        Ok(Filter::TermExists(filter.to_string()))
    }
}

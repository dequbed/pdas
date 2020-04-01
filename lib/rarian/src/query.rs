use std::collections::HashSet;
use std::ops::Bound;
use std::convert::TryInto;

use lmdb::Transaction;

use crate::error::*;
use crate::db::meta::Metakey;

use crate::db::{
    Database,
    Index,
    EntryDB,
    RangeDB,
    entry::Entry,
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

#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub enum Filter<'s> {
    TermExists(&'s str),
    IntInRange(Bound<u64>, Bound<u64>),
}

pub type Target = Metakey;

#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub enum Query<'s> {
    F(Filter<'s>, Target),
    OR(&'s Query<'s>, &'s Query<'s>),
    AND(&'s Query<'s>, &'s Query<'s>),
    NOT(&'s Query<'s>)
}

pub struct Querier<'env, T> {
    txn: T,
    db: &'env Database<'env>,
}

impl<'env, T: Transaction> Querier<'env, T> {
    pub fn new(txn: T, db: &'env Database<'env>) -> Self {
        Self { txn, db }
    }
    pub fn run(&mut self, query: Query) -> Result<HashSet<UUID>> {
        // TODO: Keep the error path instead of bubbling it up throwing away that info
        match query {
            Query::F(filter, target) => {
                self.filter(filter, target)
            }
            Query::OR(a, b) => {
                let a_set = self.run(*a)?;
                let b_set = self.run(*b)?;
                Ok(HashSet::union(&a_set, &b_set).map(|u| *u).collect())
            }
            Query::AND(a, b) => {
                let a_set = self.run(*a)?;
                let b_set = self.run(*b)?;
                Ok(HashSet::intersection(&a_set, &b_set).map(|u| *u).collect())
            }
            Query::NOT(a) => {
                let a_set = self.run(*a)?;
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
                (Index::Term(db), Filter::TermExists(term)) => {
                    db.get(&self.txn, term).map(|m| m.into_set())
                }
                _ => Err(Error::QueryType),
            }
        } else {
            // Iterative search
            Err(Error::QueryIterating)
        }
    }

    pub fn all(&mut self) -> Result<HashSet<UUID>> {
        let i = self.db.entries.iter_start(&self.txn)?;
        i.map(|r| {
            let (b,_) = r?;
            let (int_bytes, _rest) = b.split_at(std::mem::size_of::<u128>());
            let u = u128::from_le_bytes(int_bytes.try_into().unwrap());
            Ok(UUID::from_u128(u))
        }).collect()
    }
}

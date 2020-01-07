#![allow(unused_imports)]
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate futures;

mod error;

// Storage of entries & indices
pub mod db;
// Creation, updating & managing of indices
pub mod index;
// Querying indices and entries
pub mod query;

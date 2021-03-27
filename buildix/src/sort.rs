#![warn(missing_debug_implementations)]
#![allow(unused_imports)]

use sqlx::Database;
use std::fmt::Debug;

pub trait Sorter {
    fn sort<DB: Database>(&self, ident: &str) -> Option<String>;
}

// direction in which to go
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Sort {
    Asc,
    Desc,
}

// provide defaults
impl Default for Sort {
    fn default() -> Self {
        Self::Asc
    }
}

// implement for sort
impl Sorter for Sort {
    fn sort<DB: Database>(&self, ident: &str) -> Option<String> {
        Some(format!(
            "{} {}",
            ident,
            match self {
                Self::Asc => "ASC",
                Self::Desc => "DESC",
            }
        ))
    }
}

// implement for option
impl Sorter for Option<Sort> {
    fn sort<DB: Database>(&self, ident: &str) -> Option<String> {
        match self {
            Some(t) => t.sort::<DB>(ident),
            None => None,
        }
    }
}

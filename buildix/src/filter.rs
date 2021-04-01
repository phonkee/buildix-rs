#![allow(dead_code)]
#![allow(unused_macros)]
#![allow(unused_imports)]

use sqlx::query::QueryAs;
use sqlx::{Database, Encode, Type};

// FilterResult returns sql clause as well as values assigned.
#[derive(Default)]
pub struct FilterResult
where
    T: Database,
{
    pub clause: String,
    pub count: usize,
}

// FilterResult implementation
impl FilterResult {
    pub fn new(clause: String, count: usize) -> Self {
        Self {
            clause: clause.trim().to_owned(),
            count,
        }
    }
}

// FilterInfo is passed into filter
#[derive(Clone, Debug, Default)]
pub struct FilterInfo<'a> {
    pub ident: &'a str,
    pub expr: Option<String>,
    pub counter: usize,
    pub isnull: bool,
}

// Filter trait
pub trait Filter {
    // filter_query returns clause if available
    fn filter_query<DB: Database>(&self, info: &FilterInfo) -> Option<String>;

    // filter arguments
    fn filter_arguments<DB, O>(&self, query: QueryAs<DB, O, true>)
    where
        DB: Database;
}

// Nullable is marker trait for fields that support `isnull`
pub trait Nullable {}

pub mod fields {
    use super::{Filter, FilterInfo, FilterResult};
    use crate::filter::Nullable;
    use sqlx::query::QueryAs;
    use sqlx::{Database, IntoArguments};

    // IsNull field that transforms into ISNULL, NOT ISNULL
    // also works with Option seamlessly (as usual)
    #[derive(Debug, Default, Eq, PartialEq)]
    pub struct IsNull(bool);

    // convert from bool (and back - thanks rust)
    impl From<bool> for IsNull {
        fn from(b: bool) -> Self {
            Self(b)
        }
    }

    impl Nullable for IsNull {}

    // implement filter for isnull
    impl Filter for IsNull {
        fn filter_query<DB: Database>(&self, info: &FilterInfo) -> Option<String> {
            match self.0 {
                true => Some(format!("{} ISNULL", info.ident)),
                false => Some(format!("{} NOT ISNULL", info.ident)),
            }
        }

        fn filter_arguments<DB, O>(&self, query: QueryAs<DB, O, A>)
        where
            DB: Database,
            O: for<'r> sqlx::FromRow<'r, <DB as Database>::Row>,
            A: IntoArguments<DB>,
        {
            // no-op
        }
    }
}

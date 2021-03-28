#![allow(dead_code)]
#![allow(unused_macros)]
#![allow(unused_imports)]

use sqlx::{Database, Type};

// FilterResult returns sql clause as well as values assigned.
#[derive(Default)]
pub struct FilterResult {
    pub clause: String,
    pub values: Vec<()>,
    pub count: usize,
}

// FilterResult implementation
impl FilterResult {
    pub fn new(clause: String, values: Vec<()>, count: usize) -> Self {
        Self {
            clause: clause.trim().to_owned(),
            values,
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
    fn process_filter<DB: Database>(&self, info: &FilterInfo) -> Option<FilterResult>;
}

// Nullable is marker trait for fields that support `isnull`
pub trait Nullable {}

pub mod fields {
    use super::{Filter, FilterInfo, FilterResult};
    use crate::filter::Nullable;
    use sqlx::Database;

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
        fn process_filter<DB: Database>(&self, info: &FilterInfo) -> Option<FilterResult> {
            match self.0 {
                true => Some(FilterResult::new(
                    format!("{} ISNULL", info.ident),
                    vec![],
                    1,
                )),
                false => Some(FilterResult::new(
                    format!("{} NOT ISNULL", info.ident),
                    vec![],
                    1,
                )),
            }
        }
    }
}

#![allow(dead_code)]
#![allow(unused_macros)]
#![allow(unused_imports)]

use sqlx::query::QueryAs;
use sqlx::{Database, Encode, IntoArguments, Type};

// Filter trait
pub trait Filter {
    // filter_query returns clause if available
    fn filter_query<DB: Database>(&self, info: &FilterInfo) -> Option<String>;

    // bind_values
    fn bind_values<'q, DB, O, T>(&self, query: QueryAs<'q, DB, O, T>) -> QueryAs<'q, DB, O, T>
    where
        DB: Database,
        T: IntoArguments<'q, DB>,
    {
        query
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

// Nullable is marker trait for fields that support `isnull`
pub trait Nullable {}

pub mod fields {
    use super::{Filter, FilterInfo};
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

        // fn filter_arguments<'q, DB, O, A>(
        //     &self,
        //     query: QueryAs<'q, DB, O, A>,
        // ) -> QueryAs<'q, DB, O, A>
        // where
        //     DB: Database,
        //     A: IntoArguments<'q, DB>,
        // {
        //     query
        // }
    }
}

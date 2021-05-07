#![allow(dead_code)]
#![allow(unused_macros)]
#![allow(unused_imports)]
#![allow(unused_braces)]

use sqlx::any::AnyArguments;
#[allow(late_bound_lifetime_arguments)]
use sqlx::query::QueryAs;
use sqlx::{Database, Encode, IntoArguments, Type};

// Filter trait
pub trait Filter {
    // filter_query returns clause if available
    fn filter_query<DB: Database>(&self, info: &FilterInfo) -> Option<String>;

    // prepare_arguments
    fn prepare_arguments<'a, 'b>(&'a self, arguments: &'b mut AnyArguments)
    where
        'a: 'b;
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
    use sqlx::any::AnyArguments;
    use sqlx::query::QueryAs;
    use sqlx::{Database, IntoArguments};

    // IsNull field that transforms into ISNULL, NOT ISNULL
    // also works with Option seamlessly (as usual)
    #[derive(Debug, Default, Eq, PartialEq, sqlx::Type)]
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

        fn prepare_arguments<'a, 'b>(&'a self, _: &'b mut AnyArguments)
        where
            'a: 'b,
        {
            // no-op
        }
    }
}

#![allow(unused_imports)]

pub use super::delete::DeleteBuilder;
pub use super::select::{Select, SelectBuilder};
pub use super::sort::Sort;
use crate::filter::Nullable;
use crate::filter::{Filter, FilterInfo};
use sqlx::database::Database;
use sqlx::query::QueryAs;
use sqlx::IntoArguments;

#[macro_export]
macro_rules! filter_impl {
    {$A:ty} => {
        impl Filter for $A {

            // return query
            fn filter_query<DB: Database>(&self, fi: &FilterInfo) -> Option<String> {
                // now we are not none
                if let Some(expr) = &fi.expr {
                    Some(expr.clone())
                } else {
                    Some(format!("{} = ?", fi.ident))
                }
            }

            // // add arguments to query
            // fn filter_arguments<'q, DB: Database, O, T>(&self, query: QueryAs<'q, DB, O, T>) -> QueryAs<'q, DB, O, T>
            // where
            //     DB: Database,
            //     T: IntoArguments<'q, DB>,
            // {
            //     let mut query = query;
            //     query.bind(self)
            // }

        }
    };
}

filter_impl!(&i32);
filter_impl!(i32);
filter_impl!(i64);
filter_impl!(&i64);
filter_impl!(String);
filter_impl!(&String);
filter_impl!(&str);

// implement filter for option
impl<T> Filter for Option<T>
where
    T: Filter,
{
    fn filter_query<DB: Database>(&self, fi: &FilterInfo) -> Option<String> {
        match self {
            None => {
                if fi.isnull {
                    Some(format!("{} ISNULL", fi.ident).to_string())
                } else {
                    None
                }
            }
            Some(val) => val.filter_query::<DB>(fi),
        }
    }

    // fn filter_arguments<'q, DB: Database, O, V>(
    //     &self,
    //     query: QueryAs<'q, DB, O, V>,
    // ) -> QueryAs<'q, DB, O, V>
    // where
    //     DB: Database,
    //     V: IntoArguments<'q, DB>,
    // {
    //     query
    // }
}

// Option is nullable
impl<T> Nullable for Option<T> {}

// add IN
impl<T> Filter for Vec<T>
where
    T: Filter,
{
    fn filter_query<DB: Database>(&self, info: &FilterInfo) -> Option<String> {
        let len = self.len();
        if len == 0 {
            None
        } else {
            let placeholders: Vec<String> = [0..len].iter().map(|_| "?".to_string()).collect();
            Some(format!("{} IN ({})", info.ident, placeholders.join(", ")))
        }
    }

    // fn filter_arguments<'q, DB, O, V>(&self, query: QueryAs<DB, O, V>) -> QueryAs<'q, DB, O, V>
    // where
    //     DB: Database,
    //     V: IntoArguments<'q, DB>,
    // {
    //     let mut query = query;
    //
    //     // apply values
    //     for value in self {
    //         query = query.bind()
    //     }
    //
    //     query
    // }
}

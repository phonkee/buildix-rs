#![allow(unused_imports)]
#![allow(unused_variables)]

pub use super::delete::DeleteBuilder;
pub use super::select::{Select, SelectBuilder};
pub use super::sort::Sort;
use crate::filter::Nullable;
use crate::filter::{Filter, FilterInfo};
use sqlx::any::AnyArguments;
use sqlx::database::Database;
use sqlx::query::QueryAs;
use sqlx::{Arguments, Encode, Type};

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

            fn prepare_arguments<'a, 'b>(&'a self, arguments: &'b mut ::sqlx::any::AnyArguments) where 'a: 'b{
               arguments.add(self);
            }
        }
    };
}

filter_impl!(bool);
filter_impl!(i64);
filter_impl!(String);

// filter_impl!(&i32);
// filter_impl!(i32);
// filter_impl!(&i64);
// filter_impl!(&String);
// filter_impl!(&str);

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

    fn prepare_arguments<'a, 'b>(&'a self, arguments: &'b mut AnyArguments)
    where
        'a: 'b,
    {
        match self {
            None => {}
            Some(val) => {
                val.prepare_arguments(arguments);
            }
        };
    }
}

// Option is nullable
impl<T> Nullable for Option<T> {}

// add IN
impl<T> Filter for Vec<T>
where
    T: Filter + Send + Sync,
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

    fn prepare_arguments<'a, 'b>(&'a self, arguments: &'b mut AnyArguments)
    where
        'a: 'b,
    {
        // apply values
        for value in self {
            value.prepare_arguments(arguments);
        }
    }
}

#![allow(unused_imports)]

pub use super::select::{Select, SelectBuilder};
pub use super::sort::Sort;
use crate::filter::Nullable;
use crate::filter::{Filter, FilterInfo, FilterResult};

#[macro_export]
macro_rules! filter_impl {
    {$A:ty} => {
        impl Filter for $A {
            fn process_filter(&self, fi: &FilterInfo) -> Option<FilterResult> {
                // now we are not none
                if let Some(expr) = &fi.expr {
                    Some(FilterResult::new(expr.clone(), vec![()], 1))
                } else {
                    Some(FilterResult::new(format!("{} = ?", fi.ident), vec![()], 1))
                }
            }
        }
    };
}

filter_impl!(&i32);
filter_impl!(i32);
filter_impl!(i64);
filter_impl!(&i64);
filter_impl!(String);
filter_impl!(&String);

// implement filter for option
impl<T> Filter for Option<T>
where
    T: Filter,
{
    fn process_filter(&self, fi: &FilterInfo) -> Option<FilterResult> {
        match self {
            None => {
                if fi.isnull {
                    Some(FilterResult::new(
                        format!("{} ISNULL", fi.ident).to_string(),
                        vec![],
                        1,
                    ))
                } else {
                    None
                }
            }
            Some(val) => val.process_filter(fi),
        }
    }
}

// Option is nullable
impl<T> Nullable for Option<T> {}

// add IN
impl<T> Filter for Vec<T>
where
    T: Filter,
{
    fn process_filter(&self, info: &FilterInfo) -> Option<FilterResult> {
        let len = self.len();
        if len == 0 {
            None
        } else {
            let placeholders: Vec<String> = [0..len].iter().map(|_| "?".to_string()).collect();
            Some(FilterResult::new(
                format!("{} IN ({})", info.ident, placeholders.join(", ")),
                vec![(); len],
                len,
            ))
        }
    }
}

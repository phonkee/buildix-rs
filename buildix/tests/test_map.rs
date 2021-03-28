#![allow(dead_code)]
#![allow(unused_imports)]

use buildix::sort::Sort;
use buildix_derive::{Filter, Select, SelectBuilder};

#[allow(unused_imports)]
use buildix::prelude::*;
use sqlx::Postgres;
use thiserror::Error;

#[test]
fn test_map() {
    let mut query = TestSelectBuilder::default();
    assert!(query.to_sql::<Postgres>().is_err());
}

#[derive(Default, SelectBuilder)]
#[buildix(map = "map_select")]
pub struct TestSelectBuilder {
    #[buildix(select)]
    select: Vec<SelectUser>,

    #[buildix(sort = "name")]
    sort_name: Option<buildix::sort::Sort>,

    #[buildix(sort = "age")]
    sort_age: buildix::sort::Sort,
}

#[derive(Debug, Error)]
pub enum CustomError {
    #[error("this is error")]
    Error,
}

// map_select is called before execute, and when error is returned, it is returned back.
// this is useful for various validations.
pub fn map_select(_: &mut TestSelectBuilder) -> buildix::Result<()> {
    Err(buildix::Error::MapError(Box::new(CustomError::Error)))
}

#[derive(Default, Select)]
#[buildix(from(table(name = "user")))]
struct SelectUser {
    // simple value
    name: String,

    // simple value
    email: String,

    // nullable value
    #[buildix(table = "user")]
    age: Option<i64>,

    #[buildix(expr = "IF(age > 18, true, false)")]
    is_adult: bool,

    #[buildix(expr = r#"COALESCE(other, "")"#)]
    other: String,

    #[buildix(table = "user", column = "column")]
    some_other: String,
}

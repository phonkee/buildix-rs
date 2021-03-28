#![allow(dead_code)]
#![allow(unused_imports)]

use buildix::sort::Sort;
use buildix_derive::{DeleteBuilder, Filter};

#[allow(unused_imports)]
use buildix::prelude::*;
use sqlx::Postgres;
use thiserror::Error;

#[test]
fn test_map() {
    let mut query = TestDeleteBuilder::default();
    assert!(query.to_sql::<Postgres>().is_err());
}

#[derive(Default, DeleteBuilder)]
#[buildix(table = "user")]
pub struct TestDeleteBuilder {
    #[buildix(filter)]
    id: i32,

    #[buildix(limit)]
    limit: i32,
}

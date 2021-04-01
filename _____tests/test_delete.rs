#![allow(dead_code)]
#![allow(unused_imports)]

use buildix::sort::Sort;
use buildix_derive::{DeleteBuilder, Filter};

#[allow(unused_imports)]
use buildix::prelude::*;
use sqlx::Postgres;
use thiserror::Error;

#[test]
fn test_delete() {
    let mut query = TestDeleteBuilder::default();
    let (q, _) = query.to_sql::<Postgres>().unwrap();
    assert_eq!(q, "DELETE FROM user WHERE id = ?");

    query.limit = Some(42);
    let (q, _) = query.to_sql::<Postgres>().unwrap();

    assert_eq!(q, "DELETE FROM user WHERE id = ? LIMIT 42");
}

#[derive(Default, DeleteBuilder)]
#[buildix(table = "user")]
pub struct TestDeleteBuilder {
    #[buildix(filter)]
    id: i32,

    #[buildix(limit)]
    limit: Option<i32>,
}

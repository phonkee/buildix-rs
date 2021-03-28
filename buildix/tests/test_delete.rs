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
    println!("delete query: {}", q);
}

#[derive(Default, DeleteBuilder)]
#[buildix(table = "user")]
pub struct TestDeleteBuilder {
    #[buildix(filter)]
    id: i32,

    #[buildix(limit)]
    limit: i32,
}

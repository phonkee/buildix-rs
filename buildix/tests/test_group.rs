#![allow(dead_code)]
#![allow(unused_imports)]

use buildix::sort::Sort;
use buildix_derive::{Filter, Select, SelectBuilder};

#[allow(unused_imports)]
use buildix::prelude::*;
use sqlx::Postgres;

#[test]
fn test_group() {
    let mut query = GroupQueryBuilder::default();
    let (q, _) = query.to_sql::<Postgres>();
    assert_eq!(q, r#"SELECT id FROM user GROUP BY name, age, email"#);
}

#[derive(Default, SelectBuilder)]
struct GroupQueryBuilder {
    #[buildix(select)]
    select: Vec<SelectUser>,
}

#[derive(Default, Select)]
#[buildix(from(table(name = "user")))]
#[buildix(group = "name", group = "age", group = "email")]
struct SelectUser {
    id: i32,
}

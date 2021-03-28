#![allow(dead_code)]
#![allow(unused_imports)]

use buildix::sort::Sort;
use buildix_derive::{Filter, Select, SelectBuilder};

#[allow(unused_imports)]
use buildix::prelude::*;
use sqlx::Postgres;

#[test]
fn test_simple() {
    let mut query = JoinQueryBuilder::default();
    let (q, _v) = query.to_sql::<Postgres>();

    assert_eq!(
        q,
        r#"SELECT name, email, age, IF(age > 18, true, false) AS is_adult, COALESCE(other, "") AS other FROM user AS u, INNER JOIN order o (o.user_id = u.id) ORDER BY age ASC"#
    );
}

#[test]
fn test_sort() {
    let mut query = JoinQueryBuilder::default();
    query.sort_name = Some(Sort::Asc);
    query.sort_age = Sort::Desc;

    let (q, _v) = query.to_sql::<Postgres>();

    assert_eq!(
        q,
        r#"SELECT name, email, age, IF(age > 18, true, false) AS is_adult, COALESCE(other, "") AS other FROM user AS u, INNER JOIN order o (o.user_id = u.id) ORDER BY name ASC, age DESC"#
    );
}

#[derive(Default, SelectBuilder)]
struct JoinQueryBuilder {
    #[buildix(select)]
    select: Vec<SelectUser>,

    #[buildix(sort = "name")]
    sort_name: Option<buildix::sort::Sort>,

    #[buildix(sort = "age")]
    sort_age: buildix::sort::Sort,
}

#[derive(Default, Select)]
#[buildix(from(table(name = "user", alias = "u")))]
#[buildix(from(join(name = "order", alias = "o", on = "o.user_id = u.id")))]
struct SelectUser {
    // simple value
    name: String,

    // simple value
    email: String,

    // nullable value
    age: Option<i64>,

    #[buildix(expr = "IF(age > 18, true, false)")]
    is_adult: bool,

    #[buildix(expr = r#"COALESCE(other, "")"#)]
    other: String,
}

#![allow(dead_code)]
#![allow(unused_imports)]

use buildix::sort::Sort;
use buildix_derive::{Filter, Select, SelectBuilder};

#[allow(unused_imports)]
use buildix::prelude::*;
use sqlx::Postgres;

#[test]
fn test_simple() {
    let mut query = TestSelectBuilder::default();
    let (q, _) = query.get_query::<Postgres>();
    assert_eq!(
        q,
        r#"SELECT name, email, user.age, IF(age > 18, true, false) AS is_adult, COALESCE(other, "") AS other, user.column AS some_other FROM user ORDER BY age ASC"#
    );
}

#[test]
fn test_sort() {
    let mut query = TestSelectBuilder::default();
    query.sort_name = Some(Sort::Asc);
    query.sort_age = Sort::Desc;
    let (q, _) = query.get_query::<Postgres>();

    assert_eq!(
        q,
        r#"SELECT name, email, user.age, IF(age > 18, true, false) AS is_adult, COALESCE(other, "") AS other, user.column AS some_other FROM user ORDER BY name ASC, age DESC"#
    );
}

#[derive(Default, SelectBuilder)]
struct TestSelectBuilder {
    #[buildix(select)]
    select: Vec<SelectUser>,

    #[buildix(sort = "name")]
    sort_name: Option<buildix::sort::Sort>,

    #[buildix(sort = "age")]
    sort_age: buildix::sort::Sort,
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

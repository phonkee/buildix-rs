#![allow(dead_code)]
#![allow(unused_imports)]

use buildix::sort::Sort;
use buildix_derive::{Filter, Select, SelectBuilder};

#[allow(unused_imports)]
use buildix::prelude::*;
use sqlx::Postgres;

#[test]
fn test_readme_select() {
    let mut qb = SelectUserBuilder::default();
    let (q, _) = qb.to_sql::<Postgres>().unwrap();

    assert_eq!(
        q,
        r#"SELECT u.name, u.email, u.custom_age AS age, IF(age > 18, true, false) AS is_adult, COALESCE(other, "") AS other FROM user AS u, INNER JOIN order o (o.user_id = u.id) WHERE (priority = ? OR age ISNULL) GROUP BY name, email ORDER BY age ASC"#
    );

    qb.filter.inner.value = Some(42);
    qb.filter.inner.value2 = Some(314);
    let (q, _) = qb.to_sql::<Postgres>().unwrap();

    assert_eq!(
        q,
        r#"SELECT u.name, u.email, u.custom_age AS age, IF(age > 18, true, false) AS is_adult, COALESCE(other, "") AS other FROM user AS u, INNER JOIN order o (o.user_id = u.id) WHERE (priority = ? OR age ISNULL OR (value = ? AND value2 = ?)) GROUP BY name, email ORDER BY age ASC"#
    );
}

#[derive(Default, SelectBuilder)]
struct SelectUserBuilder {
    #[buildix(select)]
    select: Vec<SelectUser>,

    #[buildix(filter)]
    filter: Filter,

    #[buildix(limit)]
    limit: Option<i32>,

    #[buildix(offset)]
    offset: i32,

    #[buildix(sort = "name")]
    sort_name: Option<buildix::sort::Sort>,

    #[buildix(sort = "age")]
    sort_age: buildix::sort::Sort,

    #[buildix(count)]
    count: i32,
}

#[derive(Default, Select)]
#[buildix(from(table(name = "user", alias = "u")))]
#[buildix(from(join(name = "order", alias = "o", on = "o.user_id = u.id")))]
#[buildix(group = "name", group = "email")]
struct SelectUser {
    #[buildix(table = "u")]
    name: String,

    #[buildix(table = "u")]
    email: String,

    #[buildix(table = "u", column = "custom_age")]
    age: Option<i64>,

    #[buildix(expr = "IF(age > 18, true, false)")]
    is_adult: bool,

    #[buildix(expr = r#"COALESCE(other, "")"#)]
    other: String,
}

#[derive(Default, Filter)]
#[buildix(operator = "OR")]
struct Filter {
    author_id: Option<i32>,

    #[buildix(expr = "last_updated < ?")]
    last_updated: Option<i32>,

    // automatically provides filter = "priority = ?"
    priority: i32,

    #[buildix(expr = "age > ?", isnull)]
    age: Option<i32>,

    // inner filter will be sub clause in parentheses (if needed)
    inner: InnerFilter,

    // Vec automatically converts to IN(...), if no value is available
    // this filter will not be available in where clause
    id: Vec<i32>,
}

// even multiple filters supported
#[derive(Default, Filter)]
#[buildix(operator = "AND")]
struct InnerFilter {
    value: Option<i32>,
    value2: Option<i32>,
}

#![allow(dead_code)]
#![allow(late_bound_lifetime_arguments)]

use buildix_derive::{Filter, Select, SelectBuilder};

#[allow(unused_imports)]
use buildix::prelude::*;
use sqlx::postgres::PgPoolOptions;
use sqlx::Postgres;
use tokio_test::block_on;

#[test]
fn test_filter() {
    let mut query = FilterQuery::default();

    // now execute
    block_on(async {
        let _pool = PgPoolOptions::new()
            .max_connections(5)
            .connect("postgres://virus:virus@localhost:5432/virus")
            .await
            .unwrap();

        let (q, _) = query.to_sql::<Postgres>().unwrap();

        assert_eq!(
            q,
            r#"SELECT u.id FROM user AS u WHERE (priority = ? AND age ISNULL)"#
        );
    });

    query.filter.author_id = Some(2);
    let (q, _) = query.to_sql::<Postgres>().unwrap();

    assert_eq!(
        q,
        r#"SELECT u.id FROM user AS u WHERE (author_id = ? AND priority = ? AND age ISNULL)"#
    );

    query.filter.last_updated = Some(12345);
    let (q, _) = query.to_sql::<Postgres>().unwrap();
    assert_eq!(
        q,
        r#"SELECT u.id FROM user AS u WHERE (author_id = ? AND last_updated < ? AND priority = ? AND age ISNULL)"#
    );

    query.filter.something = Some(false.into());
    let (q, _) = query.to_sql::<Postgres>().unwrap();

    assert_eq!(
        q,
        r#"SELECT u.id FROM user AS u WHERE (author_id = ? AND last_updated < ? AND priority = ? AND age ISNULL AND something NOT ISNULL)"#
    );

    query.filter.inner.inner_id = Some(42);
    let (q, _) = query.to_sql::<Postgres>().unwrap();
    assert_eq!(
        q,
        r#"SELECT u.id FROM user AS u WHERE (author_id = ? AND last_updated < ? AND priority = ? AND age ISNULL AND something NOT ISNULL AND inner_id = ?)"#
    );

    query.filter.inner.second = Some(314);
    let (q, _) = query.to_sql::<Postgres>().unwrap();
    assert_eq!(
        q,
        r#"SELECT u.id FROM user AS u WHERE (author_id = ? AND last_updated < ? AND priority = ? AND age ISNULL AND something NOT ISNULL AND (inner_id = ? OR second = ?))"#
    );
}

#[derive(Default, SelectBuilder)]
struct FilterQuery {
    #[buildix(select)]
    select: Vec<SelectUser>,

    #[buildix(filter)]
    filter: Filter,
}

#[derive(Default, Filter)]
struct Filter {
    #[buildix(expr = "author_id = ?")]
    author_id: Option<i64>,

    #[buildix(expr = "last_updated < ?")]
    last_updated: Option<i64>,

    // automatically provides filter = "priority = ?"
    priority: i64,

    #[buildix(expr = "age > ?", isnull)]
    age: Option<i64>,

    // something
    something: Option<buildix::filter::fields::IsNull>,

    // inner filter
    inner: InnerFilter,
}

#[derive(Debug, Default, Filter)]
#[buildix(operator = "OR")]
struct InnerFilter {
    inner_id: Option<i64>,
    second: Option<i64>,
}

#[derive(Default, Select)]
#[buildix(from(table(name = "user", alias = "u")))]
#[derive(sqlx::FromRow)]
struct SelectUser {
    #[buildix(table = "u")]
    id: i64,
}

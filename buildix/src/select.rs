#![allow(unused_imports)]

use async_trait::async_trait;
use sqlx::database::Database;

#[cfg(feature = "postgres")]
use sqlx::postgres::Postgres;

#[cfg(feature = "mysql")]
use sqlx::mysql::MySql;

use sqlx::query::QueryAs;
use sqlx::{Error, IntoArguments};
use sqlx::{FromRow, Pool};

// select query implementation
pub trait SelectBuilder {
    // returns query
    fn to_sql<DB: Database>(&mut self) -> crate::Result<(String, Vec<()>)>;
    fn prepare_values<'q, DB, O, T>(
        &mut self,
        query: QueryAs<'q, DB, O, T>,
    ) -> QueryAs<'q, DB, O, T>
    where
        DB: Database,
        T: IntoArguments<'q, DB>;
}

// Query trait
// @TODO: change to static methods
pub trait Select {
    fn get_fields<DB: Database>(&self) -> &'static [&'static str];
    fn get_fields_str<DB: Database>(&self) -> &'static str;
    fn get_table<DB: Database>(&self) -> &'static str;
    fn get_query<DB: Database>(&self) -> &'static str;
    fn get_group<DB: Database>(&mut self) -> Option<&'static str>;

    // instantiate new query
    fn new_query<'q, DB, O, T>(query: String) -> QueryAs<'q, DB, O, T>
    where
        DB: sqlx::Database,
        T: sqlx::IntoArguments<'q, DB>,
    {
        sqlx::query_as::<_, Self>(&query)
    }
}

// implement Query for Vec<Query>
// @TODO: change to static methods
impl<T> Select for Vec<T>
where
    T: Select + Default,
{
    fn get_fields<DB: Database>(&self) -> &'static [&'static str] {
        T::default().get_fields::<DB>()
    }

    fn get_fields_str<DB: Database>(&self) -> &'static str {
        T::default().get_fields_str::<DB>()
    }
    fn get_table<DB: Database>(&self) -> &'static str {
        T::default().get_table::<DB>()
    }
    fn get_query<DB: Database>(&self) -> &'static str {
        T::default().get_query::<DB>()
    }
    fn get_group<DB: Database>(&mut self) -> Option<&'static str> {
        T::default().get_group::<DB>()
    }
    fn new_query<'q, DB, O, A>(query: String) -> QueryAs<'q, DB, O, A>
    where
        DB: sqlx::Database,
        A: sqlx::IntoArguments<'q, DB>,
    {
        sqlx::query_as::<_, T>(&query)
    }
}

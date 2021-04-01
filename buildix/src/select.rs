#![allow(unused_imports)]

use async_trait::async_trait;
use sqlx::database::Database;

#[cfg(feature = "postgres")]
use sqlx::postgres::Postgres;

#[cfg(feature = "mysql")]
use sqlx::mysql::MySql;

use sqlx::query::QueryAs;
use sqlx::Pool;
use sqlx::{Error, IntoArguments};

// select query implementation
pub trait SelectBuilder {
    // returns query
    fn to_sql<DB: Database>(&mut self) -> crate::Result<(String, Vec<()>)>;
    fn bind_values<'q, DB, O, T>(&mut self, query: QueryAs<'q, DB, O, T>) -> QueryAs<'q, DB, O, T>
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
}

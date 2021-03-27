#![allow(unused_imports)]

use async_trait::async_trait;
use sqlx::database::Database;

#[cfg(feature = "postgres")]
use sqlx::postgres::Postgres;

#[cfg(feature = "mysql")]
use sqlx::mysql::MySql;

use sqlx::Error;
use sqlx::Pool;

// select query implementation
pub trait SelectBuilder {
    // returns query
    fn get_query<DB: Database>(&mut self) -> (String, Vec<()>);
}

// Query trait
pub trait Select {
    fn get_fields<DB: Database>(&self) -> &'static [&'static str];
    fn get_fields_str<DB: Database>(&self) -> &'static str;
    fn get_table<DB: Database>(&self) -> &'static str;
    fn get_query<DB: Database>(&self) -> &'static str;
    fn get_group<DB: Database>(&mut self) -> Option<&'static str>;
}

// implement Query for Vec<Query>
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

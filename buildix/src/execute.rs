#![allow(unused_imports)]
use async_trait::async_trait;
use sqlx::database::Database;
use sqlx::postgres::Postgres;
use sqlx::Error;
use sqlx::Pool;

#[async_trait]
pub trait Execute {
    // perform query
    async fn execute<DB: Database>(&mut self, pool: Pool<DB>) -> Result<(), Error>;
}

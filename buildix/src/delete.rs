use sqlx::Database;

// select query implementation
pub trait DeleteBuilder {
    // returns query
    fn to_sql<DB: Database>(&mut self) -> crate::Result<(String, Vec<()>)>;
}

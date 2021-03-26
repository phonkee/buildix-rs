use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("sqlx error: `{0}`")]
    Sqlx(sqlx::error::Error),

    #[error("filter error: `{0}`")]
    Filter(String),

    #[error("select query error: `{0}`")]
    SelectBuilder(String),
}

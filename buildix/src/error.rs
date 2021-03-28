use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("sqlx error: `{0}`")]
    Sqlx(sqlx::error::Error),

    #[error("map error: `{0}`")]
    MapError(Box<dyn std::error::Error>),

    #[error("filter error: `{0}`")]
    FilterError(Box<dyn std::error::Error>),
}

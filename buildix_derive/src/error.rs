#![allow(dead_code)]

use std::fmt::Debug;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Only buildix fields allowed: #[buildix(query)], #[buildix(filter)], #[buildix(offset)], #[buildix(limit)], #[buildix(count)], #[buildix(sort)], #[buildix(group)], #[buildix(having)]")]
    InvalidColumn,

    #[error("Please provide single #[buildix(query)] field")]
    MissingQuery,

    #[error("Please provide only single `{0}` field")]
    Multiple(String),

    #[error("Invalid table definition: `{0}`")]
    InvalidTable(String),

    #[error("Invalid field: please provide either `table` or `expr` or none, but not both")]
    InvalidSelectField,

    #[error("error")]
    Error,

    #[error(
        "Only buildix fields allowed: #[buildix(filter)], #[buildix(count)], #[buildix(limit)]"
    )]
    InvalidDelete,
}

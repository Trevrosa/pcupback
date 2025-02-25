use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug, Serialize)]
pub enum DBErrorKind {
    #[error("db store error")]
    StoreError,
    #[error("db fetch error")]
    FetchError,
}

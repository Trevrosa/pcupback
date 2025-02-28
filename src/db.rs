use serde::Serialize;
use thiserror::Error;

#[allow(unused)]
#[derive(Error, Debug, Serialize)]
pub enum DBErrorKind {
    #[error("StoreError")]
    StoreError(String),
    #[error("FetchError")]
    FetchError(String),
}

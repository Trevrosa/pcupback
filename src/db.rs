use serde::{Deserialize, Serialize};
use thiserror::Error;

#[allow(unused)]
#[derive(Error, Debug, Serialize, Deserialize)]
pub enum DBErrorKind {
    #[error("StoreError")]
    StoreError(String),
    #[error("FetchError")]
    FetchError(String),
}

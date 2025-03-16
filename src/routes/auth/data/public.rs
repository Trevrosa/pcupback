use serde::{Deserialize, Serialize};
use thiserror::Error;

use pcupback::DBErrorKind;

use super::private::DBUserSession;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserSession {
    pub user_id: u32,
    pub id: String,
}

impl From<DBUserSession> for UserSession {
    fn from(value: DBUserSession) -> Self {
        Self {
            user_id: value.user_id,
            id: value.id,
        }
    }
}

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum AuthError {
    #[error("EmptyUsername")]
    EmptyUsername,
    #[error("InvalidPassword")]
    InvalidPassword(#[from] InvalidPasswordKind),
    #[error("WrongPassword")]
    WrongPassword,
    #[error("HashError")]
    HashError(#[from] HashErrorKind),
    #[error("db error")]
    DBError(#[from] DBErrorKind),
    #[error("InternalError")]
    InternalError(String),
}

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum InvalidPasswordKind {
    #[error("TooFewChars")]
    TooFewChars,
    #[error("TooManyChars")]
    TooManyChars,
}

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum HashErrorKind {
    #[error("CreateError")]
    CreateError(String),
    #[error("ParseError")]
    ParseError(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}

impl AuthRequest {
    #[cfg(test)]
    pub fn random_valid() -> Self {
        Self {
            username: uuid::Uuid::new_v4().to_string(),
            password: "12345678".to_string(),
        }
    }
}

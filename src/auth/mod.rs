/// Data structs regarding authorization to be shared in requests
mod structs;

use std::time::{Duration, Instant, SystemTime, SystemTimeError, UNIX_EPOCH};

use argon2::password_hash::{SaltString, rand_core::OsRng};
use chrono::{DateTime, NaiveDateTime, NaiveTime, TimeDelta, Utc};
use rocket::{State, post, serde::json::Json};
use rocket_db_pools::{Connection, sqlx};
use serde::Serialize;
use sqlx::{Pool, Sqlite};
use structs::{
    db::{DBUser, DBUserSession, Storable},
    http::{AuthRequest, UserSession},
};
use thiserror::Error;
use tracing::{Level, span};
use uuid::Uuid;

use crate::{ARGON2, db::DBErrorKind};

#[derive(Error, Debug, Serialize)]
pub enum AuthenticationError {
    #[error("password was invalid")]
    InvalidPassword(#[from] PasswordError),
    #[error("password did not match stored password")]
    WrongPassword,
    #[error("failed to hash password")]
    FailedToHash,
    #[error("db error")]
    DBError(#[from] DBErrorKind),
}

#[derive(Error, Debug, Serialize)]
pub enum PasswordError {
    #[error("not enough characters (min. 6)")]
    NotEnoughChars,
}

const SESSION_TIMEOUT: TimeDelta = TimeDelta::days(1);

fn datetime_from_session(session: &DBUserSession) -> Option<DateTime<Utc>> {
    DateTime::from_timestamp(session.session_id as i64, 0)
}

fn session_no_timeout(dt: DateTime<Utc>) -> bool {
    Utc::now().time() - dt.time() > SESSION_TIMEOUT
}

#[post("/auth", data = "<request>")]
pub async fn authenticate(
    db: &State<Pool<Sqlite>>,
    request: Json<AuthRequest>,
) -> Json<Result<UserSession, AuthenticationError>> {
    use AuthenticationError::*;
    use PasswordError::*;

    // we have &State<Pool<Sqlite>>, so:
    // deref &State<..> => State<..>
    // deref State<T> => T
    let db = &**db;

    let my_span = span!(Level::INFO, "authenticate");
    let _enter = my_span.enter();

    // check the database for a user with the same username requested.
    // get it if it exists.
    let existing_user: Result<DBUser, _> = sqlx::query_as("SELECT * FROM users WHERE username = ?")
        .bind(&request.username)
        .fetch_one(db)
        .await;

    let rx_username = request.username.trim();

    // the user exists
    let session: Result<UserSession, AuthenticationError> = match existing_user {
        Ok(existing_user) => {
            tracing::info!("got auth request for existing user.");

            // hash the received password
            let salt = SaltString::generate(&mut OsRng);
            let mut rx_password_hash = Vec::new();

            let hash_op = ARGON2.hash_password_into(
                request.password.as_ref(),
                salt.as_str().as_ref(),
                &mut rx_password_hash,
            );
            tracing::debug!("hashed password");

            if hash_op.is_err() {
                return Json(Err(FailedToHash));
            }

            if rx_password_hash == existing_user.password_hash.as_bytes() {
                let session: Result<DBUserSession, _> =
                    sqlx::query_as("SELECT last_set FROM sessions WHERE user_id = ?")
                        .bind(existing_user.id)
                        .fetch_one(db)
                        .await;

                match session {
                    Ok(session) => {
                        // if `last_set` was more than `SESSION_TIMEOUT` ago, we create a new session.
                        let session_datetime = datetime_from_session(&session);

                        if let Some(session_datetime) = session_datetime {
                            if session_no_timeout(session_datetime) {
                                let session = DBUserSession::new(existing_user.id);
                                session.store(db).await;
                            }
                        }
                    }
                }

                todo!()
            } else {
                tracing::debug!("mismatched password");
                Err(WrongPassword)
            }
        }
        Err(err) => {
            todo!()
        }
    };

    Ok(Json(session))
}

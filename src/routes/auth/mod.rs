/// Data structs regarding authorization shared in requests
pub mod data;

#[cfg(test)]
mod tests;

use argon2::{Argon2, PasswordHash, PasswordVerifier, password_hash};
use chrono::{DateTime, TimeDelta, Utc};
use data::{
    private::{DBUser, DBUserSession, Storable},
    public::UserSession,
};
use rocket::{
    State, post,
    serde::json::{self, Json},
};
use rocket_db_pools::sqlx;
use serde::{Deserialize, Serialize};
use sqlx::{Executor, Pool, Sqlite};
use thiserror::Error;
use tracing::{Level, span};

use crate::db::DBErrorKind;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum AuthenticationError {
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
    #[error("NotEnoughChars")]
    NotEnoughChars,
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

const SESSION_TIMEOUT: TimeDelta = TimeDelta::days(1);

fn session_timeout(dt: DateTime<Utc>) -> bool {
    Utc::now().time() - dt.time() > SESSION_TIMEOUT
}

/// Check if [`session`] is timed out. If it is, generate and store a new one.
async fn validate_session<'a>(
    executor: impl Executor<'a, Database = Sqlite>,
    session: Result<DBUserSession, sqlx::Error>,
    new_id: u32,
) -> Result<UserSession, AuthenticationError> {
    if let Ok(session) = session {
        // if `last_set` was more than `SESSION_TIMEOUT` ago, we create a new session.
        let session_last_set = session.last_set_datetime();

        if let Some(session_last_set) = session_last_set {
            if session_timeout(session_last_set) {
                tracing::info!("session timed out, generating new one");
                generate_store_session(executor, new_id).await
            } else {
                // session is ok, return it
                Ok(session.into())
            }
        } else {
            generate_store_session(executor, new_id).await
        }
    } else {
        tracing::warn!("no session, generating one");
        generate_store_session(executor, new_id).await
    }
}

// store a session, mapping errors to AuthenticationError, returning Ok(session)
async fn generate_store_session(
    executor: impl Executor<'_, Database = Sqlite>,
    user_id: u32,
) -> Result<UserSession, AuthenticationError> {
    use AuthenticationError::DBError;
    use DBErrorKind::StoreError;

    let session = DBUserSession::generate(user_id);
    match session.store(executor).await {
        // stored session successfully, return
        Ok(_) => Ok(session.into()),
        Err(err) => {
            tracing::error!("failed to store session: {err:?}");
            Err(DBError(StoreError(err.to_string())))
        }
    }
}

#[post("/auth", data = "<request>")]
pub async fn authenticate(
    db: &State<Pool<Sqlite>>,
    request: Json<AuthRequest>,
) -> Json<Result<UserSession, AuthenticationError>> {
    use AuthenticationError::{DBError, HashError, InternalError, InvalidPassword, WrongPassword};
    use DBErrorKind::StoreError;
    use HashErrorKind::ParseError;
    use InvalidPasswordKind::{NotEnoughChars, TooManyChars};

    // we have &State<Pool<Sqlite>>.
    // deref &State<..> => State<..>
    // deref State<T> => T
    // ref T => &T
    // we end up with &Pool<Sqlite>
    let db = &**db;

    // FIXME: use the alternatives shown in macro doc below
    let my_span = span!(Level::INFO, "authenticate");
    let _enter = my_span.enter();

    let req_username = request.username.trim();

    // check the database for a user with the same username requested.
    // get it if it exists.
    let existing_user: Result<DBUser, _> = sqlx::query_as("SELECT * FROM users WHERE username = ?")
        .bind(req_username)
        .fetch_one(db)
        .await;

    let session: Result<UserSession, AuthenticationError> = match existing_user {
        // the user requested exists, lets check if the request password hash matches:
        Ok(existing_user) => {
            tracing::info!("got auth request for existing user.");

            let parse_existing_hash = PasswordHash::new(&existing_user.password_hash);
            let parse_and_validate = parse_existing_hash
                .map(|e_h| Argon2::default().verify_password(request.password.as_bytes(), &e_h));
            tracing::debug!("hashed password");

            match parse_and_validate {
                // the stored password parsed successfully and the request password matched!
                Ok(Ok(())) => {
                    // lets now provide them a session id.
                    tracing::info!("getting session from db for user {}", existing_user.id);

                    let last_set: Result<DBUserSession, _> =
                        sqlx::query_as("SELECT * FROM sessions WHERE user_id = ?")
                            .bind(existing_user.id)
                            .fetch_one(db)
                            .await;

                    validate_session(db, last_set, existing_user.id).await
                }
                // stored password was parsed, but didnt match.
                Ok(Err(err)) => match err {
                    password_hash::Error::Password => {
                        tracing::info!("mismatched password");
                        Err(WrongPassword)
                    }
                    err => {
                        tracing::error!("got error {err:?} when validating password");
                        Err(InternalError(err.to_string()))
                    }
                },
                // failed to parse stored password.
                Err(err) => {
                    tracing::error!("got error {err:?} when parsing stored password");
                    Err(HashError(ParseError(err.to_string())))
                }
            }
        }
        // the requested user doesnt exist. lets try to create a new account:
        Err(_err) => {
            tracing::info!("no existing user, creating new account");

            if request.password.len() < 8 {
                tracing::info!("password chars < 8");
                Err(InvalidPassword(NotEnoughChars))
            } else if request.password.len() > 64 {
                tracing::info!("password chars > 64");
                Err(InvalidPassword(TooManyChars))
            } else {
                // get the largest id in db, or 0 if there are no users.
                let last_id = sqlx::query_as::<_, (u32,)>("SELECT id FROM users ORDER BY id DESC")
                    .fetch_one(db)
                    .await
                    // unwrap the tuple
                    .map(|v| v.0)
                    // 0 is default
                    .unwrap_or(0);

                // we add 1 to get the next id.
                let new_user = DBUser::new(last_id + 1, req_username, &request.password);

                match new_user {
                    Ok(new_user) => {
                        // store the user in db
                        if let Err(err) = new_user.store(db).await {
                            tracing::error!("failed to store user: {err:?}");
                            Err(DBError(StoreError(err.to_string())))
                        } else {
                            // create and store the session
                            generate_store_session(db, new_user.id).await
                        }
                    }
                    Err(err) => {
                        tracing::error!("got err {err} trying to create a new user");
                        Err(HashError(err))
                    }
                }
            }
        }
    };

    tracing::debug!(
        "json response: {}",
        json::to_pretty_string(&session).unwrap()
    );
    Json(session)
}

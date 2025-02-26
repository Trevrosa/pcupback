/// Data structs regarding authorization to be shared in requests
mod structs;

use argon2::{Argon2, PasswordHash, PasswordVerifier, password_hash};
use chrono::{DateTime, TimeDelta, Utc};
use rocket::{State, post, serde::json::Json};
use rocket_db_pools::sqlx;
use serde::Serialize;
use sqlx::{Pool, Sqlite};
use structs::{
    db::{DBUser, DBUserSession, Storable},
    http::{AuthRequest, UserSession},
};
use thiserror::Error;
use tracing::{Level, span};

use crate::db::DBErrorKind;

#[derive(Error, Debug, Serialize)]
pub enum AuthenticationError {
    #[error("password was invalid")]
    InvalidPassword(#[from] InvalidPasswordKind),
    #[error("password did not match stored password")]
    WrongPassword,
    #[error("password hashing failed")]
    HashError(#[from] HashErrorKind),
    #[error("db error")]
    DBError(#[from] DBErrorKind),
    #[error("internal error occured while handling request")]
    InternalError,
    #[error("unhandled internal error occured while handling request")]
    InternalUnhandledError,
}

#[derive(Error, Debug, Serialize)]
pub enum InvalidPasswordKind {
    #[error("not enough characters (min. 8)")]
    NotEnoughChars,
    #[error("too many characters (max is 64)")]
    TooManyChars,
}

#[derive(Error, Debug, Serialize)]
pub enum HashErrorKind {
    #[error("failed to hash")]
    CreateError,
    #[error("failed to parse hash")]
    ParseError,
}

const SESSION_TIMEOUT: TimeDelta = TimeDelta::days(1);

fn session_timeout(dt: DateTime<Utc>) -> bool {
    Utc::now().time() - dt.time() > SESSION_TIMEOUT
}

#[post("/auth", data = "<request>")]
pub async fn authenticate(
    db: &State<Pool<Sqlite>>,
    request: Json<AuthRequest>,
) -> Json<Result<UserSession, AuthenticationError>> {
    use AuthenticationError::*;
    use DBErrorKind::*;
    use HashErrorKind::*;
    use InvalidPasswordKind::*;

    // we have &State<Pool<Sqlite>>.
    // deref &State<..> => State<..>
    // deref State<T> => T
    // ref T => &T
    // we end up with &Pool<Sqlite>
    let db = &**db;

    // FIXME: use the alternatives shown in macro doc below
    let my_span = span!(Level::INFO, "authenticate");
    let _enter = my_span.enter();

    // check the database for a user with the same username requested.
    // get it if it exists.
    let existing_user: Result<DBUser, _> = sqlx::query_as("SELECT * FROM users WHERE username = ?")
        .bind(request.username.trim())
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
                Ok(Ok(_)) => {
                    // lets now provide them a session id.

                    tracing::debug!("getting session from db for user {}", existing_user.id);
                    let stored_session: Result<DBUserSession, _> =
                        sqlx::query_as("SELECT last_set FROM sessions WHERE user_id = ?")
                            .bind(existing_user.id)
                            .fetch_one(db)
                            .await;

                    match stored_session {
                        // we have a stored session
                        Ok(stored_session) => {
                            // if `last_set` was more than `SESSION_TIMEOUT` ago, we create a new session.
                            let session_last_set = stored_session.last_set_datetime();

                            match session_last_set {
                                Some(session_last_set) => {
                                    if session_timeout(session_last_set) {
                                        tracing::info!("session timed out, generating new one");
                                        let new_session = DBUserSession::generate(existing_user.id);

                                        match new_session.store(db).await {
                                            // stored session successfully
                                            Ok(_) => Ok(new_session.into()),
                                            Err(err) => {
                                                tracing::error!("failed to store session: {err:?}");
                                                Err(DBError(StoreError))
                                            }
                                        }
                                    } else {
                                        // session is ok, return it
                                        Ok(stored_session.into())
                                    }
                                }
                                None => {
                                    tracing::error!("no");
                                    Err(InternalError)
                                }
                            }
                        }
                        // we do not have a stored session, generate one.
                        Err(_) => {
                            tracing::warn!("no session, generating one");
                            let new_session = DBUserSession::generate(existing_user.id);

                            match new_session.store(db).await {
                                // stored session successfully, return
                                Ok(_) => Ok(new_session.into()),
                                Err(err) => {
                                    tracing::error!("failed to store session: {err:?}");
                                    Err(DBError(StoreError))
                                }
                            }
                        }
                    }
                }
                // stored password was parsed, but didnt match.
                Ok(Err(err)) => match err {
                    password_hash::Error::Password => {
                        tracing::debug!("mismatched password");
                        Err(WrongPassword)
                    }
                    err => {
                        tracing::error!("got error {err:?} when validating password");
                        Err(InternalUnhandledError)
                    }
                },
                // failed to parse stored password.
                Err(err) => {
                    tracing::error!("got error {err:?} when parsing stored password");
                    Err(HashError(ParseError))
                }
            }
        }
        // the requested user doesnt exist. lets try to create a new account:
        Err(_err) => {
            tracing::debug!("no existing user, creating new account");

            if request.password.len() < 8 {
                Err(InvalidPassword(NotEnoughChars))
            } else if request.password.len() > 64 {
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

                let new_user =
                    DBUser::new(last_id, request.username.clone(), request.password.clone());

                match new_user {
                    Ok(new_user) => {
                        let new_session = DBUserSession::generate(new_user.id);

                        match new_session.store(db).await {
                            // stored session successfully, return
                            Ok(_) => Ok(new_session.into()),
                            Err(err) => {
                                tracing::error!("failed to store session: {err:?}");
                                Err(DBError(StoreError))
                            }
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

    Json(session)
}

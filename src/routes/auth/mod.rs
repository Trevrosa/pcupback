/// Data structs regarding authorization shared in requests
pub mod data;

#[cfg(test)]
mod tests;

use argon2::{Argon2, PasswordHash, PasswordVerifier, password_hash};
use data::{
    private::{DBUser, DBUserSession},
    public::{AuthError, AuthRequest, UserSession},
};
use rocket::{
    State, post,
    serde::json::{self, Json},
};
use sqlx::{Pool, Sqlite};
use tracing::instrument;

use pcupback::{
    DBErrorKind::{InsertError, OtherError},
    Fetchable, Storable,
};

use crate::util::{
    auth::{generate_store_session, validate_session},
    db::PoolStateExt,
};

pub type AuthResult = Result<UserSession, AuthError>;

#[instrument(skip_all)]
#[post("/auth", data = "<request>")]
pub async fn authenticate(
    state: &State<Pool<Sqlite>>,
    request: Json<AuthRequest>,
) -> Json<AuthResult> {
    use AuthError::{
        DBError, EmptyUsername, HashError, InternalError, InvalidPassword, WrongPassword,
    };
    use data::public::{
        HashErrorKind::ParseError, InvalidPasswordKind::TooFewChars,
        InvalidPasswordKind::TooManyChars,
    };
    use pcupback::DBErrorKind::SelectError;

    let db = state.to_db();

    let req_username = request.username.trim();

    if req_username.is_empty() {
        return Json(Err(EmptyUsername));
    }

    // check the database for a user with the same username requested.
    // get it if it exists.
    let existing_user: Result<DBUser, _> = DBUser::fetch_one(req_username, db).await;

    let session: AuthResult = match existing_user {
        // the user requested exists, lets check if the request password hash matches:
        Ok(existing_user) => {
            tracing::info!("got auth request for existing user {req_username}.");

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

                    validate_session(db, last_set, existing_user.id)
                        .await
                        .map_err(|err| DBError(InsertError(err.to_string())))
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
        Err(sqlx::Error::RowNotFound) => {
            tracing::info!(
                "no existing user {}, creating new account",
                request.username
            );

            if request.password.len() < 8 {
                tracing::info!("password chars < 8");
                Err(InvalidPassword(TooFewChars))
            } else if request.password.len() > 64 {
                tracing::info!("password chars > 64");
                Err(InvalidPassword(TooManyChars))
            } else {
                let transaction = db
                    .begin()
                    .await
                    .map_err(|e| DBError(OtherError(e.to_string())));

                let mut transaction = match transaction {
                    Ok(t) => t,
                    Err(err) => return Json(Err(err)),
                };

                // get the last-inserted id in db, or 0 if not found.
                let max_id = sqlx::query_as("SELECT MAX(id) FROM users")
                    .fetch_one(&mut *transaction)
                    .await
                    // unwrap the tuple
                    .map(|v: (u32,)| v.0);
                let max_id = match max_id {
                    Ok(id) => id,
                    Err(err) => {
                        if matches!(err, sqlx::Error::RowNotFound) {
                            0
                        } else {
                            tracing::error!("got err {err:?} trying to get last user id");
                            return Json(Err(DBError(OtherError(err.to_string()))));
                        }
                    }
                };

                // we add 1 to get the next id.
                let new_user = DBUser::new(max_id + 1, req_username, &request.password);

                let sesh = match new_user {
                    Ok(new_user) => {
                        // store the user in db

                        if let Err(err) = new_user.store(&mut *transaction).await {
                            tracing::error!("failed to store user: {err:?}");
                            Err(DBError(InsertError(err.to_string())))
                        } else {
                            // create and store the session
                            generate_store_session(&mut *transaction, new_user.id)
                                .await
                                .map_err(|err| DBError(InsertError(err.to_string())))
                        }
                    }
                    Err(err) => {
                        tracing::error!("got err {err} trying to create a new user");
                        Err(HashError(err))
                    }
                };

                if let Err(err) = transaction.commit().await {
                    return Json(Err(AuthError::DBError(OtherError(err.to_string()))));
                }
                sesh
            }
        }
        // an error occurred while querying database
        Err(err) => {
            tracing::error!("got err {err:?} trying to query db for user {req_username}.");
            Err(DBError(SelectError(err.to_string())))
        }
    };
    tracing::info!("created with: {:?}", session.as_ref().map(|a| a.user_id));

    tracing::debug!(
        "json response: {}",
        json::to_pretty_string(&session).unwrap()
    );
    Json(session)
}

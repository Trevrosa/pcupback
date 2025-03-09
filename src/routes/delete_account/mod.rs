#[cfg(test)]
mod tests;

use pcupback::DBErrorKind;
use rocket::{State, put, serde::json::Json};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use thiserror::Error;
use tracing::instrument;

#[derive(Debug, Error, Deserialize, Serialize)]
enum DeleteAccountError {
    #[error("InvalidSession")]
    InvalidSession,
    #[error("DBError")]
    DBError(DBErrorKind),
}

type DeleteAccountResult = Result<(), DeleteAccountError>;

#[instrument(skip_all)]
#[put("/auth/delete_account/<session_id>")]
pub async fn delete_account(
    db: &State<Pool<Sqlite>>,
    session_id: &str,
) -> Json<DeleteAccountResult> {
    use self::DeleteAccountError::{DBError, InvalidSession};
    use DBErrorKind::DeleteError;

    // TODO: move this to a lib function?
    let db = &**db;

    // TODO: this too? lib/auth
    let user_id = sqlx::query_as("SELECT user_id FROM sessions WHERE id = ?")
        .bind(session_id)
        .fetch_optional(db)
        .await
        .map(|o| o.map(|v: (u32,)| v.0));

    let Ok(Some(user_id)) = user_id else {
        // no such session.
        tracing::info!("session was invalid");
        return Json(Err(InvalidSession));
    };

    let delete_op = sqlx::query!("DELETE FROM users WHERE id = ?", user_id)
        .execute(db)
        .await
        .map_err(|err| {
            tracing::error!("got err {err:?} trying to delete user {user_id}");
            DBError(DeleteError(err.to_string()))
        });

    if let Err(err) = delete_op {
        return Json(Err(err));
    }

    tracing::info!("successfully deleted user {user_id}");

    Json(Ok(()))
}

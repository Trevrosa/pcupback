#[cfg(test)]
mod tests;

use pcupback::{DBErrorKind, Fetchable};
use rocket::{State, put, serde::json::Json};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use thiserror::Error;
use tracing::instrument;

use crate::{routes::auth::data::private::DBUserSession, util::db::PoolStateExt};

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
    state: &State<Pool<Sqlite>>,
    session_id: &str,
) -> Json<DeleteAccountResult> {
    use self::DeleteAccountError::{DBError, InvalidSession};
    use DBErrorKind::DeleteError;

    let db = state.to_db();

    let session = DBUserSession::fetch_one(session_id, db).await;

    let Ok(session) = session else {
        // no such session.
        tracing::info!("session was invalid");
        return Json(Err(InvalidSession));
    };

    let user_id = session.user_id;

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

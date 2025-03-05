pub mod data;

#[cfg(test)]
mod tests;

use data::{
    private::DBAppInfo,
    public::{SyncError, UserData},
};
use pcupback::{Fetchable, Storable};
use rocket::{State, post, serde::json::Json};
use sqlx::{Pool, Sqlite};
use tracing::instrument;

type SyncResult = Result<UserData, SyncError>;

/// We want to receive the client's state,
/// find the diff of the client state and stored state,
/// and return the final, combined state.  
#[instrument(skip_all)]
#[post("/sync/<session>", data = "<request_user_data>")]
pub async fn sync(
    db: &State<Pool<Sqlite>>,
    session: &str,
    request_user_data: Json<Option<UserData>>,
) -> Json<SyncResult> {
    use data::public::SyncError::{DBError, InvalidSession};
    use pcupback::DBErrorKind::{InsertError, SelectError};

    tracing::info!("got data fetch");

    // see src/routes/auth/mod.rs:86
    let db = &**db;

    let user_id = sqlx::query_as("SELECT user_id FROM sessions WHERE id = ?")
        .bind(session)
        .fetch_optional(db)
        .await
        .map(|o| o.map(|v: (u32,)| v.0));

    let Ok(Some(user_id)) = user_id else {
        // no such session.
        return Json(Err(InvalidSession));
    };

    // TODO: add more logging, check errors sent are descriptive enough

    let stored_app_info = DBAppInfo::fetch_all(user_id, db)
        .await
        .map_err(|e| DBError(SelectError(e.to_string())));

    let stored_app_info = match stored_app_info {
        Ok(info) => info,
        Err(err) => return Json(Err(err)),
    };

    // check for `app`s that arent in `stored_app_info`.
    let mut added = 0;
    if let Json(Some(user_data)) = request_user_data {
        for app in &user_data.app_usage {
            if stored_app_info.iter().any(|s| s.eq(app)) {
                // `stored_app_info` conatins `app`
                continue;
            }

            // `app`, from the request, was not found in the `stored_app_info`.
            let in_db = DBAppInfo::with_app_info(user_id, app.clone());
            if let Err(err) = in_db.store(db).await {
                return Json(Err(DBError(InsertError(err.to_string()))));
            }
            added += 1;
        }
    }

    // the final, combined user data.
    let stored_data = UserData::fetch_one(user_id, db)
        .await
        .map_err(|e| DBError(SelectError(e.to_string())));

    tracing::info!(
        "sync summary => incoming: {added}, outgoing: {}",
        stored_data
            .as_ref()
            .map(|d| d.app_usage.len() - added)
            .unwrap_or(0)
    );

    Json(stored_data)
}

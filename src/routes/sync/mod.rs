mod data;

use data::public::{SyncError, UserData};
use pcupback::Fetchable;
use rocket::{State, get, serde::json::Json};
use sqlx::{Pool, Sqlite};

type SyncResult = Result<UserData, SyncError>;

/// We want to receive the client's state,
/// find the diff of the client state and stored state,
/// and return the final, combined state.  
#[get("/sync/<session>", data = "<user_data>")]
pub async fn sync(
    db: &State<Pool<Sqlite>>,
    session: String,
    user_data: Json<UserData>,
) -> Json<SyncResult> {
    use data::public::SyncError::*;
    use pcupback::DBErrorKind::*;

    tracing::info!("got data fetch");

    // see src/routes/auth/mod.rs:86
    let db = &**db;

    let user_id = sqlx::query_as::<_, (u32,)>("SELECT user_id FROM sessions WHERE id = ?")
        .bind(&session)
        .fetch_optional(db)
        .await
        .map(|o| o.map(|v| v.0));

    let Ok(Some(user_id)) = user_id else {
        // no such session.
        return Json(Err(InvalidSession));
    };

    // TODO: do the diff checking and actual synchronizing here.

    let stored_data = UserData::fetch(user_id, db)
        .await
        .map_err(|e| DBError(SelectError(e.to_string())));

    Json(stored_data)
}

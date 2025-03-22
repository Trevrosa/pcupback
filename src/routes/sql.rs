use rocket::{State, post, serde::json::Json};
use sqlx::{Pool, Sqlite};
use tracing::instrument;

use crate::util::db::PoolStateExt;

pub type SqlExecResult = Json<Result<u64, String>>;

#[instrument(skip_all)]
#[post("/sql/<cmd>")]
pub async fn sql(state: &State<Pool<Sqlite>>, cmd: &str) -> SqlExecResult {
    let db = state.to_db();
    let res = sqlx::query(cmd)
        .execute(db)
        .await
        .map_err(|err| err.to_string())
        .map(|ok| ok.rows_affected());
    Json(res)
}

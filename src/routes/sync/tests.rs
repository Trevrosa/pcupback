use rocket::{http::ContentType, local::asynchronous::Client, serde::json};
use uuid::Uuid;

use crate::routes::{
    auth::{AuthResult, data::public::AuthRequest},
    sync::SyncResult,
};

use super::data::public::UserData;

// FIXME: fix rocket tests.
#[rocket::async_test]
async fn dry_sync() {
    let client = Client::tracked(crate::rocket().await).await.unwrap();

    let user = AuthRequest {
        username: Uuid::new_v4().to_string(),
        password: "12345678".to_string(),
    };

    let session = client
        .post("/auth")
        .header(ContentType::JSON)
        .body(json::to_string(&user).unwrap())
        .dispatch()
        .await
        .into_json::<AuthResult>()
        .await
        .unwrap()
        .unwrap();

    let session_id = session.id;

    let req: Option<UserData> = None;

    let resp = client
        .post(format!("/sync/{session_id}"))
        .header(ContentType::JSON)
        .body(json::to_string(&req).unwrap())
        .dispatch()
        .await
        .into_json::<SyncResult>()
        .await
        .unwrap()
        .unwrap();

    assert_eq!(resp.app_usage.len(), 0);
}

use rocket::{http::ContentType, local::blocking::Client, serde::json};
use uuid::Uuid;

use crate::routes::{
    auth::{AuthResult, data::public::AuthRequest},
    sync::SyncResult,
};

use super::data::public::UserData;

#[test]
fn dry_sync() {
    let client = Client::tracked(crate::test_rocket("dry_sync")).unwrap();

    let user = AuthRequest {
        username: Uuid::new_v4().to_string(),
        password: "12345678".to_string(),
    };

    let session = client
        .post("/auth")
        .header(ContentType::JSON)
        .body(json::to_string(&user).unwrap())
        .dispatch()
        .into_json::<AuthResult>()
        .unwrap()
        .unwrap();

    let session_id = session.id;

    let req: Option<UserData> = None;

    let resp = client
        .post(format!("/sync/{session_id}"))
        .header(ContentType::JSON)
        .body(json::to_string(&req).unwrap())
        .dispatch()
        .into_json::<SyncResult>()
        .unwrap()
        .unwrap();

    assert_eq!(resp.app_usage.len(), 0);
}

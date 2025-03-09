use rocket::{http::ContentType, serde::json};

use crate::routes::{
    auth::{AuthResult, data::public::AuthRequest},
    sync::SyncResult,
};

use super::DeleteAccountResult;

#[macros::rocket_test]
fn create_and_delete() {
    let user = AuthRequest {
        username: "xddddd".to_string(),
        password: "12345678".to_string(),
    };

    // create the user
    let create = client
        .post("/auth")
        .header(ContentType::JSON)
        .body(json::to_string(&user).unwrap())
        .dispatch()
        .into_json::<AuthResult>()
        .unwrap();
    let session = create.unwrap();
    let session_id = session.id;

    // delete that user
    let delete = client
        .put(format!("/auth/delete_account/{session_id}"))
        .dispatch()
        .into_json::<DeleteAccountResult>()
        .unwrap();
    delete.unwrap();

    // verify it's deleted
    let verify = client
        .post(format!("/sync/{session_id}"))
        .header(ContentType::JSON)
        .body("null") // None
        .dispatch()
        .into_json::<SyncResult>()
        .unwrap();
    // should fail since user doesnt exist anymore, so session doesnt exist anymore.
    verify.unwrap_err();
}

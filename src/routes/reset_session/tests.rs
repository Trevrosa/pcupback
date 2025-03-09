use rocket::http::ContentType;

use crate::routes::{
    auth::{AuthResult, data::public::AuthRequest},
    sync::SyncResult,
};

use super::ResetSessionResult;

#[macros::rocket_test]
fn reset_session() {
    let user = AuthRequest {
        username: "ppkxddddd".to_string(),
        password: "12345678".to_string(),
    };

    // create the user
    let create = client
        .post("/auth")
        .json(&user)
        .dispatch()
        .into_json::<AuthResult>()
        .unwrap();
    let orig_session = create.unwrap();
    let orig_session_id = orig_session.id;

    let reset = client
        .put(format!("/auth/reset_session/{orig_session_id}"))
        .dispatch()
        .into_json::<ResetSessionResult>()
        .unwrap();
    let new_session = reset.unwrap();
    let new_session_id = new_session.id;

    let test_fail = client
        .post(format!("/sync/{orig_session_id}"))
        .header(ContentType::JSON)
        .body("null")
        .dispatch()
        .into_json::<SyncResult>()
        .unwrap();
    // orig session was invalidated.
    test_fail.unwrap_err();

    let test_ok = client
        .post(format!("/sync/{new_session_id}"))
        .header(ContentType::JSON)
        .body("null")
        .dispatch()
        .into_json::<SyncResult>()
        .unwrap();
    test_ok.unwrap();
}

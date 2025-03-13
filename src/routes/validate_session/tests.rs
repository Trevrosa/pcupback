use crate::routes::auth::{AuthResult, data::public::AuthRequest};

#[macros::rocket_test]
fn validate_session() {
    // test the false
    let expect_invalid = client
        .get("/auth/validate_session/xdd")
        .dispatch()
        .into_json::<bool>();
    assert_eq!(expect_invalid, Some(false));

    // test the true
    let user = AuthRequest::random_valid();

    // get the session
    let session = client
        .post("/auth")
        .json(&user)
        .dispatch()
        .into_json::<AuthResult>()
        .unwrap();
    let session_id = session.unwrap().id;

    let expect_valid = client
        .get(format!("/auth/validate_session/{session_id}"))
        .dispatch()
        .into_json::<bool>();
    assert_eq!(expect_valid, Some(true))
}

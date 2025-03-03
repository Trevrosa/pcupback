use rocket::{
    http::{ContentType, Status},
    local::asynchronous::Client,
    serde::json,
};
use uuid::Uuid;

use super::{
    AuthResult,
    data::public::{AuthError, AuthRequest},
};

#[rocket::async_test]
async fn not_enough_chars() {
    use super::data::public::InvalidPasswordKind::TooFewChars;

    let client = Client::tracked(crate::rocket().await).await.unwrap();

    let req = AuthRequest {
        username: Uuid::new_v4().to_string(),
        password: "123".to_string(),
    };

    let resp = client
        .post("/auth")
        .header(ContentType::JSON)
        .body(json::to_string(&req).unwrap())
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Ok);
    let invalid_resp_json: AuthResult = resp.into_json().await.unwrap();
    assert!(matches!(
        invalid_resp_json.unwrap_err(),
        AuthError::InvalidPassword(TooFewChars)
    ));
}

#[rocket::async_test]
async fn too_many_chars() {
    use super::data::public::InvalidPasswordKind::TooManyChars;

    let client = Client::tracked(crate::rocket().await).await.unwrap();

    let req = AuthRequest {
        username: Uuid::new_v4().to_string(),
        password: "1".repeat(65),
    };

    let resp = client
        .post("/auth")
        .header(ContentType::JSON)
        .body(json::to_string(&req).unwrap())
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Ok);
    let resp_json: AuthResult = resp.into_json().await.unwrap();
    assert!(matches!(
        resp_json.unwrap_err(),
        AuthError::InvalidPassword(TooManyChars)
    ));
}

// FIXME: fix rocket tests with multiple requests
#[rocket::async_test]
async fn login() {
    let client = Client::tracked(crate::rocket().await).await.unwrap();

    let req = AuthRequest {
        username: Uuid::new_v4().to_string(),
        password: "12345678".to_string(),
    };

    let resp1 = client
        .post("/auth")
        .header(ContentType::JSON)
        .body(json::to_string(&req).unwrap())
        .dispatch()
        .await
        .into_json::<AuthResult>()
        .await
        .unwrap();

    let session1 = resp1.unwrap();

    let resp2 = client
        .post("/auth")
        .header(ContentType::JSON)
        .body(json::to_string(&req).unwrap())
        .dispatch()
        .await
        .into_json::<AuthResult>()
        .await
        .unwrap();

    let session2 = resp2.unwrap();

    assert_eq!(session1, session2);
}

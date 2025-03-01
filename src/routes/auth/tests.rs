use rocket::{
    http::{ContentType, Status},
    local::asynchronous::Client,
    serde::json,
};

use crate::routes::auth::AuthRequest;

use super::{AuthenticationError, data::public::UserSession};

type AuthResult = Result<UserSession, AuthenticationError>;

#[rocket::async_test]
async fn not_enough_chars() {
    use crate::routes::auth::InvalidPasswordKind::NotEnoughChars;

    let client = Client::tracked(crate::rocket().await).await.unwrap();

    let req = AuthRequest {
        username: "xdd".to_string(),
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
        AuthenticationError::InvalidPassword(NotEnoughChars)
    ));
}

#[rocket::async_test]
async fn too_many_chars() {
    use crate::routes::auth::InvalidPasswordKind::TooManyChars;

    let client = Client::tracked(crate::rocket().await).await.unwrap();

    let req = AuthRequest {
        username: "xdd".to_string(),
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
        AuthenticationError::InvalidPassword(TooManyChars)
    ));
}

#[rocket::async_test]
async fn login() {
    let client = Client::tracked(crate::rocket().await).await.unwrap();

    let req = AuthRequest {
        username: "xdd".to_string(),
        password: "1".repeat(8),
    };

    let resp = client
        .post("/auth")
        .header(ContentType::JSON)
        .body(json::to_string(&req).unwrap())
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Ok);
    let resp_json: AuthResult = resp.into_json().await.unwrap();

    let first_session = resp_json.unwrap().id;

    let resp = client
        .post("/auth")
        .header(ContentType::JSON)
        .body(json::to_string(&req).unwrap())
        .dispatch()
        .await;

    assert_eq!(resp.status(), Status::Ok);

    let resp2_json: AuthResult = resp.into_json().await.unwrap();

    let second_session = resp2_json.unwrap().id;

    assert_eq!(first_session, second_session);
}

use rocket::{http::ContentType, serde::json};
use uuid::Uuid;

use crate::routes::{
    auth::{AuthResult, data::public::AuthRequest},
    sync::SyncResult,
};

use super::data::public::{AppInfo, UserData};

#[macros::rocket_test]
fn dry_sync() {
    let user = AuthRequest::random_valid();

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

    assert_eq!(resp.data.app_usage.len(), 0);
}

#[macros::rocket_test]
fn sync_store() {
    let user = AuthRequest::random_valid();

    let session = client
        .post("/auth")
        .header(ContentType::JSON)
        .body(json::to_string(&user).unwrap())
        .dispatch()
        .into_json::<AuthResult>()
        .unwrap();
    let session = session.unwrap();
    let session_id = &session.id;

    let my_data = UserData {
        app_usage: vec![AppInfo::new("io1", 2, 0), AppInfo::new("io2", 10, 10)],
        debug: vec![],
    };

    let store = client
        .post(format!("/sync/{session_id}"))
        .json(&Some(&my_data))
        .dispatch()
        .into_json::<SyncResult>()
        .unwrap();
    let stored = store.unwrap();

    assert_eq!(my_data, stored.data);
}

#[macros::rocket_test]
fn sync_multi_client() {
    let user = AuthRequest {
        username: Uuid::new_v4().to_string(),
        password: "12345678".to_string(),
    };

    let session = client
        .post("/auth")
        .json(&user)
        .dispatch()
        .into_json::<AuthResult>()
        .unwrap();
    let session = session.unwrap();
    let session_id = &session.id;

    let my_data = UserData {
        app_usage: vec![AppInfo::new("io1", 2, 0), AppInfo::new("io2", 10, 10)],
        debug: vec![],
    };

    let url = format!("/sync/{session_id}");

    // this client has some data to store
    let first_client = client
        .post(&url)
        .json(&Some(&my_data))
        .dispatch()
        .into_json::<SyncResult>()
        .unwrap();
    let first_client_sync = first_client.unwrap();

    // this client has no data
    let another_client = client
        .post(&url)
        .header(ContentType::JSON)
        .body("null") // None in json
        .dispatch()
        .into_json::<SyncResult>()
        .unwrap();
    let another_client_data = another_client.unwrap();

    assert_eq!(&my_data, &first_client_sync.data);
    assert_eq!(first_client_sync, another_client_data);
}

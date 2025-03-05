use sqlx::{Pool, Sqlite};

#[sqlx::test]
async fn expect_fail(db: Pool<Sqlite>) {
    // foreign key not exist
    sqlx::query!(
        "INSERT INTO app_info(user_id, app_name, app_usage, app_limit) VALUES(1, 'xdd', 1, 1)"
    )
    .execute(&db)
    .await
    .unwrap();

    // null values
    sqlx::query!("INSERT INTO app_info(app_name, app_usage, app_limit) VALUES('xdd', NULL, 1)")
        .execute(&db)
        .await
        .unwrap();
}

#[sqlx::test]
async fn expect_success(db: Pool<Sqlite>) {
    // create user to then insert app_info
    let user_id = 1;
    sqlx::query!(
        "INSERT INTO users(id, username, password_hash) VALUES(?, 'test', 'xd')",
        user_id
    )
    .execute(&db)
    .await
    .unwrap();

    // insert app info
    sqlx::query!(
        "INSERT INTO app_info(user_id, app_name, app_usage, app_limit) VALUES(?, 'xdd', 1, 1)",
        user_id
    )
    .execute(&db)
    .await
    .unwrap();
    // insert another of the same name
    sqlx::query!(
        "INSERT INTO app_info(user_id, app_name, app_usage, app_limit) VALUES(?, 'xdd', 1, 1)",
        user_id
    )
    .execute(&db)
    .await
    .unwrap();
}

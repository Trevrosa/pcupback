use sqlx::{Pool, Sqlite};

#[sqlx::test]
async fn expect_fail(db: Pool<Sqlite>) {
    // duplicate username
    let duplicate_username = "xdd";
    // first, ok
    sqlx::query!(
        "INSERT INTO users(id, username, password_hash) VALUES(1, ?, 'xdd')",
        duplicate_username
    )
    .execute(&db)
    .await
    .unwrap();

    // second, not ok
    sqlx::query!(
        "INSERT INTO users(id, username, password_hash) VALUES(2, ?, 'xdd')",
        duplicate_username
    )
    .execute(&db)
    .await
    .unwrap_err();

    // duplicate user id
    sqlx::query!("INSERT INTO users(id, username, password_hash) VALUES(1, '123', 'xdd')",)
        .execute(&db)
        .await
        .unwrap_err();

    // null values
    sqlx::query!("INSERT INTO users(id) VALUES(1)")
        .execute(&db)
        .await
        .unwrap_err();
    sqlx::query!("INSERT INTO users(username) VALUES('po3')")
        .execute(&db)
        .await
        .unwrap_err();
}

#[sqlx::test]
async fn expect_success(db: Pool<Sqlite>) {
    sqlx::query!("INSERT INTO users(id, username, password_hash) VALUES(1, 'hi1', 'xd')")
        .execute(&db)
        .await
        .unwrap();
}

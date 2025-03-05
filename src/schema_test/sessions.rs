use sqlx::{Pool, Sqlite};

#[sqlx::test]
async fn expect_fail(db: Pool<Sqlite>) {
    // no such user id, foreign key fail.
    sqlx::query!("INSERT INTO sessions(user_id, id, last_set) VALUES(1, 'xd', 1)")
        .execute(&db)
        .await
        .unwrap_err();

    // null user id
    sqlx::query!("INSERT INTO sessions(id, last_set) VALUES('xd', 1)")
        .execute(&db)
        .await
        .unwrap_err();
    sqlx::query!("INSERT INTO sessions(user_id, id, last_set) VALUES(NULL, 'xd', 1)")
        .execute(&db)
        .await
        .unwrap_err();

    // null values
    sqlx::query!("INSERT INTO sessions(user_id, id, last_set) VALUES(NULL, NULL, NULL)")
        .execute(&db)
        .await
        .unwrap_err();
}

#[sqlx::test]
async fn expect_success(db: Pool<Sqlite>) {
    // create user to then insert session
    let user_id = 1;
    sqlx::query!(
        "INSERT INTO users(id, username, password_hash) VALUES(?, 'test', 'xd')",
        user_id
    )
    .execute(&db)
    .await
    .unwrap();

    // insert the session for `user_id`
    sqlx::query!(
        "INSERT INTO sessions(user_id, id, last_set) VALUES(?, 'test', 0)",
        user_id
    )
    .execute(&db)
    .await
    .unwrap();
}

use sqlx::{Pool, Sqlite};

#[sqlx::test]
async fn fail_user_session(db: Pool<Sqlite>) {
    let no_such_user_id =
        sqlx::query!("INSERT INTO sessions(user_id, id, last_set) VALUES(1, 'xd', 1)")
            .execute(&db)
            .await;
    assert!(no_such_user_id.is_err());

    let null_user_id = sqlx::query!("INSERT INTO sessions(id, last_set) VALUES('xd', 1)")
        .execute(&db)
        .await;
    assert!(null_user_id.is_err());

    let null_values =
        sqlx::query!("INSERT INTO sessions(user_id, id, last_set) VALUES(NULL, NULL, NULL)")
            .execute(&db)
            .await;
    assert!(null_values.is_err())
}

#[sqlx::test]
async fn success_user_session(db: Pool<Sqlite>) {
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

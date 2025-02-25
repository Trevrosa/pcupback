mod auth;
mod db;

use std::sync::LazyLock;

use argon2::{Algorithm::Argon2id, Argon2, Params, Version::V0x13};
use auth::authenticate;
use rocket::{get, launch, routes};
use rocket_db_pools::Database;
use sqlx::{SqlitePool, migrate, sqlite::SqlitePoolOptions};

#[get("/")]
fn index<'a>() -> &'a str {
    "Hello, World!"
}

static ARGON2: LazyLock<Argon2<'static>> = LazyLock::new(|| {
    Argon2::new(
        Argon2id,
        V0x13, // version 19
        Params::new(19 * 1024, 2, 1, None).expect("params shld be valid"),
    )
});

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let db_pool = SqlitePool::connect("sqlite://db.sqlite")
        .await
        .expect("could not open db");

    // do db migrations. (./migrations dir)
    let migrator = migrate!();
    migrator
        .run(&db_pool)
        .await
        .expect("could not run migrations");

    let _result = rocket::build()
        .manage(db_pool)
        .mount("/", routes![index, authenticate])
        .launch()
        .await;

    Ok(())
}

mod db;
mod routes;

use rocket::{Build, Rocket, get, routes};
use routes::auth::authenticate;
use sqlx::{SqlitePool, migrate, sqlite::SqliteConnectOptions};
use tracing::level_filters::LevelFilter;

#[get("/")]
fn index<'a>() -> &'a str {
    "Hello, World!"
}

#[cfg(not(debug_assertions))]
const LOG_LEVEL: LevelFilter = LevelFilter::INFO;
#[cfg(debug_assertions)]
const LOG_LEVEL: LevelFilter = LevelFilter::DEBUG;

#[cfg(not(test))]
const DB_PATH: &str = "xdd.db";
#[cfg(test)]
const DB_PATH: &str = "test.db";

async fn rocket() -> Rocket<Build> {
    let db_options = SqliteConnectOptions::new()
        .filename(DB_PATH)
        .create_if_missing(true);
    let db_pool = SqlitePool::connect_with(db_options)
        .await
        .expect("could not open db");

    // do db migrations. (./migrations dir)
    let migrator = migrate!();
    migrator
        .run(&db_pool)
        .await
        .expect("could not run migrations");

    rocket::build()
        .manage(db_pool)
        .mount("/", routes![index, authenticate])
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    // FIXME: see if we can change the ugly "rocket::launch::_:".
    tracing_subscriber::fmt().with_max_level(LOG_LEVEL).init();
    rocket().await.launch().await.unwrap();

    Ok(())
}

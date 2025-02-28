mod auth;
mod db;
#[cfg(test)]
mod tests;

use auth::authenticate;
use rocket::{Ignite, Rocket, get, routes};
use sqlx::{Pool, Sqlite, SqlitePool, migrate};
use tracing::level_filters::LevelFilter;

#[get("/")]
fn index<'a>() -> &'a str {
    "Hello, World!"
}

#[cfg(debug_assertions)]
const LOG_LEVEL: LevelFilter = LevelFilter::DEBUG;
#[cfg(not(debug_assertions))]
const LOG_LEVEL: LevelFilter = LevelFilter::INFO;

async fn launch(db_pool: Pool<Sqlite>) -> Rocket<Ignite> {
    rocket::build()
        .manage(db_pool)
        .mount("/", routes![index, authenticate])
        .launch()
        .await
        .unwrap()
}

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

    tracing_subscriber::fmt().with_max_level(LOG_LEVEL).init();

    launch(db_pool).await;

    Ok(())
}

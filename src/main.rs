mod routes;

use rocket::{Build, Rocket, get, routes};
use routes::{auth::authenticate, sync::sync};
use sqlx::{Pool, Sqlite, migrate, pool::PoolOptions, sqlite::SqliteConnectOptions};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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
    let db_pool: Pool<Sqlite> = PoolOptions::new()
        .min_connections(1)
        .max_connections(5)
        .connect_with(db_options)
        .await
        .expect("could not open db");

    // do db migrations. (from the `./migrations` dir)
    tracing::debug!("running migrations..");
    let migrator = migrate!();
    migrator
        .run(&db_pool)
        .await
        .expect("could not run migrations");

    rocket::build()
        .manage(db_pool)
        .mount("/", routes![index, authenticate, sync])
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    // TODO: see if we can change the ugly "rocket::launch::_:".
    if let Ok(journald) = tracing_journald::layer() {
        println!("activated journald tracing layer");
        tracing_subscriber::registry().with(journald).init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(LOG_LEVEL)
            .compact()
            .init();
    }

    rocket().await.launch().await.unwrap();

    Ok(())
}

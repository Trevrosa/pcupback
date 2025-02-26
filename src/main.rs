mod auth;
mod db;

use auth::authenticate;
use rocket::{get, routes};
use sqlx::{SqlitePool, migrate};

#[get("/")]
fn index<'a>() -> &'a str {
    "Hello, World!"
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

    let _result = rocket::build()
        .manage(db_pool)
        .mount("/", routes![index, authenticate])
        .launch()
        .await;

    Ok(())
}

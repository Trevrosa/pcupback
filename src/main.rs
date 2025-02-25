mod auth;
mod db;

use db::Db;
use rocket::{get, launch, routes};
use rocket_db_pools::{Database, sqlx};

#[get("/")]
fn index<'a>() -> &'a str {
    "Hello, World!"
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Db::init())
        .mount("/", routes![index])
}

mod auth;
mod db;

use std::sync::LazyLock;

use argon2::{Algorithm::Argon2id, Argon2, Params, Version::V0x13};
use db::Db;
use rocket::{get, launch, routes};
use rocket_db_pools::Database;

#[get("/")]
fn index<'a>() -> &'a str {
    "Hello, World!"
}

static ARGON2: LazyLock<Argon2<'static>> = LazyLock::new(|| {
    Argon2::new(
        Argon2id,
        V0x13,
        Params::new(19 * 1024, 2, 1, None).expect("params shld be valid"),
    )
});

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Db::init())
        .mount("/", routes![index])
}

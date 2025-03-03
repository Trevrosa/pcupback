mod routes;

use rocket::{Build, Rocket, fairing::AdHoc, get, routes};
use routes::{auth::authenticate, sync::sync};
use sqlx::{Pool, Sqlite, migrate, pool::PoolOptions, sqlite::SqliteConnectOptions};
use tracing_subscriber::{
    Registry,
    fmt::{
        self,
        format::{Compact, DefaultFields, Format},
    },
    layer::SubscriberExt,
    reload,
    util::SubscriberInitExt,
};

#[get("/")]
fn index<'a>() -> &'a str {
    "Hello, World!"
}

// #[cfg(not(debug_assertions))]
// const LOG_LEVEL: LevelFilter = LevelFilter::INFO;
// #[cfg(debug_assertions)]
// const LOG_LEVEL: LevelFilter = LevelFilter::DEBUG;

#[cfg(not(test))]
const DB_PATH: &str = "xdd.db";
#[cfg(test)]
const DB_PATH: &str = "test.db";

/// No fmt reload handle.
#[allow(unused)]
fn test_rocket() -> impl Future<Output = Rocket<Build>> {
    crate::rocket(None)
}

async fn rocket(fmt_reload: Option<&CompactFmtLayerHandle>) -> Rocket<Build> {
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

    // TODO: change this to hide the "rocket::launch::_:"
    if let Some(fmt_reload) = fmt_reload {
        fmt_reload
            .modify(|fmt| *fmt = fmt::layer().with_level(false).compact())
            .unwrap();
    }

    rocket::build()
        .manage(db_pool)
        .mount("/", routes![index, authenticate, sync])
}

type CompactFmtLayer = fmt::Layer<Registry, DefaultFields, Format<Compact>>;
type CompactFmtLayerHandle = reload::Handle<CompactFmtLayer, Registry>;
type CompactFmtLayerReload = reload::Layer<CompactFmtLayer, Registry>;

fn set_tracing(fmt: CompactFmtLayerReload) {
    if let Ok(journald) = tracing_journald::layer() {
        println!("activated journald tracing layer");
        tracing_subscriber::registry().with(journald).init();
    } else {
        tracing_subscriber::registry().with(fmt).init();
    }
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    // we initally have compact logs
    let fmt_default = || fmt::layer().compact();
    let (fmt, fmt_reload) = reload::Layer::new(fmt_default());
    set_tracing(fmt);

    // the rocket function then modifies the fmt
    rocket(Some(&fmt_reload))
        .await
        .attach(AdHoc::on_liftoff("Tracing Subscriber", move |_| {
            Box::pin(async move {
                // we now change it back
                fmt_reload.modify(|fmt| *fmt = fmt_default()).unwrap();
            })
        }))
        .launch()
        .await
        .unwrap();

    Ok(())
}

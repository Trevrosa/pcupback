mod routes;

use rocket::{Build, Rocket, fairing::AdHoc, get, routes};
use routes::{auth::authenticate, sync::sync};
use sqlx::{Pool, Sqlite, migrate, pool::PoolOptions, sqlite::SqliteConnectOptions};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
    Layer, Registry,
    fmt::{
        self,
        format::{Compact, DefaultFields, Format},
    },
    layer::{self, SubscriberExt},
    reload,
    util::SubscriberInitExt,
};

#[get("/")]
fn index<'a>() -> &'a str {
    "Hello, World!"
}

/// The max log level.
#[cfg(not(debug_assertions))]
const LOG_LEVEL: LevelFilter = LevelFilter::INFO;
/// The max log level.
#[cfg(debug_assertions)]
const LOG_LEVEL: LevelFilter = LevelFilter::DEBUG;

/// The path to database.
#[cfg(not(test))]
const DB_PATH: &str = "xdd.db";
/// The path to database.
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

type CompactFmtLayer = fmt::Layer<Registry, DefaultFields, Format<Compact>>;

/// Set the global logger. journald if available, else `fmt`.
fn set_tracing(fmt: reload::Layer<CompactFmtLayer, Registry>) {
    /// Filters `log level` by [`crate::LOG_LEVEL`]
    struct Filter;
    impl layer::Filter<Registry> for Filter {
        fn enabled(
            &self,
            meta: &tracing::Metadata<'_>,
            _ctx: &layer::Context<'_, Registry>,
        ) -> bool {
            meta.level() <= &LOG_LEVEL
        }
    }

    if let Ok(journald) = tracing_journald::layer() {
        println!("activated journald tracing layer");
        tracing_subscriber::registry()
            .with(journald.with_filter(Filter))
            .init();
    } else {
        // TODO: try tokio-cconsole subscriber on debug.
        tracing_subscriber::registry()
            .with(fmt.with_filter(Filter))
            .init();
    }
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    // this is our default fmt.
    let fmt_default = || fmt::layer().compact();
    let (fmt, fmt_reload) = reload::Layer::new(fmt_default());
    set_tracing(fmt);

    let rocket = rocket().await;

    // we then hide log `target`s for the initial launch messages.
    fmt_reload
        .modify(|fmt| *fmt = fmt::layer().with_target(false).compact())
        .unwrap();

    rocket
        .attach(AdHoc::on_liftoff("Tracing", move |_| {
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

mod routes;
/// Test the database schema.
#[cfg(test)]
mod schema_test;

use console_subscriber::Server;
use rocket::{Build, Rocket, get, routes};
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
fn index() -> &'static str {
    "Hello, World!"
}

/// The max log level.
const LOG_LEVEL: LevelFilter = if cfg!(debug_assertions) {
    LevelFilter::DEBUG
} else {
    LevelFilter::INFO
};

/// The database path.
const DB_PATH: &str = if cfg!(debug_assertions) {
    // debug db
    "debug.db"
} else {
    "xdd.db"
};

// TODO: create a proc attr macro for my own tests, to harness test_rocket() with the function name.

/// Create a [`Pool<Sqlite>`] with an optional `name`.
///
/// if `name` is [`None`], we use the [`DB_PATH`] const.
///
/// # Panics
///
/// Will panic if db cannot be opened.
async fn get_db_pool(name: Option<&str>) -> Pool<Sqlite> {
    // create the test db dir if we are testing
    #[cfg(test)]
    let _ = std::fs::create_dir("test_dbs");

    let db_options = SqliteConnectOptions::new()
        .filename(name.unwrap_or(DB_PATH))
        .create_if_missing(true);

    PoolOptions::new()
        .min_connections(1)
        .max_connections(5)
        .connect_with(db_options)
        .await
        .expect("could not open db")
}

/// Test a Rocket!
///
/// `name` is the test's name.
#[cfg(test)]
pub(crate) fn test_rocket(name: &str) -> Rocket<Build> {
    let mut new_rocket = None;
    rocket::tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            new_rocket = Some(rocket(Some(&format!("test_dbs/{name}.db"))).await);
        });
    new_rocket.expect("no rocket built")
}

/// Build a Rocket!
///
/// if `db_name` is [`None`], we use the [`DB_PATH`] const.
async fn rocket(db_name: Option<&str>) -> Rocket<Build> {
    let db_pool = get_db_pool(db_name).await;
    tracing::debug!("created db pool");

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

/// this is our default fmt.
#[inline]
fn fmt_default<T>() -> fmt::Layer<T, DefaultFields, Format<Compact>> {
    fmt::layer().compact()
}

/// Filters `log level` by [`crate::LOG_LEVEL`]
struct Filter;

impl layer::Filter<Registry> for Filter {
    fn enabled(&self, meta: &tracing::Metadata<'_>, _ctx: &layer::Context<'_, Registry>) -> bool {
        meta.level() <= &LOG_LEVEL
    }
}

type CompactFmtLayer<T> = fmt::Layer<T, DefaultFields, Format<Compact>>;
type ReloadCompactFmtLayer<T> = reload::Handle<CompactFmtLayer<T>, T>;

/// Initialize tracing layers.
///
/// Returns a reload handle to the [`fmt_default`] created.
fn init_loggers() -> ReloadCompactFmtLayer<Registry> {
    // try to create the journald
    let journald = tracing_journald::layer();

    // we use `console_subscriber` on debug build, or on feature tokio-console.
    if cfg!(debug_assertions) || cfg!(feature = "tokio-console") {
        // wrap the fmt in a reload::Layer
        let (fmt, reload) = reload::Layer::new(fmt_default());
        if let Ok(journald) = journald {
            tracing_subscriber::registry()
                .with(fmt.with_filter(Filter))
                // enable debugging with tokio-console
                .with(console_subscriber::spawn())
                // enable journald subscriber
                .with(journald)
                .init();
            tracing::debug!("init'd journald-subscriber layer");
        } else {
            tracing_subscriber::registry()
                .with(fmt.with_filter(Filter))
                // enable debugging with tokio-console
                .with(console_subscriber::spawn())
                .init();
        }
        tracing::info!(
            "init'd tokio-console rpc server at {}:{}",
            Server::DEFAULT_IP,
            Server::DEFAULT_PORT
        );
        reload
    } else {
        // wrap the fmt in a reload::Layer
        let (fmt, reload) = reload::Layer::new(fmt_default());
        if let Ok(journald) = journald {
            tracing_subscriber::registry()
                .with(fmt.with_filter(Filter))
                .with(journald)
                .init();
            tracing::debug!("init'd journald-subscriber layer");
        } else {
            tracing_subscriber::registry()
                .with(fmt.with_filter(Filter))
                .init();
        }
        reload
    }
}

// TODO: add 422 catcher
#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    init_loggers();

    rocket(None).await.launch().await.unwrap();

    Ok(())
}

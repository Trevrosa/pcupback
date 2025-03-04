mod routes;
/// Test the database schema.
#[cfg(test)]
mod schema_test;

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

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let fmt_reload = if let Ok(journald) = tracing_journald::layer() {
        tracing_subscriber::registry()
            .with(journald.with_filter(Filter))
            .init();
        println!("activated tracing-journald layer");
        None
    } else {
        // on debug build, or on feature tokio-console.
        if cfg!(debug_assertions) || cfg!(feature = "tokio-console") {
            // wrap the fmt in a reload::Layer
            let (fmt, reload) = reload::Layer::new(fmt_default());
            tracing_subscriber::registry()
                .with(fmt.with_filter(Filter))
                // enable debugging with tokio-console
                .with(console_subscriber::spawn())
                .init();
            tracing::debug!("init'd console-subscriber layer");
            Some(reload)
        } else {
            // wrap the fmt in a reload::Layer
            let (fmt, reload) = reload::Layer::new(fmt_default());
            tracing_subscriber::registry()
                .with(fmt.with_filter(Filter))
                .init();
            Some(reload)
        }
    };

    let rocket = rocket().await;

    if let Some(ref reload) = fmt_reload {
        // we then hide log `target`s for the initial launch messages.
        let _ = reload.modify(|fmt| *fmt = fmt::layer().with_target(false).compact());
    } else {
        tracing::debug!("no fmt layer reload handle");
    }

    rocket
        .attach(AdHoc::on_liftoff("Tracing", move |_| {
            Box::pin(async move {
                // we now change it back
                if let Some(reload) = fmt_reload {
                    let _ = reload.modify(|fmt| *fmt = fmt_default());
                } else {
                    tracing::debug!("failed to access fmt reload handle");
                }
            })
        }))
        .launch()
        .await
        .unwrap();

    Ok(())
}

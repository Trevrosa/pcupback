# list available recipes
default:
    just -l

# remove debug databases and create them from migration files.
refresh-dbs:
    rm debug.db
    rm -rf test_dbs
    sqlx database create
    sqlx migrate run
rm debug.db
rm -rf test_dbs
sqlx database create
sqlx migrate run

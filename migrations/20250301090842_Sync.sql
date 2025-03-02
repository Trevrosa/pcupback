-- `usage` will be stored as seconds
-- `app_limit` will be stored as seconds
CREATE TABLE app_info (
    user_id INTEGER NOT NULL,
    app_name TEXT NOT NULL,
    app_usage INTEGER NOT NULL,
    app_limit INTEGER NOT NULL
);

-- use user_id as the index to the `app_usage` table.
CREATE INDEX app_info_idx ON app_info (user_id)
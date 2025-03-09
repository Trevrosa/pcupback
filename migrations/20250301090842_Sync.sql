CREATE TABLE app_info (
    user_id INTEGER NOT NULL,
    app_name TEXT NOT NULL,
    -- stored as seconds
    app_usage INTEGER NOT NULL,
    -- stored as seconds
    app_limit INTEGER NOT NULL,
    -- disallow non-existent user ids.
    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- use user_id as the index to the `app_usage` table.
CREATE INDEX app_info_idx ON app_info (user_id)
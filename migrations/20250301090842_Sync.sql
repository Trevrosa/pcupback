-- `usage` will be stored as seconds
-- `app_limit` will be stored as seconds
CREATE TABLE app_info (user_id INTEGER, app_name TEXT, app_usage INTEGER, app_limit INTEGER);
-- use user_id as the index to the `app_usage` table.
CREATE INDEX app_info_idx ON app_info (user_id)

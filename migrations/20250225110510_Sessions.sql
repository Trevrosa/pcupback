CREATE TABLE sessions (
    user_id INTEGER PRIMARY KEY NOT NULL,
    id TEXT NOT NULL,
    -- stored as seconds after the unix epoch. 
    last_set INTEGER NOT NULL,
    -- enforce that `user_id` must exist in `users` as `id`
    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
);
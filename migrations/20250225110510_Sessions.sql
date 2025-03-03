-- last_set will be stored as seconds after the unix epoch. 
CREATE TABLE sessions (
    user_id INTEGER UNIQUE NOT NULL,
    id TEXT NOT NULL,
    last_set INTEGER NOT NULL,
    -- enforce that `user_id` must exist in `users` as `id`
    FOREIGN KEY(user_id) REFERENCES users(id)
);
CREATE INDEX sessions_idx ON sessions(user_id)
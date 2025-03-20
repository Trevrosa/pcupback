CREATE TABLE user_debug (
    user_id INTEGER NOT NULL,
    stored TEXT NOT NULL,
    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX user_debug_idx ON user_debug (user_id)
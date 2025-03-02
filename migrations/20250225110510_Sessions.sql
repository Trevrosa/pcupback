-- last_set will be stored as seconds after the unix epoch. 
CREATE TABLE sessions (
    user_id INTEGER PRIMARY KEY,
    id TEXT NOT NULL,
    last_set INTEGER NOT NULL
)
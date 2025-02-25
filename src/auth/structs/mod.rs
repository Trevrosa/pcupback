/// Structs used specifically only used in databases. They implement `FromRow`.
mod db;

/// Structs used more generally. They implement `Serialize` and `Deserialize`.
mod http;

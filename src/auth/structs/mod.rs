/// Structs used specifically only used in databases. They implement `FromRow`.
pub mod db;

/// Structs used more generally. They implement `Serialize` and `Deserialize`.
pub mod http;

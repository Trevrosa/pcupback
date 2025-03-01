/// Structs used only used in databases. They implement `FromRow`.
pub mod private;

/// Structs used by api consumers. They implement `Serialize` and `Deserialize`.
pub mod public;

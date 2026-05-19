//! Luau bridge scaffold for Kinetik.

/// Luau runtime configuration placeholder.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LuauRuntimeConfig {
    /// Whether editor-only APIs are available.
    pub editor_apis_enabled: bool,
}

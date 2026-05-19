//! Resource handles and asset path contracts for Kinetik.

use kinetik_core::ResourceId;

/// Logical project asset path, such as `res://assets/models/tree.glb`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetPath(String);

impl AssetPath {
    /// Creates an asset path after minimal validation.
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    /// Returns the asset path as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Typed resource handle.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ResourceHandle {
    id: ResourceId,
}

impl ResourceHandle {
    /// Creates a new resource handle.
    #[must_use]
    pub const fn new(id: ResourceId) -> Self {
        Self { id }
    }

    /// Returns the underlying resource ID.
    #[must_use]
    pub const fn id(self) -> ResourceId {
        self.id
    }
}

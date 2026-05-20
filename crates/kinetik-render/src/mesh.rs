use kinetik_core::Vec3;

/// Built-in primitive mesh identifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrimitiveMeshKind {
    /// Unit cube centered at the origin.
    Cube,
}

/// Built-in primitive mesh resource metadata.
#[derive(Clone, Debug, PartialEq)]
pub struct PrimitiveMesh {
    /// Primitive mesh kind.
    pub kind: PrimitiveMeshKind,
    /// Stable engine resource path.
    pub resource_path: &'static str,
    /// Object-space minimum bounds.
    pub min: Vec3,
    /// Object-space maximum bounds.
    pub max: Vec3,
}

impl PrimitiveMesh {
    /// Returns the built-in cube primitive mesh.
    #[must_use]
    pub const fn cube() -> Self {
        Self {
            kind: PrimitiveMeshKind::Cube,
            resource_path: "builtin://mesh/cube",
            min: Vec3::new(-0.5, -0.5, -0.5),
            max: Vec3::new(0.5, 0.5, 0.5),
        }
    }
}

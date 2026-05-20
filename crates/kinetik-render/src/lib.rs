//! Renderer contracts for Kinetik.

mod diagnostics;
mod extraction;
mod material;
mod mesh;
mod smoke;

pub use diagnostics::{
    render_diagnostic, MISSING_CAMERA_CODE, MISSING_LIGHT_CODE, MISSING_MATERIAL_CODE,
    MISSING_MESH_CODE, RENDER_SOURCE,
};
pub use extraction::{
    extract_render_scene, ExtractedCamera, ExtractedLight, ExtractedPrimitive, RenderExtraction,
};
pub use material::StandardMaterial;
pub use mesh::{PrimitiveMesh, PrimitiveMeshKind};
pub use smoke::{render_smoke_image, SmokeImage};

/// Returns the crate name for smoke tests and early integration checks.
#[must_use]
pub const fn crate_name() -> &'static str {
    "kinetik-render"
}

#[cfg(test)]
mod tests;

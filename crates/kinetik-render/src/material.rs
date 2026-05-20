use kinetik_core::Color;

/// PBR-compatible material scaffold used by primitive extraction.
#[derive(Clone, Debug, PartialEq)]
pub struct StandardMaterial {
    /// Base albedo color in linear RGBA.
    pub base_color: Color,
    /// Metallic factor in the PBR workflow.
    pub metallic: f32,
    /// Roughness factor in the PBR workflow.
    pub roughness: f32,
}

impl StandardMaterial {
    /// Safe fallback material used when authored material data is absent.
    pub const FALLBACK: Self = Self {
        base_color: Color::rgb(0.78, 0.82, 0.88),
        metallic: 0.0,
        roughness: 0.65,
    };

    /// Creates a material from PBR factors.
    #[must_use]
    pub fn new(base_color: Color, metallic: f32, roughness: f32) -> Self {
        Self {
            base_color,
            metallic: metallic.clamp(0.0, 1.0),
            roughness: roughness.clamp(0.0, 1.0),
        }
    }
}

impl Default for StandardMaterial {
    fn default() -> Self {
        Self::FALLBACK
    }
}

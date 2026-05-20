use kinetik_core::Color;

use crate::RenderExtraction;

/// Deterministic headless smoke-render output.
#[derive(Clone, Debug, PartialEq)]
pub struct SmokeImage {
    width: u32,
    height: u32,
    pixels: Vec<Color>,
}

impl SmokeImage {
    /// Creates a smoke image from dimensions and pixels.
    ///
    /// # Panics
    ///
    /// Panics if `pixels` does not match `width * height`.
    #[must_use]
    pub fn new(width: u32, height: u32, pixels: Vec<Color>) -> Self {
        assert_eq!(pixels.len(), (width * height) as usize);
        Self {
            width,
            height,
            pixels,
        }
    }

    /// Returns image width.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Returns image height.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Returns pixel data in row-major order.
    #[must_use]
    pub fn pixels(&self) -> &[Color] {
        &self.pixels
    }

    /// Returns whether any pixel differs from the background.
    #[must_use]
    pub fn has_non_background_pixels(&self) -> bool {
        self.pixels.iter().any(|pixel| *pixel != BACKGROUND)
    }
}

const BACKGROUND: Color = Color::rgb(0.02, 0.025, 0.03);

/// Produces a deterministic headless smoke image for extraction tests.
#[must_use]
pub fn render_smoke_image(extraction: &RenderExtraction, width: u32, height: u32) -> SmokeImage {
    let mut pixels = vec![BACKGROUND; (width * height) as usize];
    if extraction.camera.is_none() || extraction.primitives.is_empty() || width == 0 || height == 0
    {
        return SmokeImage::new(width, height, pixels);
    }

    for (index, primitive) in extraction.primitives.iter().enumerate() {
        let center_x = smoke_axis_position(primitive.transform.position.x, width);
        let center_y = smoke_axis_position(-primitive.transform.position.y, height);
        let radius = smoke_radius(index);
        let shade = shade_color(primitive.material.base_color, index);
        fill_square(
            &mut pixels,
            width,
            height,
            center_x,
            center_y,
            radius,
            shade,
        );
    }

    SmokeImage::new(width, height, pixels)
}

fn fill_square(
    pixels: &mut [Color],
    width: u32,
    height: u32,
    center_x: u32,
    center_y: u32,
    radius: u32,
    color: Color,
) {
    let min_y = center_y.saturating_sub(radius);
    let max_y = center_y
        .saturating_add(radius)
        .min(height.saturating_sub(1));
    let min_x = center_x.saturating_sub(radius);
    let max_x = center_x.saturating_add(radius).min(width.saturating_sub(1));

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            pixels[(y * width + x) as usize] = color;
        }
    }
}

fn smoke_axis_position(position: f32, length: u32) -> u32 {
    let midpoint = length / 2;
    let offset = length / 4;
    if position < -0.5 {
        midpoint.saturating_sub(offset)
    } else if position > 0.5 {
        midpoint
            .saturating_add(offset)
            .min(length.saturating_sub(1))
    } else {
        midpoint.min(length.saturating_sub(1))
    }
}

fn smoke_radius(index: usize) -> u32 {
    match index {
        0 => 4,
        1 => 3,
        _ => 2,
    }
}

fn shade_color(color: Color, index: usize) -> Color {
    let multiplier = match index {
        0 => 1.0,
        1 => 0.92,
        2 => 0.84,
        3 => 0.76,
        _ => 0.68,
    };
    Color::new(
        color.r * multiplier,
        color.g * multiplier,
        color.b * multiplier,
        color.a,
    )
}

//! Software-presented Studio template viewport.

use std::path::{Path, PathBuf};

use kinetik_core::Color;
use kinetik_render::{extract_render_scene, render_smoke_image, SmokeImage};
use kinetik_scene::InstanceClassRegistry;

use crate::{presentation_font::glyph_pattern, EditorSession};

const BACKGROUND: u32 = 0x0014_1619;
const PANEL: u32 = 0x0022_252b;
const PANEL_DARK: u32 = 0x001a_1d22;
const VIEWPORT: u32 = 0x0008_0a0d;
const BORDER: u32 = 0x0049_515c;
const TEXT: u32 = 0x00e5_e9ef;
const MUTED_TEXT: u32 = 0x0096_a0ad;
const ACCENT: u32 = 0x003d_ba83;
const WARNING: u32 = 0x00dc_a44a;

/// UI-presentable summary for all first-party templates.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TemplatePresentation {
    cards: Vec<TemplatePresentationCard>,
}

impl TemplatePresentation {
    /// Creates a presentation from template cards.
    #[must_use]
    pub(crate) fn new(cards: Vec<TemplatePresentationCard>) -> Self {
        Self { cards }
    }

    /// Returns template cards in display order.
    #[must_use]
    pub(crate) fn cards(&self) -> &[TemplatePresentationCard] {
        &self.cards
    }
}

/// UI-presentable state for one first-party template.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TemplatePresentationCard {
    /// Template display name.
    pub(crate) name: String,
    /// Repository-local template path.
    pub(crate) path: PathBuf,
    /// Number of extracted renderable primitives.
    pub(crate) primitive_count: usize,
    /// Number of extracted lights.
    pub(crate) light_count: usize,
    /// Whether a camera was extracted.
    pub(crate) has_camera: bool,
    /// Render diagnostics visible to the user if the viewport cannot draw.
    pub(crate) diagnostics: Vec<String>,
    image: SmokeImage,
}

impl TemplatePresentationCard {
    /// Returns the deterministic viewport image for this template.
    #[must_use]
    pub(crate) const fn image(&self) -> &SmokeImage {
        &self.image
    }
}

/// Software frame ready for presentation to a window.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct StudioPresentationFrame {
    /// Frame width in pixels.
    pub(crate) width: u32,
    /// Frame height in pixels.
    pub(crate) height: u32,
    /// Pixels in `0x00RRGGBB` format expected by `softbuffer`.
    pub(crate) pixels: Vec<u32>,
}

#[derive(Clone, Copy)]
struct Area {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl Area {
    const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

struct Canvas<'a> {
    pixels: &'a mut [u32],
    width: u32,
    height: u32,
}

impl<'a> Canvas<'a> {
    fn new(pixels: &'a mut [u32], width: u32, height: u32) -> Self {
        Self {
            pixels,
            width,
            height,
        }
    }

    fn fill(&mut self, area: Area, color: u32) {
        let max_x = area.x.saturating_add(area.width).min(self.width);
        let max_y = area.y.saturating_add(area.height).min(self.height);
        for row in area.y.min(self.height)..max_y {
            for column in area.x.min(self.width)..max_x {
                self.pixels[(row * self.width + column) as usize] = color;
            }
        }
    }

    fn stroke(&mut self, area: Area, color: u32) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        self.fill(Area::new(area.x, area.y, area.width, 1), color);
        self.fill(
            Area::new(
                area.x,
                area.y + area.height.saturating_sub(1),
                area.width,
                1,
            ),
            color,
        );
        self.fill(Area::new(area.x, area.y, 1, area.height), color);
        self.fill(
            Area::new(
                area.x + area.width.saturating_sub(1),
                area.y,
                1,
                area.height,
            ),
            color,
        );
    }

    fn text(&mut self, x: u32, y: u32, text: &str, color: u32) {
        let mut cursor = x;
        for character in text.chars() {
            if character == ' ' {
                cursor += 6;
                continue;
            }
            self.glyph(cursor, y, character.to_ascii_uppercase(), color);
            cursor += 6;
        }
    }

    fn glyph(&mut self, x: u32, y: u32, character: char, color: u32) {
        let pattern = glyph_pattern(character);
        for (row, bits) in pattern.iter().enumerate() {
            let Ok(row) = u32::try_from(row) else {
                continue;
            };
            for column in 0..5 {
                if bits & (1 << (4 - column)) == 0 {
                    continue;
                }
                self.set_pixel(x + column, y + row, color);
            }
        }
    }

    fn blit_smoke_image(&mut self, image: &SmokeImage, area: Area) {
        if area.width == 0 || area.height == 0 || image.width() == 0 || image.height() == 0 {
            return;
        }
        for target_y in 0..area.height {
            let source_y = target_y * image.height() / area.height;
            for target_x in 0..area.width {
                let source_x = target_x * image.width() / area.width;
                let source_index = (source_y * image.width() + source_x) as usize;
                self.set_pixel(
                    area.x + target_x,
                    area.y + target_y,
                    color_to_pixel(image.pixels()[source_index]),
                );
            }
        }
    }

    fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x < self.width && y < self.height {
            self.pixels[(y * self.width + x) as usize] = color;
        }
    }
}

/// Loads the first-party templates shown by the initial Studio UI.
///
/// # Errors
///
/// Returns a message when a template cannot be loaded or rendered.
pub(crate) fn default_template_presentation() -> Result<TemplatePresentation, String> {
    let templates_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../templates")
        .canonicalize()
        .map_err(|error| format!("templates directory missing: {error}"))?;
    let specs = [
        ("Primitive Showcase", "primitive-showcase"),
        ("PBR Material Demo", "pbr-material-demo"),
        ("Basic FPS", "basic-fps"),
    ];
    let mut cards = Vec::with_capacity(specs.len());
    for (name, directory) in specs {
        cards.push(load_template_card(name, &templates_root.join(directory))?);
    }
    Ok(TemplatePresentation::new(cards))
}

/// Renders the Studio shell and template viewport cards into a software frame.
#[must_use]
pub(crate) fn render_studio_presentation(
    presentation: &TemplatePresentation,
    width: u32,
    height: u32,
) -> StudioPresentationFrame {
    let width = width.max(1);
    let height = height.max(1);
    let mut pixels = vec![BACKGROUND; (width * height) as usize];
    let mut canvas = Canvas::new(&mut pixels, width, height);

    canvas.fill(Area::new(0, 0, width, 42), PANEL_DARK);
    canvas.text(18, 16, "KINETIK STUDIO", TEXT);
    canvas.text(180, 16, "FIRST-PARTY TEMPLATES VISIBLE IN UI", MUTED_TEXT);

    let content_top = 58;
    let content_bottom = height.saturating_sub(72);
    let left_width = width.min(260);
    let right_x = width.saturating_sub(280);
    canvas.fill(
        Area::new(
            14,
            content_top,
            left_width.saturating_sub(28),
            content_bottom.saturating_sub(content_top),
        ),
        PANEL,
    );
    canvas.fill(
        Area::new(
            right_x,
            content_top,
            width.saturating_sub(right_x + 14),
            content_bottom.saturating_sub(content_top),
        ),
        PANEL,
    );
    canvas.fill(
        Area::new(14, content_bottom + 14, width.saturating_sub(28), 44),
        PANEL,
    );
    canvas.text(28, content_top + 18, "EXPLORER", TEXT);
    canvas.text(right_x + 16, content_top + 18, "INSPECTOR", TEXT);
    canvas.text(
        28,
        content_bottom + 31,
        "DIAGNOSTICS: TEMPLATE VIEWPORT IS SOFTWARE-PRESENTED; FULL GPU VIEWPORT IS NEXT",
        MUTED_TEXT,
    );

    draw_template_cards(
        &mut canvas,
        presentation,
        Area::new(
            left_width + 20,
            content_top,
            right_x.saturating_sub(left_width + 36),
            content_bottom.saturating_sub(content_top),
        ),
    );

    StudioPresentationFrame {
        width,
        height,
        pixels,
    }
}

fn load_template_card(
    name: &str,
    template_root: &Path,
) -> Result<TemplatePresentationCard, String> {
    let mut session = EditorSession::new();
    session
        .reload_project_from(template_root, default_scene_registry()?)
        .map_err(|error| format!("{name} failed to load: {error}"))?;
    let scene = session
        .active_scene()
        .ok_or_else(|| format!("{name} has no active scene"))?;
    let extraction = extract_render_scene(scene);
    let image = render_smoke_image(&extraction, 320, 220);
    Ok(TemplatePresentationCard {
        name: name.to_owned(),
        path: template_root.to_path_buf(),
        primitive_count: extraction.primitives.len(),
        light_count: extraction.lights.len(),
        has_camera: extraction.camera.is_some(),
        diagnostics: extraction
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.clone())
            .collect(),
        image,
    })
}

fn default_scene_registry() -> Result<InstanceClassRegistry, String> {
    InstanceClassRegistry::with_default_scene_classes()
        .map_err(|error| format!("built-in scene registry failed: {error}"))
}

fn draw_template_cards(canvas: &mut Canvas<'_>, presentation: &TemplatePresentation, area: Area) {
    canvas.fill(area, VIEWPORT);
    canvas.text(area.x + 18, area.y + 20, "VIEWPORT", TEXT);

    if presentation.cards().is_empty() {
        canvas.text(area.x + 18, area.y + 54, "NO TEMPLATES LOADED", WARNING);
        return;
    }

    let gutter = 16;
    let card_count = u32::try_from(presentation.cards().len()).unwrap_or(1);
    let card_width = area
        .width
        .saturating_sub(gutter * (card_count + 1))
        .checked_div(card_count)
        .unwrap_or(1)
        .max(1);
    let card_height = area.height.saturating_sub(76).max(1);

    for (index, card) in presentation.cards().iter().enumerate() {
        let Ok(index) = u32::try_from(index) else {
            continue;
        };
        let card_x = area.x + gutter + (card_width + gutter) * index;
        draw_card(
            canvas,
            card,
            Area::new(card_x, area.y + 48, card_width, card_height),
        );
    }
}

fn draw_card(canvas: &mut Canvas<'_>, card: &TemplatePresentationCard, area: Area) {
    canvas.fill(area, PANEL);
    canvas.stroke(area, BORDER);
    canvas.text(area.x + 14, area.y + 20, &card.name, TEXT);

    let image_area = Area::new(
        area.x + 14,
        area.y + 42,
        area.width.saturating_sub(28),
        area.height.saturating_sub(126).max(24),
    );
    canvas.fill(image_area, 0x0005_070a);
    canvas.blit_smoke_image(card.image(), image_area);
    canvas.stroke(image_area, ACCENT);

    let status_y = area.y + area.height.saturating_sub(70);
    canvas.text(
        area.x + 14,
        status_y,
        &format!(
            "PRIMS {}  CAMERA {}  LIGHTS {}",
            card.primitive_count,
            if card.has_camera { "OK" } else { "MISS" },
            card.light_count
        ),
        MUTED_TEXT,
    );
    canvas.text(
        area.x + 14,
        status_y + 12,
        &format!("PATH {}", card.path.display()),
        MUTED_TEXT,
    );
    let diagnostic = card
        .diagnostics
        .first()
        .map_or("RENDER STATUS OK", String::as_str);
    canvas.text(
        area.x + 14,
        status_y + 34,
        diagnostic,
        if card.diagnostics.is_empty() {
            ACCENT
        } else {
            WARNING
        },
    );
}

fn color_to_pixel(color: Color) -> u32 {
    let red = color_channel_to_byte(color.r);
    let green = color_channel_to_byte(color.g);
    let blue = color_channel_to_byte(color.b);
    u32::from(blue) | (u32::from(green) << 8) | (u32::from(red) << 16)
}

fn color_channel_to_byte(value: f32) -> u8 {
    let value = value.clamp(0.0, 1.0);
    (0..=u8::MAX)
        .rev()
        .find(|level| f32::from(*level) / f32::from(u8::MAX) <= value)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_template_presentation_loads_all_first_party_templates() {
        let presentation = default_template_presentation().expect("load templates");

        assert_eq!(presentation.cards().len(), 3);
        assert!(presentation
            .cards()
            .iter()
            .any(|card| card.image().has_non_background_pixels()));
        assert_eq!(
            presentation
                .cards()
                .iter()
                .map(|card| card.primitive_count)
                .collect::<Vec<_>>(),
            vec![3, 4, 3]
        );
        assert!(presentation.cards().iter().all(|card| card.has_camera));
    }

    #[test]
    fn studio_presentation_frame_contains_template_pixels() {
        let presentation = default_template_presentation().expect("load templates");
        let frame = render_studio_presentation(&presentation, 640, 360);

        assert_eq!(frame.width, 640);
        assert_eq!(frame.height, 360);
        assert_eq!(frame.pixels.len(), 640 * 360);
        assert!(frame.pixels.iter().any(|pixel| *pixel != BACKGROUND));
    }
}

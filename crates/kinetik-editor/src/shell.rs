//! Kinetik Studio shell and first-window layout scaffold.

use std::{fmt, num::NonZeroU32, rc::Rc};

use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop, OwnedDisplayHandle},
    window::{Window, WindowAttributes},
};

use crate::presentation::{
    default_template_presentation, render_studio_presentation, TemplatePresentation,
};

const DEFAULT_WINDOW_TITLE: &str = "Kinetik Studio";
const DEFAULT_WINDOW_WIDTH: u32 = 1280;
const DEFAULT_WINDOW_HEIGHT: u32 = 800;

/// Editor panels present in the first Studio shell.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EditorPanel {
    /// Scene hierarchy panel placeholder.
    Explorer,
    /// Reflected property panel placeholder.
    Inspector,
    /// Project and runtime diagnostics panel placeholder.
    Diagnostics,
    /// Scene viewport placeholder.
    Viewport,
}

impl EditorPanel {
    /// Returns the stable user-facing panel label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Explorer => "Explorer",
            Self::Inspector => "Inspector",
            Self::Diagnostics => "Diagnostics",
            Self::Viewport => "Viewport",
        }
    }
}

/// Docking slot for the first non-persistent editor panel layout.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PanelDock {
    /// Left side of the editor window.
    Left,
    /// Right side of the editor window.
    Right,
    /// Bottom band of the editor window.
    Bottom,
    /// Center content area.
    Center,
}

/// Toolbar/menu placeholders that will be wired to editor commands later.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ToolbarAction {
    /// Open project placeholder.
    Open,
    /// Save project placeholder.
    Save,
    /// Enter play mode placeholder.
    Play,
    /// Stop play mode placeholder.
    Stop,
}

impl ToolbarAction {
    /// Returns the stable user-facing action label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::Save => "Save",
            Self::Play => "Play",
            Self::Stop => "Stop",
        }
    }
}

/// One panel placement in the first editor shell.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EditorShellWindow {
    /// Panel shown in this placement.
    pub panel: EditorPanel,
    /// Docking slot used by the shell layout.
    pub dock: PanelDock,
}

impl EditorShellWindow {
    /// Creates a panel placement.
    #[must_use]
    pub const fn new(panel: EditorPanel, dock: PanelDock) -> Self {
        Self { panel, dock }
    }
}

/// First editor shell layout model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EditorShellLayout {
    panels: Vec<EditorShellWindow>,
    toolbar: Vec<ToolbarAction>,
}

impl EditorShellLayout {
    /// Creates a layout from panel placements and toolbar placeholders.
    #[must_use]
    pub fn new(panels: Vec<EditorShellWindow>, toolbar: Vec<ToolbarAction>) -> Self {
        Self { panels, toolbar }
    }

    /// Returns panel placements in deterministic display order.
    #[must_use]
    pub fn panels(&self) -> &[EditorShellWindow] {
        &self.panels
    }

    /// Returns toolbar placeholders in deterministic display order.
    #[must_use]
    pub fn toolbar(&self) -> &[ToolbarAction] {
        &self.toolbar
    }
}

/// Returns the deterministic M16 shell layout.
#[must_use]
pub fn default_editor_shell_layout() -> EditorShellLayout {
    EditorShellLayout::new(
        vec![
            EditorShellWindow::new(EditorPanel::Explorer, PanelDock::Left),
            EditorShellWindow::new(EditorPanel::Viewport, PanelDock::Center),
            EditorShellWindow::new(EditorPanel::Inspector, PanelDock::Right),
            EditorShellWindow::new(EditorPanel::Diagnostics, PanelDock::Bottom),
        ],
        vec![
            ToolbarAction::Open,
            ToolbarAction::Save,
            ToolbarAction::Play,
            ToolbarAction::Stop,
        ],
    )
}

/// Public editor shell state owned by `kinetik-editor`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EditorShellState {
    window_title: String,
    window_size: (u32, u32),
    layout: EditorShellLayout,
    running: bool,
}

impl Default for EditorShellState {
    fn default() -> Self {
        Self {
            window_title: DEFAULT_WINDOW_TITLE.to_owned(),
            window_size: (DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT),
            layout: default_editor_shell_layout(),
            running: false,
        }
    }
}

impl EditorShellState {
    /// Creates the default editor shell state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the current window title.
    #[must_use]
    pub fn window_title(&self) -> &str {
        &self.window_title
    }

    /// Returns the requested window size in logical pixels.
    #[must_use]
    pub const fn window_size(&self) -> (u32, u32) {
        self.window_size
    }

    /// Returns the current shell layout.
    #[must_use]
    pub const fn layout(&self) -> &EditorShellLayout {
        &self.layout
    }

    /// Returns whether the shell lifecycle has entered a running state.
    #[must_use]
    pub const fn is_running(&self) -> bool {
        self.running
    }

    fn mark_running(&mut self) {
        self.running = true;
    }
}

/// Error returned when the editor shell cannot start or finish cleanly.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EditorShellError {
    message: String,
}

impl EditorShellError {
    fn from_error(error: impl fmt::Display) -> Self {
        Self {
            message: error.to_string(),
        }
    }

    /// Returns the launch error message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for EditorShellError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for EditorShellError {}

/// Runs the first Kinetik Studio shell window.
///
/// The window hosts editor-owned lifecycle and layout state plus a staged
/// software-presented template viewport. Full GPU viewport rendering, command
/// wiring, and renderer-backed picking are later roadmap slices.
///
/// # Errors
///
/// Returns an error when the platform event loop cannot be created or when the
/// event loop exits with a platform windowing error.
pub fn run_editor_shell() -> Result<(), EditorShellError> {
    let event_loop = EventLoop::new().map_err(EditorShellError::from_error)?;
    let context = softbuffer::Context::new(event_loop.owned_display_handle())
        .map_err(EditorShellError::from_error)?;
    let presentation = default_template_presentation().unwrap_or_else(|error| {
        eprintln!("failed to load template presentation: {error}");
        TemplatePresentation::new(Vec::new())
    });
    let mut app = WinitEditorShell::new(EditorShellState::new(), context, presentation);
    event_loop
        .run_app(&mut app)
        .map_err(EditorShellError::from_error)?;

    if let Some(error) = app.launch_error {
        return Err(error);
    }

    Ok(())
}

struct WinitEditorShell {
    state: EditorShellState,
    context: softbuffer::Context<OwnedDisplayHandle>,
    presentation: TemplatePresentation,
    window: Option<Rc<Window>>,
    surface: Option<softbuffer::Surface<OwnedDisplayHandle, Rc<Window>>>,
    launch_error: Option<EditorShellError>,
}

impl WinitEditorShell {
    fn new(
        state: EditorShellState,
        context: softbuffer::Context<OwnedDisplayHandle>,
        presentation: TemplatePresentation,
    ) -> Self {
        Self {
            state,
            context,
            presentation,
            window: None,
            surface: None,
            launch_error: None,
        }
    }

    fn window_attributes(&self) -> WindowAttributes {
        let (width, height) = self.state.window_size();
        Window::default_attributes()
            .with_title(self.state.window_title())
            .with_inner_size(LogicalSize::new(f64::from(width), f64::from(height)))
    }
}

impl ApplicationHandler for WinitEditorShell {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() || self.launch_error.is_some() {
            return;
        }

        match event_loop.create_window(self.window_attributes()) {
            Ok(window) => {
                let window = Rc::new(window);
                let surface = match softbuffer::Surface::new(&self.context, window.clone()) {
                    Ok(surface) => surface,
                    Err(error) => {
                        self.launch_error = Some(EditorShellError::from_error(error));
                        event_loop.exit();
                        return;
                    }
                };
                self.state.mark_running();
                window.request_redraw();
                self.window = Some(window);
                self.surface = Some(surface);
            }
            Err(error) => {
                self.launch_error = Some(EditorShellError::from_error(error));
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => {
                if let Err(error) = self.render_window() {
                    self.launch_error = Some(error);
                    event_loop.exit();
                }
            }
            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

impl WinitEditorShell {
    fn render_window(&mut self) -> Result<(), EditorShellError> {
        let window = self
            .window
            .as_ref()
            .ok_or_else(|| EditorShellError::from_error("window is not initialized"))?;
        let surface = self
            .surface
            .as_mut()
            .ok_or_else(|| EditorShellError::from_error("window surface is not initialized"))?;
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);
        surface
            .resize(
                NonZeroU32::new(width).expect("width is clamped to non-zero"),
                NonZeroU32::new(height).expect("height is clamped to non-zero"),
            )
            .map_err(EditorShellError::from_error)?;

        let frame = render_studio_presentation(&self.presentation, width, height);
        let mut buffer = surface.buffer_mut().map_err(EditorShellError::from_error)?;
        buffer.copy_from_slice(&frame.pixels);
        buffer.present().map_err(EditorShellError::from_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_shell_layout_contains_m16_panels_in_stable_order() {
        let layout = default_editor_shell_layout();

        assert_eq!(
            layout.panels(),
            [
                EditorShellWindow::new(EditorPanel::Explorer, PanelDock::Left),
                EditorShellWindow::new(EditorPanel::Viewport, PanelDock::Center),
                EditorShellWindow::new(EditorPanel::Inspector, PanelDock::Right),
                EditorShellWindow::new(EditorPanel::Diagnostics, PanelDock::Bottom),
            ]
        );
    }

    #[test]
    fn default_shell_toolbar_contains_first_actions_in_stable_order() {
        let layout = default_editor_shell_layout();

        assert_eq!(
            layout.toolbar(),
            [
                ToolbarAction::Open,
                ToolbarAction::Save,
                ToolbarAction::Play,
                ToolbarAction::Stop,
            ]
        );
    }

    #[test]
    fn shell_state_starts_with_default_window_contract() {
        let state = EditorShellState::new();

        assert_eq!(state.window_title(), "Kinetik Studio");
        assert_eq!(state.window_size(), (1280, 800));
        assert!(!state.is_running());
        assert_eq!(state.layout().panels().len(), 4);
    }

    #[test]
    fn panel_and_toolbar_labels_are_stable() {
        assert_eq!(EditorPanel::Explorer.label(), "Explorer");
        assert_eq!(EditorPanel::Inspector.label(), "Inspector");
        assert_eq!(EditorPanel::Diagnostics.label(), "Diagnostics");
        assert_eq!(EditorPanel::Viewport.label(), "Viewport");
        assert_eq!(ToolbarAction::Open.label(), "Open");
        assert_eq!(ToolbarAction::Save.label(), "Save");
        assert_eq!(ToolbarAction::Play.label(), "Play");
        assert_eq!(ToolbarAction::Stop.label(), "Stop");
    }
}

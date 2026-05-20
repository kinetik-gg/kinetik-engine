//! Renderer-independent viewport interaction state.

use core::fmt;

use kinetik_core::{Aabb, InstanceGuid, InstanceId, Vec2, Vec3};
use kinetik_scene::{Scene, SceneError};

use crate::{EditorDocumentSelection, EditorSelection};

const DEFAULT_DISTANCE: f32 = 10.0;
const MIN_DISTANCE: f32 = 0.05;
const DEFAULT_FOCUS_RADIUS: f32 = 1.0;
const DEFAULT_VIEWPORT_SIZE: Vec2 = Vec2::new(1280.0, 720.0);

/// Renderer-independent editor viewport state.
#[derive(Clone, Debug, PartialEq)]
pub struct ViewportState {
    camera: ViewportCameraState,
    selection_overlay: Option<ViewportSelectionOverlay>,
    size: Vec2,
}

impl ViewportState {
    /// Creates the default editor viewport state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            camera: ViewportCameraState::new(),
            selection_overlay: None,
            size: DEFAULT_VIEWPORT_SIZE,
        }
    }

    /// Returns the current camera state.
    #[must_use]
    pub const fn camera(&self) -> &ViewportCameraState {
        &self.camera
    }

    /// Returns the current selection overlay, when selection can be projected.
    #[must_use]
    pub const fn selection_overlay(&self) -> Option<&ViewportSelectionOverlay> {
        self.selection_overlay.as_ref()
    }

    /// Returns the viewport pixel size used by interaction math.
    #[must_use]
    pub const fn size(&self) -> Vec2 {
        self.size
    }

    /// Replaces the viewport size used by navigation and picking requests.
    ///
    /// # Errors
    ///
    /// Returns [`ViewportError::InvalidViewportSize`] when either dimension is
    /// zero or negative.
    pub fn resize(&mut self, size: Vec2) -> Result<(), ViewportError> {
        if size.x <= 0.0 || size.y <= 0.0 {
            return Err(ViewportError::InvalidViewportSize { size });
        }
        self.size = size;
        Ok(())
    }

    /// Applies orbit navigation deltas in radians.
    pub fn orbit(&mut self, yaw_delta: f32, pitch_delta: f32) {
        self.camera.orbit(yaw_delta, pitch_delta);
    }

    /// Applies pan navigation deltas in world units.
    pub fn pan(&mut self, delta: Vec2) {
        self.camera.pan(delta);
    }

    /// Applies dolly navigation delta in world units.
    pub fn dolly(&mut self, delta: f32) {
        self.camera.dolly(delta);
    }

    /// Focuses the camera and overlay on the active scene selection.
    ///
    /// # Errors
    ///
    /// Returns [`ViewportError`] when the active selection is not a scene
    /// instance or the selected instance cannot be projected from scene state.
    pub fn focus_selected(
        &mut self,
        scene: &Scene,
        selection: &EditorSelection,
    ) -> Result<ViewportFocusResult, ViewportError> {
        let target = ViewportFocusTarget::from_selection(scene, selection)?;
        self.camera.focus(target.center, target.radius);
        self.selection_overlay = Some(ViewportSelectionOverlay {
            instance_id: target.instance_id,
            guid: target.guid,
            scene_path: target.scene_path.clone(),
            bounds: target.bounds,
            center: target.center,
        });
        Ok(ViewportFocusResult { target })
    }

    /// Refreshes overlay data for the current selection without moving camera.
    ///
    /// # Errors
    ///
    /// Returns [`ViewportError`] when a scene instance selection exists but
    /// cannot be projected from scene state.
    pub fn refresh_selection_overlay(
        &mut self,
        scene: &Scene,
        selection: &EditorSelection,
    ) -> Result<(), ViewportError> {
        match ViewportFocusTarget::from_selection(scene, selection) {
            Ok(target) => {
                self.selection_overlay = Some(ViewportSelectionOverlay {
                    instance_id: target.instance_id,
                    guid: target.guid,
                    scene_path: target.scene_path,
                    bounds: target.bounds,
                    center: target.center,
                });
                Ok(())
            }
            Err(ViewportError::NoSceneInstanceSelection) => {
                self.selection_overlay = None;
                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    /// Returns a deterministic placeholder picking response.
    #[must_use]
    pub fn pick(&self, request: ViewportPickRequest) -> ViewportPickResponse {
        if request.position.x < 0.0
            || request.position.y < 0.0
            || request.position.x > self.size.x
            || request.position.y > self.size.y
        {
            return ViewportPickResponse::outside_viewport(request.position);
        }
        ViewportPickResponse::unsupported(request.position)
    }

    /// Returns a copyable viewport snapshot.
    #[must_use]
    pub fn snapshot(&self) -> ViewportSnapshot {
        ViewportSnapshot {
            camera: self.camera.clone(),
            selection_overlay: self.selection_overlay.clone(),
            size: self.size,
            picking: ViewportPickingContract::Placeholder,
        }
    }
}

impl Default for ViewportState {
    fn default() -> Self {
        Self::new()
    }
}

/// Orbit-style camera state for the editor viewport.
#[derive(Clone, Debug, PartialEq)]
pub struct ViewportCameraState {
    /// World-space orbit target.
    pub target: Vec3,
    /// Camera distance from target.
    pub distance: f32,
    /// Horizontal orbit angle in radians.
    pub yaw: f32,
    /// Vertical orbit angle in radians.
    pub pitch: f32,
}

impl ViewportCameraState {
    /// Creates the default editor viewport camera.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            target: Vec3::ZERO,
            distance: DEFAULT_DISTANCE,
            yaw: 0.0,
            pitch: -0.35,
        }
    }

    fn focus(&mut self, target: Vec3, radius: f32) {
        self.target = target;
        self.distance = (radius * 3.0).max(MIN_DISTANCE);
    }

    fn orbit(&mut self, yaw_delta: f32, pitch_delta: f32) {
        self.yaw += yaw_delta;
        self.pitch = (self.pitch + pitch_delta).clamp(-1.5, 1.5);
    }

    fn pan(&mut self, delta: Vec2) {
        self.target = Vec3::new(
            self.target.x + delta.x,
            self.target.y + delta.y,
            self.target.z,
        );
    }

    fn dolly(&mut self, delta: f32) {
        self.distance = (self.distance + delta).max(MIN_DISTANCE);
    }
}

impl Default for ViewportCameraState {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of data needed for selection highlighting.
#[derive(Clone, Debug, PartialEq)]
pub struct ViewportSelectionOverlay {
    /// Selected scene instance ID.
    pub instance_id: InstanceId,
    /// Stable instance GUID.
    pub guid: InstanceGuid,
    /// Scene path captured when the overlay was projected.
    pub scene_path: String,
    /// World bounds when the selected instance has concrete authoring bounds.
    pub bounds: Option<Aabb>,
    /// World-space overlay center.
    pub center: Vec3,
}

/// Focus result returned after focusing the active selection.
#[derive(Clone, Debug, PartialEq)]
pub struct ViewportFocusResult {
    /// Projected focus target.
    pub target: ViewportFocusTarget,
}

/// Projected focus target for a selected instance.
#[derive(Clone, Debug, PartialEq)]
pub struct ViewportFocusTarget {
    /// Selected scene instance ID.
    pub instance_id: InstanceId,
    /// Stable instance GUID.
    pub guid: InstanceGuid,
    /// Scene path captured during focus.
    pub scene_path: String,
    /// Camera focus center.
    pub center: Vec3,
    /// Camera focus radius.
    pub radius: f32,
    /// World bounds when available.
    pub bounds: Option<Aabb>,
}

impl ViewportFocusTarget {
    fn from_selection(scene: &Scene, selection: &EditorSelection) -> Result<Self, ViewportError> {
        let EditorDocumentSelection::SceneInstance {
            id,
            guid,
            scene_path,
        } = selection.document()
        else {
            return Err(ViewportError::NoSceneInstanceSelection);
        };

        let bounds = match scene.world_bounds(*id) {
            Ok(bounds) => Some(bounds),
            Err(SceneError::NoBounds { .. }) => None,
            Err(error) => return Err(ViewportError::Scene(error.to_string())),
        };
        let center = if let Some(bounds) = bounds {
            bounds.center()
        } else {
            scene
                .world_transform(*id)
                .map_err(|error| ViewportError::Scene(error.to_string()))?
                .position
        };
        let radius = bounds.map_or(DEFAULT_FOCUS_RADIUS, bounds_radius);

        Ok(Self {
            instance_id: *id,
            guid: *guid,
            scene_path: scene_path.clone(),
            center,
            radius,
            bounds,
        })
    }
}

/// Placeholder picking request before renderer-backed hit testing exists.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewportPickRequest {
    /// Pixel position in viewport coordinates.
    pub position: Vec2,
}

impl ViewportPickRequest {
    /// Creates a picking request from a viewport pixel position.
    #[must_use]
    pub const fn new(position: Vec2) -> Self {
        Self { position }
    }
}

/// Renderer-independent picking response.
#[derive(Clone, Debug, PartialEq)]
pub struct ViewportPickResponse {
    /// Requested pixel position.
    pub position: Vec2,
    /// Picking status.
    pub status: ViewportPickStatus,
    /// Picked instance ID when renderer-backed picking later produces one.
    pub instance_id: Option<InstanceId>,
    /// Human-readable diagnostic for unsupported or invalid requests.
    pub diagnostic: Option<String>,
}

impl ViewportPickResponse {
    fn unsupported(position: Vec2) -> Self {
        Self {
            position,
            status: ViewportPickStatus::Unsupported,
            instance_id: None,
            diagnostic: Some(
                "viewport picking is waiting for renderer-backed hit testing".to_owned(),
            ),
        }
    }

    fn outside_viewport(position: Vec2) -> Self {
        Self {
            position,
            status: ViewportPickStatus::OutsideViewport,
            instance_id: None,
            diagnostic: Some("pick position is outside the viewport bounds".to_owned()),
        }
    }
}

/// Picking contract status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ViewportPickStatus {
    /// Renderer-backed picking is not implemented yet.
    Unsupported,
    /// The request fell outside the viewport rectangle.
    OutsideViewport,
    /// A future renderer-backed request found no instance.
    NoHit,
    /// A future renderer-backed request selected an instance.
    Hit,
}

/// Picking implementation contract advertised in viewport snapshots.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ViewportPickingContract {
    /// Placeholder contract before renderer-backed hit testing exists.
    Placeholder,
    /// Future renderer-backed picking.
    RendererBacked,
}

/// Copyable viewport snapshot for UI, tests, and MCP/headless inspection.
#[derive(Clone, Debug, PartialEq)]
pub struct ViewportSnapshot {
    /// Current camera state.
    pub camera: ViewportCameraState,
    /// Current selection overlay, if projectable.
    pub selection_overlay: Option<ViewportSelectionOverlay>,
    /// Viewport size.
    pub size: Vec2,
    /// Current picking contract.
    pub picking: ViewportPickingContract,
}

/// Viewport interaction errors.
#[derive(Clone, Debug, PartialEq)]
pub enum ViewportError {
    /// No scene instance is selected.
    NoSceneInstanceSelection,
    /// The viewport size is invalid.
    InvalidViewportSize {
        /// Rejected viewport size.
        size: Vec2,
    },
    /// Scene projection failed.
    Scene(String),
}

impl fmt::Display for ViewportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSceneInstanceSelection => {
                formatter.write_str("no scene instance is selected for viewport focus")
            }
            Self::InvalidViewportSize { size } => {
                write!(
                    formatter,
                    "viewport size must be positive, got {}x{}",
                    size.x, size.y
                )
            }
            Self::Scene(error) => formatter.write_str(error),
        }
    }
}

impl std::error::Error for ViewportError {}

fn bounds_radius(bounds: Aabb) -> f32 {
    let size = bounds.size();
    (size.x.max(size.y).max(size.z) * 0.5).max(DEFAULT_FOCUS_RADIUS)
}

#[cfg(test)]
mod tests;

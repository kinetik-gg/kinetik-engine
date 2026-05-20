use kinetik_core::Vec2;

/// Gameplay actions understood by the first input foundation.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum InputAction {
    /// Move toward the current view forward direction.
    MoveForward,
    /// Move opposite the current view forward direction.
    MoveBackward,
    /// Strafe left relative to the current view direction.
    MoveLeft,
    /// Strafe right relative to the current view direction.
    MoveRight,
    /// Trigger the focused interaction primitive.
    Interact,
}

/// Mouse capture behavior requested by a gameplay mode.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MouseCapturePolicy {
    /// Mouse is free for editor or UI interaction.
    Released,
    /// Gameplay requests captured pointer deltas.
    Captured,
}

/// Deterministic input snapshot consumed by headless gameplay tests.
#[derive(Debug, Clone, PartialEq)]
pub struct InputFrame {
    actions: Vec<InputAction>,
    look_delta: Vec2,
    mouse_capture: MouseCapturePolicy,
}

impl InputFrame {
    /// Creates an empty released input snapshot.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            actions: Vec::new(),
            look_delta: Vec2::ZERO,
            mouse_capture: MouseCapturePolicy::Released,
        }
    }

    /// Creates a snapshot from pressed actions.
    #[must_use]
    pub fn from_actions(actions: impl IntoIterator<Item = InputAction>) -> Self {
        let mut frame = Self::new();
        for action in actions {
            frame = frame.with_action(action);
        }
        frame
    }

    /// Returns pressed actions in deterministic order.
    #[must_use]
    pub fn actions(&self) -> &[InputAction] {
        &self.actions
    }

    /// Returns whether an action is pressed.
    #[must_use]
    pub fn is_pressed(&self, action: InputAction) -> bool {
        self.actions.contains(&action)
    }

    /// Returns accumulated look delta for this frame.
    #[must_use]
    pub const fn look_delta(&self) -> Vec2 {
        self.look_delta
    }

    /// Returns requested mouse capture policy.
    #[must_use]
    pub const fn mouse_capture(&self) -> MouseCapturePolicy {
        self.mouse_capture
    }

    /// Adds a pressed action while preserving deterministic action order.
    #[must_use]
    pub fn with_action(mut self, action: InputAction) -> Self {
        if !self.actions.contains(&action) {
            self.actions.push(action);
            self.actions.sort_unstable();
        }
        self
    }

    /// Sets look delta for this frame.
    #[must_use]
    pub const fn with_look_delta(mut self, look_delta: Vec2) -> Self {
        self.look_delta = look_delta;
        self
    }

    /// Sets mouse capture policy for this frame.
    #[must_use]
    pub const fn with_mouse_capture(mut self, mouse_capture: MouseCapturePolicy) -> Self {
        self.mouse_capture = mouse_capture;
        self
    }

    /// Returns normalized local movement intent as X/Z components.
    #[must_use]
    pub fn movement_intent(&self) -> Vec2 {
        let x = f32::from(self.is_pressed(InputAction::MoveRight))
            - f32::from(self.is_pressed(InputAction::MoveLeft));
        let y = f32::from(self.is_pressed(InputAction::MoveForward))
            - f32::from(self.is_pressed(InputAction::MoveBackward));
        normalize_vec2(Vec2::new(x, y))
    }
}

impl Default for InputFrame {
    fn default() -> Self {
        Self::new()
    }
}

fn normalize_vec2(value: Vec2) -> Vec2 {
    let length_squared = value.x.mul_add(value.x, value.y * value.y);
    if length_squared <= f32::EPSILON {
        return Vec2::ZERO;
    }
    let inv_length = 1.0 / length_squared.sqrt();
    Vec2::new(value.x * inv_length, value.y * inv_length)
}

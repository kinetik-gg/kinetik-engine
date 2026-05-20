use kinetik_core::{Aabb, Vec2, Vec3};

use crate::InputFrame;

/// Static collider used by the first deterministic collision foundation.
#[derive(Debug, Clone, PartialEq)]
pub struct StaticCollider {
    /// Stable human-readable collider label.
    pub label: String,
    /// World-space collider bounds.
    pub bounds: Aabb,
}

impl StaticCollider {
    /// Creates a static collider from a label and world-space bounds.
    #[must_use]
    pub fn new(label: impl Into<String>, bounds: Aabb) -> Self {
        Self {
            label: label.into(),
            bounds,
        }
    }
}

/// Static primitive collision world used by headless controller tests.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StaticCollisionWorld {
    colliders: Vec<StaticCollider>,
}

impl StaticCollisionWorld {
    /// Creates an empty collision world.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            colliders: Vec::new(),
        }
    }

    /// Creates a collision world from static colliders.
    #[must_use]
    pub fn from_colliders(colliders: impl IntoIterator<Item = StaticCollider>) -> Self {
        Self {
            colliders: colliders.into_iter().collect(),
        }
    }

    /// Returns static colliders in deterministic order.
    #[must_use]
    pub fn colliders(&self) -> &[StaticCollider] {
        &self.colliders
    }

    /// Adds a static collider.
    pub fn push(&mut self, collider: StaticCollider) {
        self.colliders.push(collider);
    }

    /// Returns whether a character capsule footprint overlaps any static collider.
    #[must_use]
    pub fn overlaps_character(&self, position: Vec3, config: CharacterControllerConfig) -> bool {
        self.colliders
            .iter()
            .any(|collider| character_overlaps_aabb(position, config, collider.bounds))
    }
}

/// Tunable character controller values.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CharacterControllerConfig {
    /// Horizontal movement speed in world units per second.
    pub move_speed: f32,
    /// Mouse-look yaw sensitivity in radians per input unit.
    pub yaw_sensitivity: f32,
    /// Mouse-look pitch sensitivity in radians per input unit.
    pub pitch_sensitivity: f32,
    /// Character collision radius on the X/Z plane.
    pub radius: f32,
    /// Character half-height for coarse Y-overlap checks.
    pub half_height: f32,
    /// Absolute pitch clamp in radians.
    pub max_pitch: f32,
}

impl CharacterControllerConfig {
    /// Creates a controller configuration.
    ///
    /// # Panics
    ///
    /// Panics if dimensions, speed, or pitch clamp are negative.
    #[must_use]
    pub fn new(
        move_speed: f32,
        yaw_sensitivity: f32,
        pitch_sensitivity: f32,
        radius: f32,
        half_height: f32,
        max_pitch: f32,
    ) -> Self {
        assert!(move_speed >= 0.0, "move speed must be non-negative");
        assert!(radius >= 0.0, "radius must be non-negative");
        assert!(half_height >= 0.0, "half height must be non-negative");
        assert!(max_pitch >= 0.0, "max pitch must be non-negative");
        Self {
            move_speed,
            yaw_sensitivity,
            pitch_sensitivity,
            radius,
            half_height,
            max_pitch,
        }
    }
}

impl Default for CharacterControllerConfig {
    fn default() -> Self {
        Self::new(5.0, 0.0025, 0.0025, 0.35, 0.9, 1.45)
    }
}

/// Character state consumed by headless gameplay prototypes.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CharacterControllerState {
    /// Character center position in world space.
    pub position: Vec3,
    /// Yaw in radians.
    pub yaw: f32,
    /// Pitch in radians.
    pub pitch: f32,
}

impl CharacterControllerState {
    /// Creates a character state at a position.
    #[must_use]
    pub const fn new(position: Vec3) -> Self {
        Self {
            position,
            yaw: 0.0,
            pitch: 0.0,
        }
    }

    /// Returns the horizontal forward vector for this yaw.
    #[must_use]
    pub fn forward(self) -> Vec3 {
        Vec3::new(self.yaw.sin(), 0.0, -self.yaw.cos())
    }

    /// Returns the horizontal right vector for this yaw.
    #[must_use]
    pub fn right(self) -> Vec3 {
        Vec3::new(self.yaw.cos(), 0.0, self.yaw.sin())
    }

    /// Steps the controller using deterministic input and static collision.
    ///
    /// # Panics
    ///
    /// Panics when `delta_seconds` is negative.
    #[must_use]
    pub fn step(
        self,
        input: &InputFrame,
        delta_seconds: f32,
        config: CharacterControllerConfig,
        world: &StaticCollisionWorld,
    ) -> CharacterControllerStep {
        assert!(delta_seconds >= 0.0, "delta seconds must be non-negative");

        let mut next = self;
        let look_delta = input.look_delta();
        next.yaw += look_delta.x * config.yaw_sensitivity;
        next.pitch = (next.pitch + look_delta.y * config.pitch_sensitivity)
            .clamp(-config.max_pitch, config.max_pitch);

        let movement = movement_delta(
            self,
            input.movement_intent(),
            config.move_speed * delta_seconds,
        );
        let after_x = Vec3::new(
            self.position.x + movement.x,
            self.position.y,
            self.position.z,
        );
        let x_blocked = world.overlaps_character(after_x, config);
        if !x_blocked {
            next.position.x = after_x.x;
        }

        let after_z = Vec3::new(
            next.position.x,
            self.position.y,
            self.position.z + movement.z,
        );
        let z_blocked = world.overlaps_character(after_z, config);
        if !z_blocked {
            next.position.z = after_z.z;
        }

        CharacterControllerStep {
            previous: self,
            next,
            attempted_delta: movement,
            applied_delta: Vec3::new(
                next.position.x - self.position.x,
                next.position.y - self.position.y,
                next.position.z - self.position.z,
            ),
            collided: x_blocked || z_blocked,
        }
    }
}

/// Result of a character controller step.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CharacterControllerStep {
    /// State before the step.
    pub previous: CharacterControllerState,
    /// State after movement and look are applied.
    pub next: CharacterControllerState,
    /// Desired movement delta before collision.
    pub attempted_delta: Vec3,
    /// Applied movement delta after collision.
    pub applied_delta: Vec3,
    /// Whether any movement axis was blocked by collision.
    pub collided: bool,
}

fn movement_delta(state: CharacterControllerState, intent: Vec2, distance: f32) -> Vec3 {
    let forward = state.forward();
    let right = state.right();
    Vec3::new(
        (right.x * intent.x + forward.x * intent.y) * distance,
        0.0,
        (right.z * intent.x + forward.z * intent.y) * distance,
    )
}

fn character_overlaps_aabb(
    position: Vec3,
    config: CharacterControllerConfig,
    bounds: Aabb,
) -> bool {
    let vertical_min = position.y - config.half_height;
    let vertical_max = position.y + config.half_height;
    if vertical_max < bounds.min.y || vertical_min > bounds.max.y {
        return false;
    }

    let closest_x = position.x.clamp(bounds.min.x, bounds.max.x);
    let closest_z = position.z.clamp(bounds.min.z, bounds.max.z);
    let dx = position.x - closest_x;
    let dz = position.z - closest_z;
    dx.mul_add(dx, dz * dz) <= config.radius * config.radius
}

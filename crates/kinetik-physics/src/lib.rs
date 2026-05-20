//! Physics and interaction foundation contracts for Kinetik.

mod controller;
mod input;
mod interaction;

pub use controller::{
    CharacterControllerConfig, CharacterControllerState, CharacterControllerStep, StaticCollider,
    StaticCollisionWorld,
};
pub use input::{InputAction, InputFrame, MouseCapturePolicy};
pub use interaction::{raycast_static_world, InteractionHit, InteractionRay};

/// Returns the crate name for smoke tests and early integration checks.
#[must_use]
pub const fn crate_name() -> &'static str {
    "kinetik-physics"
}

#[cfg(test)]
mod tests;

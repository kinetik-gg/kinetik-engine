//! Core primitives for Kinetik Engine.
//!
//! This crate contains small shared foundation types used by higher-level crates.

mod diagnostics;
mod error;
mod id;
mod math;

pub use diagnostics::{
    AgentRepair, Diagnostic, DiagnosticBlockingScope, DiagnosticCode, DiagnosticLocation,
    DiagnosticSeverity, DiagnosticSource, SourceRange,
};
pub use error::{KinetikError, KinetikResult};
pub use id::{BundleId, InstanceGuid, InstanceId, ResourceId, ScriptId, SignalId};
pub use math::{Aabb, Color, Quat, Rect, Transform, Vec2, Vec3, Vec4};

#[cfg(test)]
mod tests;

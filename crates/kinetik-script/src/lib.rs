//! Runtime-agnostic scripting contracts for Kinetik.
//!
//! Concrete language bridges, such as Luau, implement these contracts without
//! exposing VM internals to the runtime, editor, resources, or diagnostics.

mod diagnostics;
mod handles;
mod identity;
mod lifecycle;
mod structural;

#[cfg(test)]
mod tests;

pub use diagnostics::{
    invalid_script_handle_diagnostic, missing_script_diagnostic, ScriptDiagnosticCode,
    ScriptDiagnosticContext,
};
pub use handles::{ResourceScriptHandle, ScriptHandleError, ScriptInstanceHandle};
pub use identity::{
    ScriptAssetRef, ScriptAttachment, ScriptAttachmentId, ScriptAttachmentTarget, ScriptLanguage,
};
pub use lifecycle::{
    LifecycleCall, LifecycleCallLog, LifecyclePhase, ScriptRuntime, ScriptRuntimeHost,
};
pub use structural::{
    QueuedScriptChange, ScriptChangeQueue, ScriptPropertyValue, StructuralChangeKind,
};

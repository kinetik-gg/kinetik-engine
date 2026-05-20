use core::fmt;

/// Stable command target mode.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CommandTargetMode {
    /// Command targets editable source/project state.
    Edit,
    /// Command targets sandboxed runtime/play state.
    Play,
}

impl fmt::Display for CommandTargetMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Edit => f.write_str("edit"),
            Self::Play => f.write_str("play"),
        }
    }
}

/// Command execution status.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CommandStatus {
    /// Command validated and completed.
    Succeeded,
    /// Command was rejected before mutation.
    Failed,
}

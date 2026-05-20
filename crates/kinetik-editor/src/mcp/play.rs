use kinetik_command::{
    require_specific_target_mode, CommandError, CommandStatus, CommandTargetMode,
};

use crate::{diagnostics_list_response, DiagnosticSummary, EditorPlayError, EditorSession};

/// Editor-owned play MCP command names.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum McpPlayCommandName {
    /// Start play mode.
    Start,
    /// Stop play mode.
    Stop,
    /// Step play mode.
    Step,
}

impl McpPlayCommandName {
    /// Returns the stable MCP command name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Start => "play.start",
            Self::Stop => "play.stop",
            Self::Step => "play.step",
        }
    }
}

/// MCP play command request.
#[derive(Debug, Clone, PartialEq)]
pub enum McpPlayCommand {
    /// Start play mode from the current edit scene.
    Start {
        /// Requested target mode.
        target_mode: Option<CommandTargetMode>,
    },
    /// Stop and destroy the active play world.
    Stop {
        /// Requested target mode.
        target_mode: Option<CommandTargetMode>,
    },
    /// Step one runtime frame.
    Step {
        /// Requested target mode.
        target_mode: Option<CommandTargetMode>,
        /// Variable frame delta in seconds.
        delta_seconds: f32,
    },
}

/// Read-only MCP play state response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpPlayStateResponse {
    /// Whether play mode owns a runtime world.
    pub is_playing: bool,
    /// Runtime world ID raw value, when active.
    pub runtime_world_id: Option<u64>,
    /// Runtime instance count, when active.
    pub runtime_instance_count: usize,
    /// Root runtime instance ID raw value, when active.
    pub root_runtime_instance_id: Option<u64>,
    /// Most recently completed frame index.
    pub frame_index: Option<u64>,
    /// Next fixed-step index.
    pub next_fixed_step_index: Option<u64>,
}

/// MCP play command response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpPlayCommandResponse {
    /// Stable command kind.
    pub command_kind: String,
    /// Command target mode.
    pub target_mode: Option<CommandTargetMode>,
    /// Command status.
    pub status: CommandStatus,
    /// Validation diagnostics.
    pub diagnostics: Vec<DiagnosticSummary>,
    /// Play state after the command.
    pub play_state: McpPlayStateResponse,
}

impl McpPlayCommand {
    fn command_name(&self) -> McpPlayCommandName {
        match self {
            Self::Start { .. } => McpPlayCommandName::Start,
            Self::Stop { .. } => McpPlayCommandName::Stop,
            Self::Step { .. } => McpPlayCommandName::Step,
        }
    }

    const fn target_mode(&self) -> Option<CommandTargetMode> {
        match self {
            Self::Start { target_mode }
            | Self::Stop { target_mode }
            | Self::Step { target_mode, .. } => *target_mode,
        }
    }
}

impl EditorSession {
    /// Returns current play state for MCP snapshots.
    #[must_use]
    pub fn mcp_play_state(&self) -> McpPlayStateResponse {
        let Some(play) = self.play_session() else {
            return McpPlayStateResponse {
                is_playing: false,
                runtime_world_id: None,
                runtime_instance_count: 0,
                root_runtime_instance_id: None,
                frame_index: None,
                next_fixed_step_index: None,
            };
        };

        McpPlayStateResponse {
            is_playing: true,
            runtime_world_id: Some(play.world().id().raw()),
            runtime_instance_count: play.world().instances().len(),
            root_runtime_instance_id: play
                .world()
                .root_id()
                .map(kinetik_app::RuntimeInstanceId::raw),
            frame_index: Some(play.scheduler().frame_index()),
            next_fixed_step_index: Some(play.scheduler().next_fixed_step_index()),
        }
    }

    /// Executes an MCP play command through editor-owned play controls.
    pub fn mcp_execute_play_command(&mut self, command: &McpPlayCommand) -> McpPlayCommandResponse {
        let command_kind = command.command_name().as_str();
        let target_mode = command.target_mode();
        let result =
            require_specific_target_mode(command_kind, target_mode, required_target_mode(command))
                .and_then(|_| self.execute_play_command(command));
        let diagnostics = result
            .err()
            .map(|error| vec![error.to_diagnostic()])
            .unwrap_or_default();
        McpPlayCommandResponse {
            command_kind: command_kind.to_owned(),
            target_mode,
            status: if diagnostics.is_empty() {
                CommandStatus::Succeeded
            } else {
                CommandStatus::Failed
            },
            diagnostics: diagnostics_list_response(&diagnostics),
            play_state: self.mcp_play_state(),
        }
    }

    fn execute_play_command(&mut self, command: &McpPlayCommand) -> Result<(), CommandError> {
        match command {
            McpPlayCommand::Start { .. } => {
                self.start_play_mode().map_err(|error| play_error(&error))
            }
            McpPlayCommand::Stop { .. } => {
                if self.play_session().is_none() {
                    return Err(play_error(&EditorPlayError::NotPlaying));
                }
                self.stop_play_mode();
                Ok(())
            }
            McpPlayCommand::Step { delta_seconds, .. } => self
                .step_play_mode(*delta_seconds)
                .map(|_| ())
                .map_err(|error| play_error(&error)),
        }
    }
}

fn required_target_mode(command: &McpPlayCommand) -> CommandTargetMode {
    match command {
        McpPlayCommand::Start { .. } => CommandTargetMode::Edit,
        McpPlayCommand::Stop { .. } | McpPlayCommand::Step { .. } => CommandTargetMode::Play,
    }
}

fn play_error(error: &EditorPlayError) -> CommandError {
    CommandError::ValidationFailed {
        command_kind: "play".to_owned(),
        reason: error.to_string(),
    }
}

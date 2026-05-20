use std::fmt;

use kinetik_app::{
    FrameScheduler, FrameStepResult, RuntimeInstanceId, RuntimeWorld, RuntimeWorldId,
};
use kinetik_core::{
    Diagnostic, DiagnosticBlockingScope, DiagnosticCode, DiagnosticSeverity, DiagnosticSource,
};

use crate::{EditorModeState, EditorSession};

const DEFAULT_FIXED_DELTA_SECONDS: f32 = 1.0 / 60.0;

/// Error returned by editor play-mode controls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorPlayError {
    /// A project scene must be open before play can start.
    NoActiveScene,
    /// Play mode is already active.
    AlreadyPlaying,
    /// Play mode is not active.
    NotPlaying,
    /// The edit scene could not be cloned into a runtime sandbox.
    Scene(String),
}

impl fmt::Display for EditorPlayError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoActiveScene => formatter.write_str("no active scene is open"),
            Self::AlreadyPlaying => formatter.write_str("play mode is already active"),
            Self::NotPlaying => formatter.write_str("play mode is not active"),
            Self::Scene(error) => formatter.write_str(error),
        }
    }
}

impl std::error::Error for EditorPlayError {}

/// Editor-owned play-mode sandbox state.
#[derive(Debug, Clone, PartialEq)]
pub struct EditorPlaySession {
    world: RuntimeWorld,
    scheduler: FrameScheduler,
    last_step: Option<FrameStepResult>,
    diagnostics: Vec<Diagnostic>,
}

impl EditorPlaySession {
    fn new(world: RuntimeWorld) -> Self {
        let mut session = Self {
            world,
            scheduler: FrameScheduler::new(DEFAULT_FIXED_DELTA_SECONDS),
            last_step: None,
            diagnostics: Vec::new(),
        };
        session.replace_play_diagnostic("play world started");
        session
    }

    /// Returns the sandboxed runtime world.
    #[must_use]
    pub const fn world(&self) -> &RuntimeWorld {
        &self.world
    }

    /// Returns mutable sandboxed runtime world state for runtime-only tests.
    #[must_use]
    pub fn world_mut(&mut self) -> &mut RuntimeWorld {
        &mut self.world
    }

    /// Returns the frame scheduler.
    #[must_use]
    pub const fn scheduler(&self) -> &FrameScheduler {
        &self.scheduler
    }

    /// Returns the latest frame step result.
    #[must_use]
    pub const fn last_step(&self) -> Option<&FrameStepResult> {
        self.last_step.as_ref()
    }

    /// Returns current play diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    fn step(&mut self, delta_seconds: f32) -> FrameStepResult {
        let result = self.scheduler.step(delta_seconds);
        self.replace_play_diagnostic(format!("runtime frame {} stepped", result.frame_index));
        self.last_step = Some(result.clone());
        result
    }

    fn replace_play_diagnostic(&mut self, message: impl Into<String>) {
        self.diagnostics = vec![Diagnostic::new(
            DiagnosticCode::new("KT_RUNTIME_PLAY_STATE"),
            DiagnosticSeverity::Info,
            DiagnosticSource::new("Runtime"),
            message,
        )
        .with_blocking_scope(DiagnosticBlockingScope::Play)];
    }
}

impl EditorSession {
    /// Starts play mode by cloning the active edit scene into a runtime sandbox.
    ///
    /// # Errors
    ///
    /// Returns [`EditorPlayError`] when no scene is open, play is already
    /// active, or the scene cannot be cloned.
    pub fn start_play_mode(&mut self) -> Result<(), EditorPlayError> {
        if self.play.is_some() {
            return Err(EditorPlayError::AlreadyPlaying);
        }
        let scene = self.active_scene().ok_or(EditorPlayError::NoActiveScene)?;
        let world = RuntimeWorld::clone_from_edit_scene(RuntimeWorldId::new(1), scene)
            .map_err(|error| EditorPlayError::Scene(error.to_string()))?;
        self.play = Some(EditorPlaySession::new(world));
        self.mode = EditorModeState::Play;
        Ok(())
    }

    /// Stops play mode and destroys the runtime sandbox.
    pub fn stop_play_mode(&mut self) {
        self.play = None;
        self.mode = EditorModeState::Edit;
    }

    /// Steps the active runtime sandbox.
    ///
    /// # Errors
    ///
    /// Returns [`EditorPlayError::NotPlaying`] when play mode is not active.
    pub fn step_play_mode(
        &mut self,
        delta_seconds: f32,
    ) -> Result<FrameStepResult, EditorPlayError> {
        let play = self.play.as_mut().ok_or(EditorPlayError::NotPlaying)?;
        Ok(play.step(delta_seconds))
    }

    /// Returns active play session state.
    #[must_use]
    pub const fn play_session(&self) -> Option<&EditorPlaySession> {
        self.play.as_ref()
    }

    /// Returns mutable play session state.
    #[must_use]
    pub fn play_session_mut(&mut self) -> Option<&mut EditorPlaySession> {
        self.play.as_mut()
    }

    /// Returns the runtime instance ID corresponding to an edit selection.
    #[must_use]
    pub fn runtime_id_for_selected_edit_guid(&self) -> Option<RuntimeInstanceId> {
        let play = self.play.as_ref()?;
        let crate::EditorDocumentSelection::SceneInstance { guid, .. } =
            self.selection().document()
        else {
            return None;
        };
        play.world().runtime_id_for_edit_guid(*guid)
    }

    pub(crate) fn play_diagnostics(&self) -> &[Diagnostic] {
        self.play
            .as_ref()
            .map_or(&[], EditorPlaySession::diagnostics)
    }
}

#[cfg(test)]
mod tests;

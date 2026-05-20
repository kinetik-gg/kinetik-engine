use core::{fmt, num::NonZeroU64};

use crate::{
    validate_dirty_summary, CommandChangeRecord, CommandModelResult, CommandResult,
    CommandTargetMode,
};

/// User-facing undo group identity.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UndoGroupId(NonZeroU64);

impl UndoGroupId {
    /// Creates an undo group ID from a non-zero raw value.
    ///
    /// # Panics
    ///
    /// Panics when `raw` is zero.
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        let Some(raw) = NonZeroU64::new(raw) else {
            panic!("UndoGroupId raw value must be non-zero");
        };
        Self(raw)
    }

    /// Returns the raw non-zero ID value.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0.get()
    }
}

impl fmt::Display for UndoGroupId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UndoGroupId({})", self.raw())
    }
}

/// Undoable command history record.
#[derive(Debug, Clone, PartialEq)]
pub struct UndoRedoRecord {
    group_id: UndoGroupId,
    target_mode: CommandTargetMode,
    summary: String,
    changes: Vec<CommandChangeRecord>,
}

impl UndoRedoRecord {
    /// Creates an undoable command history record.
    ///
    /// # Errors
    ///
    /// Returns [`crate::CommandError::EmptyDirtySummary`] when `summary` is empty.
    pub fn new<I>(
        group_id: UndoGroupId,
        target_mode: CommandTargetMode,
        summary: impl Into<String>,
        changes: I,
    ) -> CommandModelResult<Self>
    where
        I: IntoIterator<Item = CommandChangeRecord>,
    {
        Ok(Self {
            group_id,
            target_mode,
            summary: validate_dirty_summary(summary.into())?,
            changes: changes.into_iter().collect(),
        })
    }

    /// Returns the undo group ID.
    #[must_use]
    pub const fn group_id(&self) -> UndoGroupId {
        self.group_id
    }

    /// Returns the command target mode.
    #[must_use]
    pub const fn target_mode(&self) -> CommandTargetMode {
        self.target_mode
    }

    /// Returns the user-facing undo/redo summary.
    #[must_use]
    pub fn summary(&self) -> &str {
        &self.summary
    }

    /// Returns semantic changes in deterministic command execution order.
    #[must_use]
    pub fn changes(&self) -> &[CommandChangeRecord] {
        &self.changes
    }
}

/// Deterministic command history stacks for undo and redo.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandHistory {
    undo_stack: Vec<UndoRedoRecord>,
    redo_stack: Vec<UndoRedoRecord>,
    next_undo_group_id: u64,
}

impl CommandHistory {
    /// Creates an empty command history.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            next_undo_group_id: 1,
        }
    }

    /// Commits a successful command result as an undoable history record.
    ///
    /// Failed results and successful results without changes are not undoable
    /// and return `None`.
    ///
    /// # Errors
    ///
    /// Returns [`crate::CommandError::EmptyDirtySummary`] when `summary` is empty.
    pub fn commit_result(
        &mut self,
        summary: impl Into<String>,
        result: &CommandResult,
    ) -> CommandModelResult<Option<UndoRedoRecord>> {
        if !result.is_success() || result.changes().is_empty() {
            return Ok(None);
        }
        let Some(target_mode) = result.target_mode() else {
            return Ok(None);
        };
        let group_id = self.next_group_id();
        let changes = result
            .changes()
            .iter()
            .cloned()
            .map(|change| change.with_undo_group(group_id));
        let record = UndoRedoRecord::new(group_id, target_mode, summary, changes)?;
        self.undo_stack.push(record.clone());
        self.redo_stack.clear();
        Ok(Some(record))
    }

    /// Moves the most recent undo record to the redo stack and returns it.
    #[must_use]
    pub fn pop_undo(&mut self) -> Option<UndoRedoRecord> {
        let record = self.undo_stack.pop()?;
        self.redo_stack.push(record.clone());
        Some(record)
    }

    /// Moves the most recent redo record back to the undo stack and returns it.
    #[must_use]
    pub fn pop_redo(&mut self) -> Option<UndoRedoRecord> {
        let record = self.redo_stack.pop()?;
        self.undo_stack.push(record.clone());
        Some(record)
    }

    /// Returns undo records in oldest-to-newest order.
    #[must_use]
    pub fn undo_stack(&self) -> &[UndoRedoRecord] {
        &self.undo_stack
    }

    /// Returns redo records in oldest-to-newest order.
    #[must_use]
    pub fn redo_stack(&self) -> &[UndoRedoRecord] {
        &self.redo_stack
    }

    /// Returns whether no undo or redo records are stored.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.undo_stack.is_empty() && self.redo_stack.is_empty()
    }

    fn next_group_id(&mut self) -> UndoGroupId {
        let group_id = UndoGroupId::new(self.next_undo_group_id);
        self.next_undo_group_id = self
            .next_undo_group_id
            .checked_add(1)
            .expect("command history exhausted undo group IDs");
        group_id
    }
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new()
    }
}

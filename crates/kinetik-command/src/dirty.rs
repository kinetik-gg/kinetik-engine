use crate::{CommandHistory, CommandTargetMode, UndoGroupId, UndoRedoRecord};

/// Read-only dirty-state explanation derived from command history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirtyStateExplanation {
    documents: Vec<DirtyDocumentExplanation>,
    changes: Vec<DirtyChangeExplanation>,
}

impl DirtyStateExplanation {
    /// Creates a dirty-state explanation from committed undo history.
    #[must_use]
    pub fn from_history(history: &CommandHistory) -> Self {
        let mut documents = Vec::new();
        let mut changes = Vec::new();

        for record in history.undo_stack() {
            push_record_explanations(record, &mut documents, &mut changes);
        }

        Self { documents, changes }
    }

    /// Returns whether the explanation has no dirty documents or changes.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.documents.is_empty() && self.changes.is_empty()
    }

    /// Returns dirty document explanations in deterministic first-seen order.
    #[must_use]
    pub fn documents(&self) -> &[DirtyDocumentExplanation] {
        &self.documents
    }

    /// Returns dirty change explanations in deterministic command history order.
    #[must_use]
    pub fn changes(&self) -> &[DirtyChangeExplanation] {
        &self.changes
    }
}

/// Dirty-state explanation for one affected source document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirtyDocumentExplanation {
    document_path: String,
    summaries: Vec<String>,
}

impl DirtyDocumentExplanation {
    /// Returns the affected workspace-relative document path.
    #[must_use]
    pub fn document_path(&self) -> &str {
        &self.document_path
    }

    /// Returns summaries that explain why this document is dirty.
    #[must_use]
    pub fn summaries(&self) -> &[String] {
        &self.summaries
    }
}

/// Dirty-state explanation for one committed semantic change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirtyChangeExplanation {
    undo_group: UndoGroupId,
    target_mode: CommandTargetMode,
    command_summary: String,
    change_summary: String,
    affected_documents: Vec<String>,
}

impl DirtyChangeExplanation {
    /// Returns the undo group that produced this dirty change.
    #[must_use]
    pub const fn undo_group(&self) -> UndoGroupId {
        self.undo_group
    }

    /// Returns the target mode for the command that produced this change.
    #[must_use]
    pub const fn target_mode(&self) -> CommandTargetMode {
        self.target_mode
    }

    /// Returns the user-facing command summary.
    #[must_use]
    pub fn command_summary(&self) -> &str {
        &self.command_summary
    }

    /// Returns the user-facing semantic change summary.
    #[must_use]
    pub fn change_summary(&self) -> &str {
        &self.change_summary
    }

    /// Returns affected source documents in deterministic record order.
    #[must_use]
    pub fn affected_documents(&self) -> &[String] {
        &self.affected_documents
    }
}

fn push_record_explanations(
    record: &UndoRedoRecord,
    documents: &mut Vec<DirtyDocumentExplanation>,
    changes: &mut Vec<DirtyChangeExplanation>,
) {
    for change in record.changes() {
        let affected_documents = change.affected_documents().to_vec();
        for document_path in &affected_documents {
            push_document_summary(documents, document_path, change.dirty_summary());
        }
        changes.push(DirtyChangeExplanation {
            undo_group: record.group_id(),
            target_mode: record.target_mode(),
            command_summary: record.summary().to_owned(),
            change_summary: change.dirty_summary().to_owned(),
            affected_documents,
        });
    }
}

fn push_document_summary(
    documents: &mut Vec<DirtyDocumentExplanation>,
    document_path: &str,
    summary: &str,
) {
    if let Some(document) = documents
        .iter_mut()
        .find(|document| document.document_path == document_path)
    {
        document.summaries.push(summary.to_owned());
        return;
    }

    documents.push(DirtyDocumentExplanation {
        document_path: document_path.to_owned(),
        summaries: vec![summary.to_owned()],
    });
}

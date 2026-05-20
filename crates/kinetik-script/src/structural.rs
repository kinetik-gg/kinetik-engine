use kinetik_core::{InstanceId, ResourceId};

use crate::{ScriptAttachmentId, ScriptAttachmentTarget};

/// Minimal script-originated property value for queued structural requests.
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptPropertyValue {
    /// Boolean value.
    Bool(bool),
    /// Signed integer value.
    Integer(i64),
    /// Floating-point value.
    Number(f64),
    /// String value.
    String(String),
    /// Runtime instance reference.
    Instance(InstanceId),
    /// Runtime resource reference.
    Resource(ResourceId),
}

/// Structural request kind emitted by script code.
#[derive(Debug, Clone, PartialEq)]
pub enum StructuralChangeKind {
    /// Create an instance of `class_name` under the optional parent.
    CreateInstance {
        /// Class name to create.
        class_name: String,
        /// Parent instance when known.
        parent: Option<InstanceId>,
    },
    /// Destroy an instance.
    DestroyInstance {
        /// Instance to destroy.
        instance: InstanceId,
    },
    /// Reparent an instance.
    ReparentInstance {
        /// Instance to reparent.
        instance: InstanceId,
        /// New parent instance when any.
        new_parent: Option<InstanceId>,
    },
    /// Attach a script asset to an instance.
    AttachScript {
        /// Target instance.
        target: ScriptAttachmentTarget,
        /// Script resource to attach.
        script_resource: ResourceId,
    },
    /// Detach a script attachment.
    DetachScript {
        /// Attachment to detach.
        attachment_id: ScriptAttachmentId,
    },
    /// Set a reflected property.
    SetProperty {
        /// Instance receiving the property change.
        instance: InstanceId,
        /// Canonical reflected property path.
        property_path: String,
        /// New script-originated value.
        value: ScriptPropertyValue,
    },
}

/// Script-originated structural change queued for a safe sync point.
#[derive(Debug, Clone, PartialEq)]
pub struct QueuedScriptChange {
    /// Script attachment that requested the change.
    pub requested_by: ScriptAttachmentId,
    /// Requested structural change.
    pub kind: StructuralChangeKind,
}

impl QueuedScriptChange {
    /// Creates a queued script change.
    #[must_use]
    pub const fn new(requested_by: ScriptAttachmentId, kind: StructuralChangeKind) -> Self {
        Self { requested_by, kind }
    }
}

/// FIFO queue for script-originated structural changes.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ScriptChangeQueue {
    changes: Vec<QueuedScriptChange>,
}

impl ScriptChangeQueue {
    /// Creates an empty script change queue.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            changes: Vec::new(),
        }
    }

    /// Queues a script-originated structural change.
    pub fn push(&mut self, change: QueuedScriptChange) {
        self.changes.push(change);
    }

    /// Returns queued changes without applying them.
    #[must_use]
    pub fn pending(&self) -> &[QueuedScriptChange] {
        &self.changes
    }

    /// Drains queued changes for a runtime-owned safe sync point.
    pub fn drain(&mut self) -> impl Iterator<Item = QueuedScriptChange> + '_ {
        self.changes.drain(..)
    }
}

#[cfg(test)]
mod tests {
    use kinetik_core::{InstanceId, ResourceId, ScriptId};

    use super::{QueuedScriptChange, ScriptChangeQueue, ScriptPropertyValue, StructuralChangeKind};

    #[test]
    fn structural_changes_are_queued_until_safe_sync_point() {
        let requested_by = ScriptId::new(7);
        let instance = InstanceId::new(11);
        let mut queue = ScriptChangeQueue::new();

        queue.push(QueuedScriptChange::new(
            requested_by,
            StructuralChangeKind::SetProperty {
                instance,
                property_path: "Health".to_owned(),
                value: ScriptPropertyValue::Integer(10),
            },
        ));
        queue.push(QueuedScriptChange::new(
            requested_by,
            StructuralChangeKind::AttachScript {
                target: crate::ScriptAttachmentTarget::runtime_only(instance),
                script_resource: ResourceId::new(15),
            },
        ));

        assert_eq!(queue.pending().len(), 2);

        let drained = queue.drain().collect::<Vec<_>>();
        assert_eq!(drained.len(), 2);
        assert!(queue.pending().is_empty());
    }
}

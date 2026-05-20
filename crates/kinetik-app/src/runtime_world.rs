use core::{fmt, num::NonZeroU64};

use kinetik_core::InstanceGuid;
use kinetik_scene::{Scene, SceneError, SceneInstanceDocument};

/// Runtime world ID.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RuntimeWorldId(NonZeroU64);

impl RuntimeWorldId {
    /// Creates a runtime world ID from a non-zero raw value.
    ///
    /// # Panics
    ///
    /// Panics when `raw` is zero.
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        let Some(raw) = NonZeroU64::new(raw) else {
            panic!("RuntimeWorldId raw value must be non-zero");
        };
        Self(raw)
    }

    /// Returns the raw non-zero ID value.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0.get()
    }
}

impl fmt::Display for RuntimeWorldId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RuntimeWorldId({})", self.raw())
    }
}

/// Runtime instance ID owned by a runtime world.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RuntimeInstanceId(NonZeroU64);

impl RuntimeInstanceId {
    /// Creates a runtime instance ID from a non-zero raw value.
    ///
    /// # Panics
    ///
    /// Panics when `raw` is zero.
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        let Some(raw) = NonZeroU64::new(raw) else {
            panic!("RuntimeInstanceId raw value must be non-zero");
        };
        Self(raw)
    }

    /// Returns the raw non-zero ID value.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0.get()
    }
}

impl fmt::Display for RuntimeInstanceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RuntimeInstanceId({})", self.raw())
    }
}

/// Runtime instance cloned from edit scene state or spawned during play.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeInstanceRecord {
    /// Runtime-only instance ID.
    pub id: RuntimeInstanceId,
    /// Stable edit GUID when this runtime instance came from saved edit state.
    pub edit_guid: Option<InstanceGuid>,
    /// Registered class name cloned from edit state.
    pub class_name: String,
    /// Runtime display name cloned from edit state.
    pub name: String,
    /// Runtime parent ID.
    pub parent: Option<RuntimeInstanceId>,
    /// Ordered runtime child IDs.
    pub children: Vec<RuntimeInstanceId>,
}

/// Result type for runtime world operations.
pub type RuntimeWorldResult<T> = Result<T, RuntimeWorldError>;

/// Errors returned by runtime world operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeWorldError {
    /// Runtime child spawn was requested under a missing parent.
    MissingParent {
        /// Missing parent ID.
        parent: RuntimeInstanceId,
    },
    /// Runtime instance ID was not present in the world.
    InvalidInstance {
        /// Invalid runtime instance ID.
        id: RuntimeInstanceId,
    },
    /// Runtime instance class name was empty.
    EmptyClassName,
    /// Runtime instance name was empty or contained a path separator.
    InvalidInstanceName {
        /// Invalid runtime instance name.
        name: String,
    },
    /// Runtime root cannot be despawned.
    CannotDespawnRoot {
        /// Root runtime instance ID.
        root: RuntimeInstanceId,
    },
}

impl fmt::Display for RuntimeWorldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingParent { parent } => {
                write!(f, "runtime parent instance is missing: {parent}")
            }
            Self::InvalidInstance { id } => write!(f, "runtime instance is missing: {id}"),
            Self::EmptyClassName => f.write_str("runtime instance class name must not be empty"),
            Self::InvalidInstanceName { name } => write!(
                f,
                "runtime instance name must be non-empty and must not contain '/': {name}"
            ),
            Self::CannotDespawnRoot { root } => {
                write!(f, "runtime root instance cannot be despawned: {root}")
            }
        }
    }
}

impl std::error::Error for RuntimeWorldError {}

/// Sandboxed runtime world derived from edit scene state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeWorld {
    id: RuntimeWorldId,
    instances: Vec<RuntimeInstanceRecord>,
    root: Option<RuntimeInstanceId>,
    next_instance_id: u64,
}

impl RuntimeWorld {
    /// Creates a runtime world clone from an edit scene.
    ///
    /// Runtime IDs are owned by the runtime world and edit GUIDs are retained
    /// only as provenance for saved instances.
    ///
    /// # Errors
    ///
    /// Returns [`SceneError`] when the edit scene cannot be exported as a scene
    /// document, such as when it has no root.
    pub fn clone_from_edit_scene(id: RuntimeWorldId, scene: &Scene) -> Result<Self, SceneError> {
        let document = scene.to_document()?;
        let mut world = Self {
            id,
            instances: Vec::new(),
            root: None,
            next_instance_id: 1,
        };
        let root = world.clone_document_instance(document.root, None);
        world.root = Some(root);
        Ok(world)
    }

    /// Returns this runtime world's ID.
    #[must_use]
    pub const fn id(&self) -> RuntimeWorldId {
        self.id
    }

    /// Returns the root runtime instance ID when present.
    #[must_use]
    pub const fn root_id(&self) -> Option<RuntimeInstanceId> {
        self.root
    }

    /// Returns all runtime instances in deterministic parent-before-child order.
    #[must_use]
    pub fn instances(&self) -> &[RuntimeInstanceRecord] {
        &self.instances
    }

    /// Returns a runtime instance by runtime ID.
    #[must_use]
    pub fn get(&self, id: RuntimeInstanceId) -> Option<&RuntimeInstanceRecord> {
        self.instances.iter().find(|instance| instance.id == id)
    }

    /// Returns the runtime instance cloned from `edit_guid`, if any.
    #[must_use]
    pub fn runtime_id_for_edit_guid(&self, edit_guid: InstanceGuid) -> Option<RuntimeInstanceId> {
        self.instances
            .iter()
            .find(|instance| instance.edit_guid == Some(edit_guid))
            .map(|instance| instance.id)
    }

    /// Spawns a runtime-only child under `parent`.
    ///
    /// Runtime-only instances receive runtime identity but no saved edit GUID.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeWorldError`] when the parent is missing or the requested
    /// class/name is invalid.
    pub fn spawn_runtime_child(
        &mut self,
        parent: RuntimeInstanceId,
        class_name: impl Into<String>,
        name: impl Into<String>,
    ) -> RuntimeWorldResult<RuntimeInstanceId> {
        let class_name = class_name.into();
        let name = name.into();
        validate_runtime_class_name(&class_name)?;
        validate_runtime_instance_name(&name)?;

        let parent_index = self
            .instance_index(parent)
            .ok_or(RuntimeWorldError::MissingParent { parent })?;
        let id = self.next_runtime_instance_id();
        self.instances.push(RuntimeInstanceRecord {
            id,
            edit_guid: None,
            class_name,
            name,
            parent: Some(parent),
            children: Vec::new(),
        });
        self.instances[parent_index].children.push(id);
        Ok(id)
    }

    /// Despawns a runtime instance and all of its runtime children.
    ///
    /// The returned IDs are ordered parent-before-child in the removed subtree.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeWorldError`] when the instance is missing or is the
    /// runtime root.
    pub fn despawn_runtime_subtree(
        &mut self,
        instance: RuntimeInstanceId,
    ) -> RuntimeWorldResult<Vec<RuntimeInstanceId>> {
        if self.root == Some(instance) {
            return Err(RuntimeWorldError::CannotDespawnRoot { root: instance });
        }
        let record = self
            .get(instance)
            .ok_or(RuntimeWorldError::InvalidInstance { id: instance })?;
        let parent = record.parent;
        let mut removed = Vec::new();
        self.collect_subtree_ids(instance, &mut removed);

        if let Some(parent) = parent {
            if let Some(parent_index) = self.instance_index(parent) {
                self.instances[parent_index]
                    .children
                    .retain(|child| *child != instance);
            }
        }
        self.instances
            .retain(|record| !removed.contains(&record.id));
        Ok(removed)
    }

    fn clone_document_instance(
        &mut self,
        document: SceneInstanceDocument,
        parent: Option<RuntimeInstanceId>,
    ) -> RuntimeInstanceId {
        let id = self.next_runtime_instance_id();
        let children = document.children;
        let index = self.instances.len();
        self.instances.push(RuntimeInstanceRecord {
            id,
            edit_guid: Some(document.guid),
            class_name: document.class_name,
            name: document.name,
            parent,
            children: Vec::new(),
        });
        let child_ids = children
            .into_iter()
            .map(|child| self.clone_document_instance(child, Some(id)))
            .collect();
        self.instances[index].children = child_ids;
        id
    }

    fn next_runtime_instance_id(&mut self) -> RuntimeInstanceId {
        let id = RuntimeInstanceId::new(self.next_instance_id);
        self.next_instance_id += 1;
        id
    }

    fn instance_index(&self, id: RuntimeInstanceId) -> Option<usize> {
        self.instances.iter().position(|instance| instance.id == id)
    }

    fn collect_subtree_ids(&self, id: RuntimeInstanceId, output: &mut Vec<RuntimeInstanceId>) {
        output.push(id);
        let Some(instance) = self.get(id) else {
            return;
        };
        for child in &instance.children {
            self.collect_subtree_ids(*child, output);
        }
    }
}

fn validate_runtime_class_name(class_name: &str) -> RuntimeWorldResult<()> {
    if class_name.trim().is_empty() {
        return Err(RuntimeWorldError::EmptyClassName);
    }
    Ok(())
}

fn validate_runtime_instance_name(name: &str) -> RuntimeWorldResult<()> {
    if name.trim().is_empty() || name.contains('/') {
        return Err(RuntimeWorldError::InvalidInstanceName {
            name: name.to_owned(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests;

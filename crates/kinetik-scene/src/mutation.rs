use kinetik_core::InstanceId;
use kinetik_reflect::PropertyValue;

use crate::scene::validate_instance_name;
use crate::{Scene, SceneError, SceneResult};

/// Scene-local structural mutation batch.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SceneMutationQueue {
    mutations: Vec<SceneMutation>,
}

impl SceneMutationQueue {
    /// Creates an empty mutation queue.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            mutations: Vec::new(),
        }
    }

    /// Queues a structural mutation.
    pub fn push(&mut self, mutation: SceneMutation) {
        self.mutations.push(mutation);
    }

    /// Queues root instance creation.
    pub fn create_root(&mut self, class_name: impl Into<String>, name: impl Into<String>) {
        self.push(SceneMutation::Create {
            parent: None,
            class_name: class_name.into(),
            name: name.into(),
        });
    }

    /// Queues child instance creation.
    pub fn create_child(
        &mut self,
        parent: InstanceId,
        class_name: impl Into<String>,
        name: impl Into<String>,
    ) {
        self.push(SceneMutation::Create {
            parent: Some(parent),
            class_name: class_name.into(),
            name: name.into(),
        });
    }

    /// Queues instance deletion.
    pub fn delete(&mut self, id: InstanceId) {
        self.push(SceneMutation::Delete { id });
    }

    /// Queues instance rename.
    pub fn rename(&mut self, id: InstanceId, name: impl Into<String>) {
        self.push(SceneMutation::Rename {
            id,
            name: name.into(),
        });
    }

    /// Queues instance reparenting.
    pub fn reparent(&mut self, id: InstanceId, new_parent: InstanceId) {
        self.push(SceneMutation::Reparent { id, new_parent });
    }

    /// Queues subtree duplication under a new parent.
    pub fn duplicate(&mut self, id: InstanceId, new_parent: InstanceId) {
        self.push(SceneMutation::Duplicate { id, new_parent });
    }

    /// Returns queued mutations in deterministic apply order.
    #[must_use]
    pub fn mutations(&self) -> &[SceneMutation] {
        &self.mutations
    }

    /// Returns the number of queued mutations.
    #[must_use]
    pub fn len(&self) -> usize {
        self.mutations.len()
    }

    /// Returns whether the queue is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.mutations.is_empty()
    }
}

/// Scene-local structural mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneMutation {
    /// Create a root or child instance.
    Create {
        /// Optional parent. `None` creates the scene root.
        parent: Option<InstanceId>,
        /// Registered class name.
        class_name: String,
        /// Instance name.
        name: String,
    },
    /// Delete an instance and its descendants.
    Delete {
        /// Instance to delete.
        id: InstanceId,
    },
    /// Rename an instance.
    Rename {
        /// Instance to rename.
        id: InstanceId,
        /// New instance name.
        name: String,
    },
    /// Move an instance under a new parent.
    Reparent {
        /// Instance to move.
        id: InstanceId,
        /// Target parent instance.
        new_parent: InstanceId,
    },
    /// Duplicate an instance subtree under a target parent.
    Duplicate {
        /// Root of the source subtree to duplicate.
        id: InstanceId,
        /// Target parent for the duplicated subtree root.
        new_parent: InstanceId,
    },
}

/// Result produced by applying a scene structural mutation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneMutationResult {
    /// Instance was created.
    Created {
        /// Created instance ID.
        id: InstanceId,
    },
    /// Instance and descendants were deleted.
    Deleted {
        /// Deleted root instance ID.
        id: InstanceId,
        /// Deleted instance IDs in deterministic parent-before-child order.
        deleted_ids: Vec<InstanceId>,
    },
    /// Instance was renamed.
    Renamed {
        /// Renamed instance ID.
        id: InstanceId,
    },
    /// Instance was reparented.
    Reparented {
        /// Reparented instance ID.
        id: InstanceId,
        /// Previous parent.
        old_parent: Option<InstanceId>,
        /// New parent.
        new_parent: InstanceId,
    },
    /// Instance subtree was duplicated.
    Duplicated {
        /// Source subtree root ID.
        source_id: InstanceId,
        /// Duplicated subtree root ID.
        new_root_id: InstanceId,
        /// Duplicated IDs in deterministic parent-before-child order.
        duplicated_ids: Vec<InstanceId>,
    },
}

impl Scene {
    /// Applies queued structural mutations atomically.
    ///
    /// Mutations are applied to a cloned scene in queue order first. The
    /// original scene is replaced only when every mutation validates and
    /// applies successfully.
    ///
    /// # Errors
    ///
    /// Returns [`SceneError`] from the first invalid queued mutation without
    /// partially mutating the original scene.
    pub fn apply_mutations(
        &mut self,
        queue: SceneMutationQueue,
    ) -> SceneResult<Vec<SceneMutationResult>> {
        let mut draft = self.clone();
        let mut results = Vec::with_capacity(queue.len());

        for mutation in queue.mutations {
            let result = draft.apply_mutation(mutation)?;
            results.push(result);
        }

        *self = draft;
        Ok(results)
    }

    fn apply_mutation(&mut self, mutation: SceneMutation) -> SceneResult<SceneMutationResult> {
        match mutation {
            SceneMutation::Create {
                parent,
                class_name,
                name,
            } => {
                let id = match parent {
                    Some(parent) => self.add_child(parent, class_name, name)?,
                    None => self.add_root(class_name, name)?,
                };
                Ok(SceneMutationResult::Created { id })
            }
            SceneMutation::Delete { id } => {
                let deleted_ids = self.delete_instance(id)?;
                Ok(SceneMutationResult::Deleted { id, deleted_ids })
            }
            SceneMutation::Rename { id, name } => {
                self.rename_instance(id, name)?;
                Ok(SceneMutationResult::Renamed { id })
            }
            SceneMutation::Reparent { id, new_parent } => {
                let old_parent = self.reparent_instance(id, new_parent)?;
                Ok(SceneMutationResult::Reparented {
                    id,
                    old_parent,
                    new_parent,
                })
            }
            SceneMutation::Duplicate { id, new_parent } => {
                let (new_root_id, duplicated_ids) = self.duplicate_subtree(id, new_parent)?;
                Ok(SceneMutationResult::Duplicated {
                    source_id: id,
                    new_root_id,
                    duplicated_ids,
                })
            }
        }
    }

    fn duplicate_subtree(
        &mut self,
        id: InstanceId,
        new_parent: InstanceId,
    ) -> SceneResult<(InstanceId, Vec<InstanceId>)> {
        self.index_of(id)?;
        self.index_of(new_parent)?;
        if Some(id) == self.root {
            return Err(SceneError::CannotDuplicateRoot { root_id: id });
        }

        let mut duplicated_ids = Vec::new();
        let new_root_id = self.duplicate_subtree_inner(id, new_parent, &mut duplicated_ids)?;
        self.advance_transform_revision();
        Ok((new_root_id, duplicated_ids))
    }

    fn duplicate_subtree_inner(
        &mut self,
        source_id: InstanceId,
        parent: InstanceId,
        duplicated_ids: &mut Vec<InstanceId>,
    ) -> SceneResult<InstanceId> {
        let source = self.get(source_id)?.clone();
        let new_id = self.next_instance_id();
        let new_guid = self.next_instance_guid();

        self.instances.push(crate::InstanceRecord {
            id: new_id,
            guid: new_guid,
            class_name: source.class_name,
            name: source.name,
            parent: Some(parent),
            children: Vec::new(),
            properties: source.properties,
        });
        duplicated_ids.push(new_id);

        let parent_index = self.index_of(parent)?;
        self.instances[parent_index].children.push(new_id);

        for child_id in source.children {
            self.duplicate_subtree_inner(child_id, new_id, duplicated_ids)?;
        }

        Ok(new_id)
    }
    fn delete_instance(&mut self, id: InstanceId) -> SceneResult<Vec<InstanceId>> {
        let index = self.index_of(id)?;
        if Some(id) == self.root {
            return Err(SceneError::CannotDeleteRoot { root_id: id });
        }

        let parent = self.instances[index].parent;
        let mut deleted_ids = Vec::new();
        self.collect_subtree_ids(id, &mut deleted_ids)?;

        if let Some(parent) = parent {
            let parent_index = self.index_of(parent)?;
            self.instances[parent_index]
                .children
                .retain(|child_id| *child_id != id);
        }

        self.instances
            .retain(|instance| !deleted_ids.contains(&instance.id));
        self.advance_transform_revision();
        Ok(deleted_ids)
    }

    fn rename_instance(&mut self, id: InstanceId, name: String) -> SceneResult<()> {
        validate_instance_name(&name)?;
        let index = self.index_of(id)?;
        self.instances[index].name.clone_from(&name);
        if let Some(PropertyValue::String(stored_name)) =
            self.instances[index].properties.get_mut("Name")
        {
            *stored_name = name;
        }
        Ok(())
    }

    fn reparent_instance(
        &mut self,
        id: InstanceId,
        new_parent: InstanceId,
    ) -> SceneResult<Option<InstanceId>> {
        let index = self.index_of(id)?;
        self.index_of(new_parent)?;
        if Some(id) == self.root {
            return Err(SceneError::CannotReparentRoot { root_id: id });
        }
        if id == new_parent || self.is_descendant_of(new_parent, id)? {
            return Err(SceneError::ReparentCycle { id, new_parent });
        }

        let old_parent = self.instances[index].parent;
        if old_parent == Some(new_parent) {
            return Ok(old_parent);
        }

        if let Some(old_parent) = old_parent {
            let old_parent_index = self.index_of(old_parent)?;
            self.instances[old_parent_index]
                .children
                .retain(|child_id| *child_id != id);
        }

        let new_parent_index = self.index_of(new_parent)?;
        self.instances[new_parent_index].children.push(id);
        let index = self.index_of(id)?;
        self.instances[index].parent = Some(new_parent);
        self.advance_transform_revision();
        Ok(old_parent)
    }
    fn collect_subtree_ids(&self, id: InstanceId, output: &mut Vec<InstanceId>) -> SceneResult<()> {
        let instance = self.get(id)?;
        output.push(id);
        for child_id in &instance.children {
            self.collect_subtree_ids(*child_id, output)?;
        }
        Ok(())
    }

    fn is_descendant_of(&self, id: InstanceId, ancestor: InstanceId) -> SceneResult<bool> {
        let mut current = self.get(id)?.parent;
        while let Some(current_id) = current {
            if current_id == ancestor {
                return Ok(true);
            }
            current = self.get(current_id)?.parent;
        }
        Ok(false)
    }
}

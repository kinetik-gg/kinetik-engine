//! Runtime app loop and world orchestration contracts for Kinetik.

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
}

/// Returns the crate name for smoke tests and early integration checks.
#[must_use]
pub const fn crate_name() -> &'static str {
    "kinetik-app"
}

#[cfg(test)]
mod tests {
    use super::*;
    use kinetik_scene::ROOT_CLASS_NAME;

    #[test]
    fn exposes_crate_name() {
        assert_eq!(crate_name(), "kinetik-app");
    }

    #[test]
    fn runtime_world_clone_preserves_edit_guid_mapping() {
        let scene = kinetik_scene::Scene::default_scene().unwrap();
        let document = scene.to_document().unwrap();
        let world = RuntimeWorld::clone_from_edit_scene(RuntimeWorldId::new(1), &scene).unwrap();

        assert_eq!(world.id(), RuntimeWorldId::new(1));
        assert_eq!(world.root_id(), Some(RuntimeInstanceId::new(1)));
        assert_eq!(
            world.runtime_id_for_edit_guid(document.root.guid),
            Some(RuntimeInstanceId::new(1))
        );
        assert_eq!(
            world.get(RuntimeInstanceId::new(1)).unwrap().class_name,
            ROOT_CLASS_NAME
        );
        assert_eq!(
            world.get(RuntimeInstanceId::new(2)).unwrap().parent,
            Some(RuntimeInstanceId::new(1))
        );
    }

    #[test]
    fn runtime_world_clone_uses_deterministic_parent_before_child_order() {
        let scene = kinetik_scene::Scene::default_scene().unwrap();
        let world = RuntimeWorld::clone_from_edit_scene(RuntimeWorldId::new(7), &scene).unwrap();
        let names: Vec<&str> = world
            .instances()
            .iter()
            .map(|instance| instance.name.as_str())
            .collect();

        assert_eq!(
            names,
            vec![
                "Game",
                "Workspace",
                "Prefabs",
                "Scripts",
                "UI",
                "Lighting",
                "Audio",
                "Physics",
                "Assets",
                "Packages",
            ]
        );
        assert_eq!(
            world.get(RuntimeInstanceId::new(1)).unwrap().children,
            (2..=10).map(RuntimeInstanceId::new).collect::<Vec<_>>()
        );
    }

    #[test]
    fn runtime_world_clone_does_not_require_edit_instance_ids() {
        let mut scene = kinetik_scene::Scene::new();
        let root = scene.add_root(ROOT_CLASS_NAME, "Game").unwrap();
        let node = scene.add_child(root, "Node3D", "Node").unwrap();
        let edit_root_raw = root.raw();
        let edit_node_raw = node.raw();

        let world = RuntimeWorld::clone_from_edit_scene(RuntimeWorldId::new(3), &scene).unwrap();

        assert_eq!(world.root_id().unwrap().raw(), 1);
        assert_eq!(world.get(RuntimeInstanceId::new(2)).unwrap().name, "Node");
        assert_eq!(edit_root_raw, 1);
        assert_eq!(edit_node_raw, 2);
        assert_ne!(
            core::any::type_name::<RuntimeInstanceId>(),
            core::any::type_name::<kinetik_core::InstanceId>()
        );
    }

    #[test]
    fn runtime_world_clone_requires_edit_scene_root() {
        let scene = kinetik_scene::Scene::new();

        assert_eq!(
            RuntimeWorld::clone_from_edit_scene(RuntimeWorldId::new(1), &scene).unwrap_err(),
            SceneError::MissingRoot
        );
    }
}

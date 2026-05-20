//! Explorer hierarchy projection for the active scene.

use kinetik_core::{InstanceGuid, InstanceId};
use kinetik_scene::{Scene, SceneResult};

/// One row in the Explorer hierarchy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExplorerRow {
    /// Instance ID in the active edit scene.
    pub id: InstanceId,
    /// Stable edit-world instance GUID.
    pub guid: InstanceGuid,
    /// Parent instance ID when this row is not the root.
    pub parent: Option<InstanceId>,
    /// Registered class name.
    pub class_name: String,
    /// Instance display name.
    pub name: String,
    /// Absolute scene path.
    pub scene_path: String,
    /// Depth from the scene root.
    pub depth: usize,
    /// Child instance IDs in deterministic display order.
    pub children: Vec<InstanceId>,
}

/// Deterministic Explorer hierarchy snapshot.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExplorerSnapshot {
    rows: Vec<ExplorerRow>,
}

impl ExplorerSnapshot {
    /// Builds an Explorer snapshot from `scene`.
    ///
    /// # Errors
    ///
    /// Returns [`kinetik_scene::SceneError`] when the scene root or hierarchy is
    /// internally inconsistent.
    pub fn from_scene(scene: &Scene) -> SceneResult<Self> {
        let mut rows = Vec::new();
        if let Some(root_id) = scene.root_id() {
            push_row(scene, root_id, 0, &mut rows)?;
        }
        Ok(Self { rows })
    }

    /// Returns Explorer rows in parent-before-child display order.
    #[must_use]
    pub fn rows(&self) -> &[ExplorerRow] {
        &self.rows
    }

    /// Returns whether the snapshot has no rows.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Finds a row by instance ID.
    #[must_use]
    pub fn row_by_id(&self, id: InstanceId) -> Option<&ExplorerRow> {
        self.rows.iter().find(|row| row.id == id)
    }
}

fn push_row(
    scene: &Scene,
    id: InstanceId,
    depth: usize,
    rows: &mut Vec<ExplorerRow>,
) -> SceneResult<()> {
    let instance = scene.get(id)?;
    let children = instance.children.clone();
    rows.push(ExplorerRow {
        id,
        guid: instance.guid,
        parent: instance.parent,
        class_name: instance.class_name.clone(),
        name: instance.name.clone(),
        scene_path: scene.path(id)?,
        depth,
        children: children.clone(),
    });

    for child in children {
        push_row(scene, child, depth + 1, rows)?;
    }

    Ok(())
}

//! Scene and instance graph contracts for Kinetik.

use kinetik_core::{InstanceId, KinetikError, KinetikResult};

/// Minimal instance record used by the initial scaffold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceRecord {
    /// Runtime instance ID.
    pub id: InstanceId,
    /// Human-readable instance name.
    pub name: String,
}

/// Minimal scene placeholder.
#[derive(Debug, Default)]
pub struct Scene {
    instances: Vec<InstanceRecord>,
}

impl Scene {
    /// Creates an empty scene.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            instances: Vec::new(),
        }
    }

    /// Adds a root-level instance and returns its ID.
    pub fn add_root(&mut self, name: impl Into<String>) -> InstanceId {
        let id = InstanceId::new(self.instances.len() as u64 + 1);
        self.instances.push(InstanceRecord {
            id,
            name: name.into(),
        });
        id
    }

    /// Finds an instance by ID.
    ///
    /// # Errors
    ///
    /// Returns [`KinetikError::InvalidHandle`] when the instance ID is not present in this scene.
    pub fn get(&self, id: InstanceId) -> KinetikResult<&InstanceRecord> {
        self.instances
            .iter()
            .find(|instance| instance.id == id)
            .ok_or(KinetikError::InvalidHandle {
                kind: "InstanceId",
                id: id.raw(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_get_root_instance() {
        let mut scene = Scene::new();
        let id = scene.add_root("Workspace");
        assert_eq!(scene.get(id).unwrap().name, "Workspace");
    }
}

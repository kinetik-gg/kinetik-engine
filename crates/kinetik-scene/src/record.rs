use std::collections::BTreeMap;

use kinetik_core::{InstanceGuid, InstanceId};
use kinetik_reflect::PropertyValue;

/// Instance record stored by a scene.
#[derive(Debug, Clone, PartialEq)]
pub struct InstanceRecord {
    /// Runtime instance ID.
    pub id: InstanceId,
    /// Stable instance GUID for edit-world identity.
    pub guid: InstanceGuid,
    /// Registered instance class name.
    pub class_name: String,
    /// Human-readable instance name.
    pub name: String,
    /// Parent runtime instance ID.
    pub parent: Option<InstanceId>,
    /// Ordered child runtime instance IDs.
    pub children: Vec<InstanceId>,
    /// Reflected property values keyed by canonical property path.
    pub properties: BTreeMap<String, PropertyValue>,
}

use std::collections::{BTreeMap, BTreeSet};

use kinetik_core::{
    Aabb, Color, InstanceGuid, InstanceId, Quat, Rect, ResourceId, Transform, Vec2, Vec3, Vec4,
};
use kinetik_reflect::{AssetReferenceValue, PropertyValue};
use serde::{Deserialize, Serialize};

use crate::scene::{default_properties_for_class, property_value_error, validate_instance_name};
use crate::{InstanceClassRegistry, InstanceRecord, Scene, SceneError, SceneResult};

/// Dependency-free `.knscene` document contract.
#[derive(Debug, Clone, PartialEq)]
pub struct SceneDocument {
    /// Root serialized instance tree.
    pub root: SceneInstanceDocument,
}

impl SceneDocument {
    /// Creates a scene document from a root instance tree.
    #[must_use]
    pub const fn new(root: SceneInstanceDocument) -> Self {
        Self { root }
    }

    /// Serializes this scene document to deterministic `.knscene` RON text.
    ///
    /// # Errors
    ///
    /// Returns [`SceneError::Serialization`] if RON writing fails.
    pub fn to_ron_string(&self) -> SceneResult<String> {
        let contract = SceneRon::from_document(self);
        let config = ron::ser::PrettyConfig::new();
        ron::ser::to_string_pretty(&contract, config).map_err(serialization_error)
    }

    /// Parses a scene document from `.knscene` RON text.
    ///
    /// # Errors
    ///
    /// Returns [`SceneError::Serialization`] when parsing fails.
    pub fn from_ron_str(source: &str) -> SceneResult<Self> {
        let contract = ron::from_str::<SceneRon>(source).map_err(serialization_error)?;
        contract.into_document()
    }
}

/// Serialized instance tree contract used by scene and prefab documents.
#[derive(Debug, Clone, PartialEq)]
pub struct SceneInstanceDocument {
    /// Stable serialized instance identity.
    pub guid: InstanceGuid,
    /// Registered class name.
    pub class_name: String,
    /// Human-readable instance name.
    pub name: String,
    /// Reflected property values keyed by canonical property path.
    pub properties: BTreeMap<String, PropertyValue>,
    /// Ordered child instance documents.
    pub children: Vec<SceneInstanceDocument>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct SceneRon {
    root: SceneInstanceRon,
}

impl SceneRon {
    fn from_document(document: &SceneDocument) -> Self {
        Self {
            root: SceneInstanceRon::from_document(&document.root),
        }
    }

    fn into_document(self) -> SceneResult<SceneDocument> {
        Ok(SceneDocument {
            root: self.root.into_document()?,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct SceneInstanceRon {
    guid: u64,
    class_name: String,
    name: String,
    properties: BTreeMap<String, PropertyValueRon>,
    children: Vec<SceneInstanceRon>,
}

impl SceneInstanceRon {
    fn from_document(document: &SceneInstanceDocument) -> Self {
        Self {
            guid: document.guid.raw(),
            class_name: document.class_name.clone(),
            name: document.name.clone(),
            properties: document
                .properties
                .iter()
                .map(|(path, value)| (path.clone(), PropertyValueRon::from_value(value)))
                .collect(),
            children: document.children.iter().map(Self::from_document).collect(),
        }
    }

    fn into_document(self) -> SceneResult<SceneInstanceDocument> {
        let properties = self
            .properties
            .into_iter()
            .map(|(path, value)| Ok((path, value.into_value()?)))
            .collect::<SceneResult<BTreeMap<_, _>>>()?;
        let children = self
            .children
            .into_iter()
            .map(Self::into_document)
            .collect::<SceneResult<Vec<_>>>()?;
        Ok(SceneInstanceDocument {
            guid: InstanceGuid::new(non_zero_raw("instance guid", self.guid)?),
            class_name: self.class_name,
            name: self.name,
            properties,
            children,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
enum PropertyValueRon {
    String(String),
    Bool(bool),
    F32(f32),
    Vec2(Vec2Ron),
    Vec3(Vec3Ron),
    Vec4(Vec4Ron),
    Quat(QuatRon),
    Color(ColorRon),
    Transform(TransformRon),
    Rect(RectRon),
    Aabb(AabbRon),
    InstanceId(u64),
    ResourceId(u64),
    AssetReference(AssetReferenceRon),
}

impl PropertyValueRon {
    fn from_value(value: &PropertyValue) -> Self {
        match value {
            PropertyValue::String(value) => Self::String(value.clone()),
            PropertyValue::Bool(value) => Self::Bool(*value),
            PropertyValue::F32(value) => Self::F32(*value),
            PropertyValue::Vec2(value) => Self::Vec2(Vec2Ron::from_vec2(*value)),
            PropertyValue::Vec3(value) => Self::Vec3(Vec3Ron::from_vec3(*value)),
            PropertyValue::Vec4(value) => Self::Vec4(Vec4Ron::from_vec4(*value)),
            PropertyValue::Quat(value) => Self::Quat(QuatRon::from_quat(*value)),
            PropertyValue::Color(value) => Self::Color(ColorRon::from_color(*value)),
            PropertyValue::Transform(value) => {
                Self::Transform(TransformRon::from_transform(*value))
            }
            PropertyValue::Rect(value) => Self::Rect(RectRon::from_rect(*value)),
            PropertyValue::Aabb(value) => Self::Aabb(AabbRon::from_aabb(*value)),
            PropertyValue::InstanceId(value) => Self::InstanceId(value.raw()),
            PropertyValue::ResourceId(value) => Self::ResourceId(value.raw()),
            PropertyValue::AssetReference(value) => {
                Self::AssetReference(AssetReferenceRon::from_reference(value))
            }
        }
    }

    fn into_value(self) -> SceneResult<PropertyValue> {
        let value = match self {
            Self::String(value) => PropertyValue::String(value),
            Self::Bool(value) => PropertyValue::Bool(value),
            Self::F32(value) => PropertyValue::F32(value),
            Self::Vec2(value) => PropertyValue::Vec2(value.into_vec2()),
            Self::Vec3(value) => PropertyValue::Vec3(value.into_vec3()),
            Self::Vec4(value) => PropertyValue::Vec4(value.into_vec4()),
            Self::Quat(value) => PropertyValue::Quat(value.into_quat()),
            Self::Color(value) => PropertyValue::Color(value.into_color()),
            Self::Transform(value) => PropertyValue::Transform(value.into_transform()),
            Self::Rect(value) => PropertyValue::Rect(value.into_rect()),
            Self::Aabb(value) => PropertyValue::Aabb(value.into_aabb()),
            Self::InstanceId(value) => {
                PropertyValue::InstanceId(InstanceId::new(non_zero_raw("instance id", value)?))
            }
            Self::ResourceId(value) => {
                PropertyValue::ResourceId(ResourceId::new(non_zero_raw("resource id", value)?))
            }
            Self::AssetReference(value) => PropertyValue::AssetReference(value.into_reference()?),
        };
        Ok(value)
    }
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
struct Vec2Ron {
    x: f32,
    y: f32,
}

impl Vec2Ron {
    const fn from_vec2(value: Vec2) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }

    const fn into_vec2(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
struct Vec3Ron {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3Ron {
    const fn from_vec3(value: Vec3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }

    const fn into_vec3(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
struct Vec4Ron {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

impl Vec4Ron {
    const fn from_vec4(value: Vec4) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
            w: value.w,
        }
    }

    const fn into_vec4(self) -> Vec4 {
        Vec4::new(self.x, self.y, self.z, self.w)
    }
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
struct QuatRon {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

impl QuatRon {
    const fn from_quat(value: Quat) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
            w: value.w,
        }
    }

    const fn into_quat(self) -> Quat {
        Quat::new(self.x, self.y, self.z, self.w)
    }
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
struct ColorRon {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl ColorRon {
    const fn from_color(value: Color) -> Self {
        Self {
            r: value.r,
            g: value.g,
            b: value.b,
            a: value.a,
        }
    }

    const fn into_color(self) -> Color {
        Color::new(self.r, self.g, self.b, self.a)
    }
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
struct TransformRon {
    position: Vec3Ron,
    rotation: QuatRon,
    scale: Vec3Ron,
}

impl TransformRon {
    const fn from_transform(value: Transform) -> Self {
        Self {
            position: Vec3Ron::from_vec3(value.position),
            rotation: QuatRon::from_quat(value.rotation),
            scale: Vec3Ron::from_vec3(value.scale),
        }
    }

    const fn into_transform(self) -> Transform {
        Transform::new(
            self.position.into_vec3(),
            self.rotation.into_quat(),
            self.scale.into_vec3(),
        )
    }
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
struct RectRon {
    min: Vec2Ron,
    size: Vec2Ron,
}

impl RectRon {
    const fn from_rect(value: Rect) -> Self {
        Self {
            min: Vec2Ron::from_vec2(value.min),
            size: Vec2Ron::from_vec2(value.size),
        }
    }

    const fn into_rect(self) -> Rect {
        Rect::new(self.min.into_vec2(), self.size.into_vec2())
    }
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
struct AabbRon {
    min: Vec3Ron,
    max: Vec3Ron,
}

impl AabbRon {
    const fn from_aabb(value: Aabb) -> Self {
        Self {
            min: Vec3Ron::from_vec3(value.min),
            max: Vec3Ron::from_vec3(value.max),
        }
    }

    const fn into_aabb(self) -> Aabb {
        Aabb::new(self.min.into_vec3(), self.max.into_vec3())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct AssetReferenceRon {
    guid: u64,
    path: String,
}

impl AssetReferenceRon {
    fn from_reference(value: &AssetReferenceValue) -> Self {
        Self {
            guid: value.guid(),
            path: value.path().to_owned(),
        }
    }

    fn into_reference(self) -> SceneResult<AssetReferenceValue> {
        AssetReferenceValue::new(self.guid, self.path).map_err(|error| SceneError::Serialization {
            reason: error.to_string(),
        })
    }
}

impl SceneInstanceDocument {
    /// Creates a serialized instance document.
    #[must_use]
    pub fn new(guid: InstanceGuid, class_name: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            guid,
            class_name: class_name.into(),
            name: name.into(),
            properties: BTreeMap::new(),
            children: Vec::new(),
        }
    }

    /// Sets reflected property values.
    #[must_use]
    pub fn with_properties(mut self, properties: BTreeMap<String, PropertyValue>) -> Self {
        self.properties = properties;
        self
    }

    /// Sets ordered children.
    #[must_use]
    pub fn with_children(mut self, children: Vec<Self>) -> Self {
        self.children = children;
        self
    }
}

impl Scene {
    /// Converts this scene into a deterministic dependency-free scene document.
    ///
    /// # Errors
    ///
    /// Returns [`SceneError::MissingRoot`] when the scene has no root.
    pub fn to_document(&self) -> SceneResult<SceneDocument> {
        let root_id = self.root.ok_or(SceneError::MissingRoot)?;
        Ok(SceneDocument::new(self.instance_to_document(root_id)?))
    }

    /// Creates a scene from a dependency-free scene document and class registry.
    ///
    /// Runtime instance IDs are assigned deterministically in parent-before-child
    /// document order. Serialized GUIDs are preserved.
    ///
    /// # Errors
    ///
    /// Returns [`SceneError`] when the document contains duplicate GUIDs,
    /// unknown classes, invalid names, or invalid property values.
    pub fn from_document(
        class_registry: InstanceClassRegistry,
        document: SceneDocument,
    ) -> SceneResult<Self> {
        let mut scene = Self::with_class_registry(class_registry);
        let mut seen_guids = BTreeSet::new();
        let root = scene.add_document_instance(document.root, None, &mut seen_guids)?;
        scene.root = Some(root);
        Ok(scene)
    }
    fn instance_to_document(&self, id: InstanceId) -> SceneResult<SceneInstanceDocument> {
        let instance = self.get(id)?;
        let children = instance
            .children
            .iter()
            .map(|child_id| self.instance_to_document(*child_id))
            .collect::<SceneResult<Vec<_>>>()?;
        Ok(SceneInstanceDocument {
            guid: instance.guid,
            class_name: instance.class_name.clone(),
            name: instance.name.clone(),
            properties: instance.properties.clone(),
            children,
        })
    }

    fn add_document_instance(
        &mut self,
        document: SceneInstanceDocument,
        parent: Option<InstanceId>,
        seen_guids: &mut BTreeSet<InstanceGuid>,
    ) -> SceneResult<InstanceId> {
        if !seen_guids.insert(document.guid) {
            return Err(SceneError::DuplicateInstanceGuid {
                guid: document.guid,
            });
        }

        validate_instance_name(&document.name)?;
        let class_name = document.class_name;
        let name = document.name;
        let guid = document.guid;
        let children = document.children;
        let mut properties =
            default_properties_for_class(self.class_descriptor(&class_name)?, &name)?;
        self.validate_document_properties(&class_name, &document.properties)?;
        for (path, value) in document.properties {
            properties.insert(path, value);
        }
        if let Some(PropertyValue::String(stored_name)) = properties.get_mut("Name") {
            stored_name.clone_from(&name);
        }

        let id = self.next_instance_id();
        self.next_guid = self.next_guid.max(guid.raw() + 1);
        self.instances.push(InstanceRecord {
            id,
            guid,
            class_name,
            name,
            parent,
            children: Vec::new(),
            properties,
        });

        for child in children {
            let child_id = self.add_document_instance(child, Some(id), seen_guids)?;
            let index = self.index_of(id)?;
            self.instances[index].children.push(child_id);
        }

        Ok(id)
    }

    fn validate_document_properties(
        &self,
        class_name: &str,
        properties: &BTreeMap<String, PropertyValue>,
    ) -> SceneResult<()> {
        for (path, value) in properties {
            let descriptor = self.property_descriptor_for_class(class_name, path)?;
            value
                .validate_for_descriptor(descriptor)
                .map_err(|error| property_value_error(class_name, path, error))?;
            if let ("Name", PropertyValue::String(name)) = (path.as_str(), value) {
                validate_instance_name(name)?;
            }
        }
        Ok(())
    }
}

fn serialization_error(error: impl std::fmt::Display) -> SceneError {
    SceneError::Serialization {
        reason: error.to_string(),
    }
}

fn non_zero_raw(field: &'static str, value: u64) -> SceneResult<u64> {
    if value == 0 {
        return Err(SceneError::Serialization {
            reason: format!("{field} must be non-zero"),
        });
    }
    Ok(value)
}

use crate::*;
use kinetik_core::{InstanceGuid, InstanceId};
use kinetik_reflect::{PropertyDescriptor, PropertyType, PropertyValue};

fn scene_with_part_class() -> Scene {
    Scene::new()
}

mod class;
mod default_scene;
mod document;
mod mutation;
mod scene_core;
mod transform;

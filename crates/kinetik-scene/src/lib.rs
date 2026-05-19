//! Scene and instance graph contracts for Kinetik.

mod class;
mod document;
mod error;
mod mutation;
mod record;
mod scene;
mod transform;

pub use class::{
    ClassRegistryError, ClassRegistryResult, InstanceClassCapability, InstanceClassDescriptor,
    InstanceClassRegistry, BUILT_IN_3D_CLASS_NAMES, DEFAULT_SERVICE_CLASS_NAMES, ROOT_CLASS_NAME,
};
pub use document::{SceneDocument, SceneInstanceDocument};
pub use error::{SceneError, SceneResult};
pub use mutation::{SceneMutation, SceneMutationQueue, SceneMutationResult};
pub use record::InstanceRecord;
pub use scene::Scene;
pub use transform::PART_LOCAL_BOUNDS;

#[cfg(test)]
mod tests;

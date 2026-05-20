use kinetik_core::{Diagnostic, InstanceGuid, InstanceId, Transform};
use kinetik_reflect::PropertyValue;
use kinetik_scene::Scene;

use crate::{
    render_diagnostic, PrimitiveMesh, StandardMaterial, MISSING_CAMERA_CODE, MISSING_LIGHT_CODE,
    MISSING_MATERIAL_CODE, MISSING_MESH_CODE,
};

/// Renderer-owned scene extraction output.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderExtraction {
    /// First active camera found in deterministic scene order.
    pub camera: Option<ExtractedCamera>,
    /// Extracted lights in deterministic scene order.
    pub lights: Vec<ExtractedLight>,
    /// Extracted renderable primitives in deterministic scene order.
    pub primitives: Vec<ExtractedPrimitive>,
    /// Structured render diagnostics.
    pub diagnostics: Vec<Diagnostic>,
}

/// Camera extracted from scene state.
#[derive(Clone, Debug, PartialEq)]
pub struct ExtractedCamera {
    /// Source scene instance ID.
    pub instance_id: InstanceId,
    /// Stable source instance GUID.
    pub guid: InstanceGuid,
    /// World transform.
    pub transform: Transform,
}

/// Light extracted from scene state.
#[derive(Clone, Debug, PartialEq)]
pub struct ExtractedLight {
    /// Source scene instance ID.
    pub instance_id: InstanceId,
    /// Stable source instance GUID.
    pub guid: InstanceGuid,
    /// World transform.
    pub transform: Transform,
    /// Simple light intensity scaffold.
    pub intensity: f32,
}

/// Renderable primitive extracted from scene state.
#[derive(Clone, Debug, PartialEq)]
pub struct ExtractedPrimitive {
    /// Source scene instance ID.
    pub instance_id: InstanceId,
    /// Stable source instance GUID.
    pub guid: InstanceGuid,
    /// World transform.
    pub transform: Transform,
    /// Built-in primitive mesh metadata.
    pub mesh: PrimitiveMesh,
    /// PBR-compatible material scaffold.
    pub material: StandardMaterial,
}

/// Extracts renderer-owned data from a scene snapshot.
#[must_use]
pub fn extract_render_scene(scene: &Scene) -> RenderExtraction {
    let mut extraction = RenderExtraction::default();
    if let Some(root_id) = scene.root_id() {
        collect_scene_instance(scene, root_id, &mut extraction);
    }

    if extraction.camera.is_none() {
        extraction.diagnostics.push(render_diagnostic(
            MISSING_CAMERA_CODE,
            "render scene has no Camera3D instance",
        ));
    }
    if extraction.lights.is_empty() {
        extraction.diagnostics.push(render_diagnostic(
            MISSING_LIGHT_CODE,
            "render scene has no Light3D instance",
        ));
    }
    if extraction.primitives.is_empty() {
        extraction.diagnostics.push(render_diagnostic(
            MISSING_MESH_CODE,
            "render scene has no supported primitive mesh instances",
        ));
    }

    extraction
}

fn collect_scene_instance(scene: &Scene, id: InstanceId, extraction: &mut RenderExtraction) {
    let Ok(instance) = scene.get(id) else {
        return;
    };
    match instance.class_name.as_str() {
        "Camera3D" if extraction.camera.is_none() => {
            if let Ok(transform) = scene.world_transform(id) {
                extraction.camera = Some(ExtractedCamera {
                    instance_id: id,
                    guid: instance.guid,
                    transform,
                });
            }
        }
        "Light3D" => {
            if let Ok(transform) = scene.world_transform(id) {
                extraction.lights.push(ExtractedLight {
                    instance_id: id,
                    guid: instance.guid,
                    transform,
                    intensity: 1.0,
                });
            }
        }
        "Part" => extract_part(scene, id, extraction),
        _ => {}
    }

    if let Ok(children) = scene.children(id) {
        for child in children {
            collect_scene_instance(scene, *child, extraction);
        }
    }
}

fn extract_part(scene: &Scene, id: InstanceId, extraction: &mut RenderExtraction) {
    let Ok(instance) = scene.get(id) else {
        return;
    };
    let visible = matches!(
        scene.get_property(id, "Visible"),
        Ok(PropertyValue::Bool(true)) | Err(_)
    );
    if !visible {
        return;
    }
    let Ok(transform) = scene.world_transform(id) else {
        extraction.diagnostics.push(render_diagnostic(
            MISSING_MESH_CODE,
            format!("Part {} could not produce a world transform", instance.name),
        ));
        return;
    };
    extraction.diagnostics.push(render_diagnostic(
        MISSING_MATERIAL_CODE,
        format!("Part {} used StandardMaterial fallback", instance.name),
    ));
    extraction.primitives.push(ExtractedPrimitive {
        instance_id: id,
        guid: instance.guid,
        transform,
        mesh: PrimitiveMesh::cube(),
        material: StandardMaterial::FALLBACK,
    });
}

use kinetik_command::DirtyStateExplanation;
use kinetik_core::{Diagnostic, DiagnosticBlockingScope, DiagnosticSeverity, InstanceGuid};
use kinetik_project::{ProjectDocumentRefs, ProjectSettingsDocument};
use kinetik_resource::AssetManifest;
use kinetik_scene::Scene;

/// Editor-owned read-only MCP command names.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum McpReadOnlyCommand {
    /// Inspect project identity and active source documents.
    ProjectStatus,
    /// List scene instances in deterministic hierarchy order.
    SceneListInstances,
    /// List resource manifest entries in deterministic manifest order.
    ResourceList,
    /// List current diagnostics.
    DiagnosticsList,
    /// Explain current edit dirty state.
    DirtyState,
}

impl McpReadOnlyCommand {
    /// Returns the stable MCP command name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ProjectStatus => "project.status",
            Self::SceneListInstances => "scene.list_instances",
            Self::ResourceList => "resource.list",
            Self::DiagnosticsList => "diagnostics.list",
            Self::DirtyState => "editor.dirty_state",
        }
    }
}

/// Read-only project status response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectStatusResponse {
    /// Project display name.
    pub project_name: String,
    /// Engine compatibility string.
    pub engine_compatibility: String,
    /// Active scene source document path.
    pub active_scene: String,
    /// Asset manifest source document path.
    pub assets_manifest: String,
    /// Instance manifest source document path.
    pub instances_manifest: String,
}

/// Read-only scene instance summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneInstanceSummary {
    /// Runtime/editor session instance ID raw value.
    pub instance_id: u64,
    /// Stable instance GUID raw value.
    pub guid: u64,
    /// Registered class name.
    pub class_name: String,
    /// Display instance name.
    pub name: String,
    /// Human-readable scene path.
    pub scene_path: String,
    /// Parent instance ID raw value.
    pub parent_id: Option<u64>,
    /// Child instance IDs in scene order.
    pub child_ids: Vec<u64>,
}

/// Read-only resource manifest entry summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceManifestEntrySummary {
    /// Stable asset GUID raw value.
    pub guid: u64,
    /// Project resource path.
    pub path: String,
    /// Importer identifier.
    pub importer_id: String,
    /// Importer version.
    pub importer_version: String,
    /// Import settings hash.
    pub settings_hash: String,
}

/// Read-only resource manifest response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceManifestResponse {
    /// Manifest entries in deterministic path order.
    pub entries: Vec<ResourceManifestEntrySummary>,
}

/// Read-only diagnostic summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticSummary {
    /// Stable diagnostic code.
    pub code: String,
    /// Diagnostic severity.
    pub severity: DiagnosticSeverity,
    /// System that produced the diagnostic.
    pub source: String,
    /// Human-readable message.
    pub message: String,
    /// Blocked workflow, when known.
    pub blocking: Option<DiagnosticBlockingScope>,
    /// Related instance GUID raw value, when known.
    pub instance_guid: Option<u64>,
    /// Related scene path, when known.
    pub scene_path: Option<String>,
    /// Related asset path, when known.
    pub asset_path: Option<String>,
    /// Related script path, when known.
    pub script_path: Option<String>,
    /// Related property path, when known.
    pub property_path: Option<String>,
}

/// Read-only dirty-state response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirtyStateResponse {
    /// Whether any edit document is dirty.
    pub is_dirty: bool,
    /// Dirty document paths in first-change order.
    pub documents: Vec<String>,
    /// Human-readable dirty summaries in command history order.
    pub summaries: Vec<String>,
}

/// Builds a read-only project status response.
#[must_use]
pub fn project_status_response(
    settings: &ProjectSettingsDocument,
    documents: &ProjectDocumentRefs,
) -> ProjectStatusResponse {
    ProjectStatusResponse {
        project_name: settings.identity().name().to_owned(),
        engine_compatibility: settings.identity().engine_compatibility().to_owned(),
        active_scene: documents.active_scene().to_owned(),
        assets_manifest: documents.assets_manifest().to_owned(),
        instances_manifest: documents.instances_manifest().to_owned(),
    }
}

/// Builds a deterministic read-only scene hierarchy response.
///
/// # Errors
///
/// Returns scene errors when the scene has invalid parent/child references.
pub fn scene_hierarchy_response(
    scene: &Scene,
) -> kinetik_scene::SceneResult<Vec<SceneInstanceSummary>> {
    let mut summaries = Vec::new();
    if let Some(root_id) = scene.root_id() {
        collect_instance_summary(scene, root_id, &mut summaries)?;
    }
    Ok(summaries)
}

/// Builds a deterministic read-only resource manifest response.
#[must_use]
pub fn resource_manifest_response(manifest: &AssetManifest) -> ResourceManifestResponse {
    ResourceManifestResponse {
        entries: manifest
            .entries()
            .iter()
            .map(|entry| ResourceManifestEntrySummary {
                guid: entry.guid().raw(),
                path: entry.path().as_str().to_owned(),
                importer_id: entry.importer_id().to_owned(),
                importer_version: entry.importer_version().to_owned(),
                settings_hash: entry.settings_hash().as_str().to_owned(),
            })
            .collect(),
    }
}

/// Builds a read-only diagnostics list response.
#[must_use]
pub fn diagnostics_list_response(diagnostics: &[Diagnostic]) -> Vec<DiagnosticSummary> {
    diagnostics
        .iter()
        .map(|diagnostic| DiagnosticSummary {
            code: diagnostic.code.as_str().to_owned(),
            severity: diagnostic.severity,
            source: diagnostic.source.as_str().to_owned(),
            message: diagnostic.message.clone(),
            blocking: diagnostic.blocking,
            instance_guid: diagnostic.location.instance_guid.map(InstanceGuid::raw),
            scene_path: diagnostic.location.scene_path.clone(),
            asset_path: diagnostic.location.asset_path.clone(),
            script_path: diagnostic.location.script_path.clone(),
            property_path: diagnostic.location.property_path.clone(),
        })
        .collect()
}

/// Builds a read-only dirty-state response.
#[must_use]
pub fn dirty_state_response(explanation: &DirtyStateExplanation) -> DirtyStateResponse {
    let documents = explanation
        .documents()
        .iter()
        .map(|document| document.document_path().to_owned())
        .collect::<Vec<_>>();
    let summaries = explanation
        .changes()
        .iter()
        .map(|change| change.change_summary().to_owned())
        .collect::<Vec<_>>();

    DirtyStateResponse {
        is_dirty: !documents.is_empty(),
        documents,
        summaries,
    }
}

fn collect_instance_summary(
    scene: &Scene,
    instance_id: kinetik_core::InstanceId,
    summaries: &mut Vec<SceneInstanceSummary>,
) -> kinetik_scene::SceneResult<()> {
    let instance = scene.get(instance_id)?;
    let children = scene.children(instance_id)?.to_vec();
    summaries.push(SceneInstanceSummary {
        instance_id: instance.id.raw(),
        guid: instance.guid.raw(),
        class_name: instance.class_name.clone(),
        name: instance.name.clone(),
        scene_path: scene.path(instance_id)?,
        parent_id: instance.parent.map(kinetik_core::InstanceId::raw),
        child_ids: children.iter().map(|id| id.raw()).collect(),
    });

    for child_id in children {
        collect_instance_summary(scene, child_id, summaries)?;
    }

    Ok(())
}

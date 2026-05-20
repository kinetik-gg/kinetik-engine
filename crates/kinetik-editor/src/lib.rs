//! editor scaffold for Kinetik.

mod mcp;
mod shell;

pub use mcp::{
    diagnostics_list_response, dirty_state_response, project_status_response,
    resource_manifest_response, scene_hierarchy_response, DiagnosticSummary, DirtyStateResponse,
    McpMutatingCommand, McpMutationResponse, McpMutationSession, McpReadOnlyCommand,
    McpSceneMutationRequest, McpUndoRedoResponse, ProjectStatusResponse,
    ResourceManifestEntrySummary, ResourceManifestResponse, SceneInstanceSummary,
};
pub use shell::{
    default_editor_shell_layout, run_editor_shell, EditorPanel, EditorShellError,
    EditorShellLayout, EditorShellState, EditorShellWindow, PanelDock, ToolbarAction,
};

/// Returns the crate name for smoke tests and early integration checks.
#[must_use]
pub const fn crate_name() -> &'static str {
    "kinetik-editor"
}

#[cfg(test)]
mod tests {
    use super::*;
    use kinetik_command::{create_scene_child_instance, CommandHistory, DirtyStateExplanation};
    use kinetik_core::{
        Diagnostic, DiagnosticBlockingScope, DiagnosticCode, DiagnosticLocation,
        DiagnosticSeverity, DiagnosticSource, InstanceGuid,
    };
    use kinetik_project::{ProjectDocumentRefs, ProjectIdentity, ProjectSettingsDocument};
    use kinetik_resource::{AssetGuid, AssetManifest, AssetManifestEntry};
    use kinetik_scene::Scene;

    #[test]
    fn exposes_crate_name() {
        assert_eq!(crate_name(), "kinetik-editor");
    }

    #[test]
    fn mcp_read_only_command_names_are_stable() {
        assert_eq!(McpReadOnlyCommand::ProjectStatus.as_str(), "project.status");
        assert_eq!(
            McpReadOnlyCommand::SceneListInstances.as_str(),
            "scene.list_instances"
        );
        assert_eq!(McpReadOnlyCommand::ResourceList.as_str(), "resource.list");
        assert_eq!(
            McpReadOnlyCommand::DiagnosticsList.as_str(),
            "diagnostics.list"
        );
        assert_eq!(
            McpReadOnlyCommand::DirtyState.as_str(),
            "editor.dirty_state"
        );
    }

    #[test]
    fn mcp_mutating_command_names_are_stable() {
        assert_eq!(
            McpMutatingCommand::SceneCreateInstance.as_str(),
            "scene.create_instance"
        );
        assert_eq!(
            McpMutatingCommand::SceneDeleteInstance.as_str(),
            "scene.delete_instance"
        );
        assert_eq!(McpMutatingCommand::EditorUndo.as_str(), "editor.undo");
        assert_eq!(McpMutatingCommand::EditorRedo.as_str(), "editor.redo");
    }

    #[test]
    fn project_status_reports_identity_and_documents() {
        let settings = ProjectSettingsDocument::new(
            ProjectIdentity::new("Demo", "0.1").expect("valid project identity"),
        );
        let documents =
            ProjectDocumentRefs::new("scenes/main.knscene", "assets.knmanifest", "instances.ron")
                .expect("valid document refs");

        let response = project_status_response(&settings, &documents);

        assert_eq!(response.project_name, "Demo");
        assert_eq!(response.engine_compatibility, "0.1");
        assert_eq!(response.active_scene, "scenes/main.knscene");
        assert_eq!(response.assets_manifest, "assets.knmanifest");
        assert_eq!(response.instances_manifest, "instances.ron");
    }

    #[test]
    fn scene_hierarchy_reports_parent_before_child_order() {
        let mut scene = Scene::default_scene().expect("valid default scene");
        let workspace = scene.get_by_path("/Game/Workspace").unwrap().id;
        create_scene_child_instance(
            &mut scene,
            workspace,
            "Part",
            "Block",
            "scenes/main.knscene",
        )
        .unwrap();

        let response = scene_hierarchy_response(&scene).unwrap();

        assert_eq!(response[0].scene_path, "/Game");
        assert_eq!(response[1].scene_path, "/Game/Workspace");
        assert!(response
            .iter()
            .any(|instance| instance.scene_path == "/Game/Workspace/Block"));
    }

    #[test]
    fn resource_manifest_response_preserves_manifest_order() {
        let manifest = AssetManifest::from_entries(vec![
            AssetManifestEntry::from_parts(
                AssetGuid::new(2),
                "res://assets/z.glb",
                "gltf",
                "1",
                "hash-z",
            )
            .unwrap(),
            AssetManifestEntry::from_parts(
                AssetGuid::new(1),
                "res://assets/a.glb",
                "gltf",
                "1",
                "hash-a",
            )
            .unwrap(),
        ])
        .unwrap();

        let response = resource_manifest_response(&manifest);

        assert_eq!(response.entries[0].path, "res://assets/a.glb");
        assert_eq!(response.entries[1].path, "res://assets/z.glb");
        assert_eq!(response.entries[0].settings_hash, "hash-a");
    }

    #[test]
    fn diagnostics_list_response_preserves_structured_targets() {
        let diagnostic = Diagnostic::new(
            DiagnosticCode::new("KT_TEST"),
            DiagnosticSeverity::Warning,
            DiagnosticSource::new("Test"),
            "Something is notable.",
        )
        .with_blocking_scope(DiagnosticBlockingScope::Edit)
        .with_location(DiagnosticLocation {
            instance_guid: Some(InstanceGuid::new(7)),
            scene_path: Some("/Game/Workspace".to_owned()),
            asset_path: Some("res://assets/a.glb".to_owned()),
            script_path: Some("res://scripts/a.luau".to_owned()),
            source_range: None,
            property_path: Some("Name".to_owned()),
        });

        let response = diagnostics_list_response(&[diagnostic]);

        assert_eq!(response[0].code, "KT_TEST");
        assert_eq!(response[0].severity, DiagnosticSeverity::Warning);
        assert_eq!(response[0].blocking, Some(DiagnosticBlockingScope::Edit));
        assert_eq!(response[0].instance_guid, Some(7));
        assert_eq!(response[0].property_path.as_deref(), Some("Name"));
    }

    #[test]
    fn dirty_state_response_reports_documents_and_summaries() {
        let mut scene = Scene::default_scene().expect("valid default scene");
        let workspace = scene.get_by_path("/Game/Workspace").unwrap().id;
        let command = create_scene_child_instance(
            &mut scene,
            workspace,
            "Part",
            "Block",
            "scenes/main.knscene",
        )
        .unwrap();
        let mut history = CommandHistory::new();
        history
            .commit_result("Create Block", &command.command)
            .unwrap();
        let explanation = DirtyStateExplanation::from_history(&history);

        let response = dirty_state_response(&explanation);

        assert!(response.is_dirty);
        assert_eq!(response.documents, vec!["scenes/main.knscene".to_owned()]);
        assert_eq!(
            response.summaries,
            vec!["created /Game/Workspace/Block".to_owned()]
        );
    }

    #[test]
    fn mcp_scene_create_maps_to_command_and_dirty_state() {
        let mut scene = Scene::default_scene().expect("valid default scene");
        let workspace = scene.get_by_path("/Game/Workspace").unwrap().id;
        let mut session = McpMutationSession::new();

        let response = session.execute_scene_mutation(
            &mut scene,
            McpSceneMutationRequest::CreateInstance {
                target_mode: Some(kinetik_command::CommandTargetMode::Edit),
                parent_id: workspace,
                class_name: "Part".to_owned(),
                name: "Block".to_owned(),
            },
            "scenes/main.knscene",
        );

        assert_eq!(response.status, kinetik_command::CommandStatus::Succeeded);
        assert_eq!(
            response.command_kind,
            kinetik_command::CREATE_INSTANCE_COMMAND
        );
        assert_eq!(response.undo_group, Some(1));
        assert_eq!(
            response.change_summaries,
            vec!["created /Game/Workspace/Block".to_owned()]
        );
        assert!(response.dirty_state.is_dirty);
        assert_eq!(
            scene.get_by_path("/Game/Workspace/Block").unwrap().name,
            "Block"
        );
        assert_eq!(session.history().undo_stack().len(), 1);
    }

    #[test]
    fn mcp_scene_property_maps_to_command_path() {
        let mut scene = Scene::default_scene().expect("valid default scene");
        let workspace = scene.get_by_path("/Game/Workspace").unwrap().id;
        let mut session = McpMutationSession::new();

        let response = session.execute_scene_mutation(
            &mut scene,
            McpSceneMutationRequest::SetProperty {
                target_mode: Some(kinetik_command::CommandTargetMode::Edit),
                instance_id: workspace,
                property_path: "Name".to_owned(),
                value: kinetik_reflect::PropertyValue::String("World".to_owned()),
            },
            "scenes/main.knscene",
        );

        assert_eq!(response.status, kinetik_command::CommandStatus::Succeeded);
        assert_eq!(response.command_kind, kinetik_command::SET_PROPERTY_COMMAND);
        assert_eq!(scene.path(workspace).unwrap(), "/Game/World");
        assert_eq!(response.undo_group, Some(1));
    }

    #[test]
    fn mcp_mutation_rejects_ambiguous_target_mode_before_mutation() {
        let mut scene = Scene::default_scene().expect("valid default scene");
        let workspace = scene.get_by_path("/Game/Workspace").unwrap().id;
        let mut session = McpMutationSession::new();

        let response = session.execute_scene_mutation(
            &mut scene,
            McpSceneMutationRequest::CreateInstance {
                target_mode: None,
                parent_id: workspace,
                class_name: "Part".to_owned(),
                name: "Block".to_owned(),
            },
            "scenes/main.knscene",
        );

        assert_eq!(response.status, kinetik_command::CommandStatus::Failed);
        assert_eq!(
            response.command_kind,
            kinetik_command::CREATE_INSTANCE_COMMAND
        );
        assert_eq!(response.undo_group, None);
        assert!(!response.diagnostics.is_empty());
        assert!(scene.get_by_path("/Game/Workspace/Block").is_err());
        assert!(session.history().undo_stack().is_empty());
    }

    #[test]
    fn mcp_undo_redo_moves_history_stacks() {
        let mut scene = Scene::default_scene().expect("valid default scene");
        let workspace = scene.get_by_path("/Game/Workspace").unwrap().id;
        let mut session = McpMutationSession::new();

        session.execute_scene_mutation(
            &mut scene,
            McpSceneMutationRequest::CreateInstance {
                target_mode: Some(kinetik_command::CommandTargetMode::Edit),
                parent_id: workspace,
                class_name: "Part".to_owned(),
                name: "Block".to_owned(),
            },
            "scenes/main.knscene",
        );

        let undo = session.undo();
        assert!(undo.moved);
        assert_eq!(undo.undo_group, Some(1));
        assert!(session.history().undo_stack().is_empty());
        assert_eq!(session.history().redo_stack().len(), 1);

        let redo = session.redo();
        assert!(redo.moved);
        assert_eq!(redo.undo_group, Some(1));
        assert_eq!(session.history().undo_stack().len(), 1);
        assert!(session.history().redo_stack().is_empty());
    }
}

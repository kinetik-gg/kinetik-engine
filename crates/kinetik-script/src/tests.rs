use kinetik_core::{
    DiagnosticBlockingScope, DiagnosticSeverity, InstanceGuid, InstanceId, ResourceId, ScriptId,
    SourceRange,
};

use crate::{
    invalid_script_handle_diagnostic, missing_script_diagnostic, LifecycleCallLog, LifecyclePhase,
    ResourceScriptHandle, ScriptAssetRef, ScriptDiagnosticCode, ScriptDiagnosticContext,
    ScriptInstanceHandle, ScriptLanguage, ScriptRuntimeHost,
};

#[test]
fn lifecycle_calls_dispatch_in_scheduler_order() {
    let attachment_a = ScriptId::new(1);
    let attachment_b = ScriptId::new(2);
    let mut host = ScriptRuntimeHost::new();
    let mut runtime = LifecycleCallLog::new();

    host.queue_ready(attachment_a);
    host.queue_update(attachment_a, 0.016);
    host.queue_physics_update(attachment_b, 0.02);
    host.queue_exit(attachment_a);

    host.dispatch(&mut runtime).unwrap();

    let calls = runtime.calls();
    assert_eq!(calls.len(), 4);
    assert_eq!(calls[0].phase, LifecyclePhase::Ready);
    assert_eq!(calls[1].phase, LifecyclePhase::Update);
    assert_eq!(calls[1].delta_seconds, Some(0.016));
    assert_eq!(calls[2].phase, LifecyclePhase::PhysicsUpdate);
    assert_eq!(calls[2].delta_seconds, Some(0.02));
    assert_eq!(calls[3].phase, LifecyclePhase::Exit);
    assert!(host.pending_calls().is_empty());
}

#[test]
fn missing_script_diagnostic_carries_script_and_instance_provenance() {
    let script = ScriptAssetRef::new("res://scripts/player.luau", ScriptLanguage::luau())
        .with_resource_id(ResourceId::new(8))
        .with_source_range(SourceRange::new(3, 1, 3, 20));
    let target = crate::ScriptAttachmentTarget::with_guid(InstanceId::new(4), InstanceGuid::new(5));
    let context = ScriptDiagnosticContext::new(script)
        .with_target(target)
        .with_scene_path("/Game/Workspace/Player");

    let diagnostic = missing_script_diagnostic(&context);

    assert_eq!(diagnostic.code, ScriptDiagnosticCode::MISSING_SCRIPT);
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
    assert_eq!(diagnostic.blocking, Some(DiagnosticBlockingScope::Play));
    assert_eq!(
        diagnostic.location.script_path.as_deref(),
        Some("res://scripts/player.luau")
    );
    assert_eq!(
        diagnostic.location.scene_path.as_deref(),
        Some("/Game/Workspace/Player")
    );
    assert_eq!(
        diagnostic.location.instance_guid,
        Some(InstanceGuid::new(5))
    );
    assert_eq!(
        diagnostic.location.source_range,
        Some(SourceRange::new(3, 1, 3, 20))
    );
}

#[test]
fn invalidated_handles_fail_without_exposing_raw_state() {
    let instance = ScriptInstanceHandle::invalidated(InstanceId::new(9));
    let resource = ResourceScriptHandle::invalidated(ResourceId::new(10));

    assert!(instance.resolve().is_err());
    assert!(resource.resolve().is_err());

    let context = ScriptDiagnosticContext::new(ScriptAssetRef::new(
        "res://scripts/enemy.luau",
        ScriptLanguage::luau(),
    ));
    let diagnostic = invalid_script_handle_diagnostic(&context);

    assert_eq!(diagnostic.code, ScriptDiagnosticCode::INVALID_HANDLE);
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
    assert_eq!(diagnostic.blocking, Some(DiagnosticBlockingScope::Play));
}

#[test]
fn valid_handles_resolve_to_runtime_ids() {
    let instance_id = InstanceId::new(12);
    let resource_id = ResourceId::new(13);

    assert_eq!(
        ScriptInstanceHandle::new(instance_id).resolve(),
        Ok(instance_id)
    );
    assert_eq!(
        ResourceScriptHandle::new(resource_id).resolve(),
        Ok(resource_id)
    );
}

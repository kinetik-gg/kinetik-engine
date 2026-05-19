use super::*;

#[test]
fn vector_primitives_store_components_and_defaults() {
    assert_eq!(Vec2::new(1.0, 2.0), Vec2 { x: 1.0, y: 2.0 });
    assert_eq!(Vec2::default(), Vec2::ZERO);
    assert_eq!(Vec2::splat(3.0), Vec2::new(3.0, 3.0));

    assert_eq!(
        Vec3::new(1.0, 2.0, 3.0),
        Vec3 {
            x: 1.0,
            y: 2.0,
            z: 3.0
        }
    );
    assert_eq!(Vec3::default(), Vec3::ZERO);
    assert_eq!(Vec3::splat(4.0), Vec3::new(4.0, 4.0, 4.0));

    assert_eq!(
        Vec4::new(1.0, 2.0, 3.0, 4.0),
        Vec4 {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            w: 4.0
        }
    );
    assert_eq!(Vec4::default(), Vec4::ZERO);
    assert_eq!(Vec4::splat(5.0), Vec4::new(5.0, 5.0, 5.0, 5.0));
}

#[test]
fn rotation_color_and_transform_defaults_are_explicit() {
    assert_eq!(Quat::default(), Quat::IDENTITY);
    assert_eq!(Quat::IDENTITY, Quat::new(0.0, 0.0, 0.0, 1.0));

    assert_eq!(Color::default(), Color::WHITE);
    assert_eq!(
        Color::rgb(0.25, 0.5, 0.75),
        Color::new(0.25, 0.5, 0.75, 1.0)
    );
    assert_eq!(Color::TRANSPARENT, Color::new(0.0, 0.0, 0.0, 0.0));

    assert_eq!(Transform::default(), Transform::IDENTITY);
    assert_eq!(
        Transform::IDENTITY,
        Transform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE)
    );
    assert_eq!(
        Transform::from_position(Vec3::new(1.0, 2.0, 3.0)),
        Transform::new(Vec3::new(1.0, 2.0, 3.0), Quat::IDENTITY, Vec3::ONE)
    );
}

#[test]
fn rect_and_aabb_helpers_are_deterministic() {
    let rect = Rect::new(Vec2::new(2.0, 3.0), Vec2::new(4.0, 5.0));
    assert_eq!(Rect::default(), Rect::ZERO);
    assert_eq!(rect.max(), Vec2::new(6.0, 8.0));

    let bounds = Aabb::new(Vec3::new(-1.0, 2.0, 3.0), Vec3::new(5.0, 6.0, 9.0));
    assert_eq!(Aabb::default(), Aabb::ZERO);
    assert_eq!(bounds.size(), Vec3::new(6.0, 4.0, 6.0));
    assert_eq!(bounds.center(), Vec3::new(2.0, 4.0, 6.0));
}

#[test]
fn diagnostic_codes_are_stable_strings() {
    assert_eq!(
        DiagnosticCode::CORE_INVALID_HANDLE.as_str(),
        "KT_CORE_INVALID_HANDLE"
    );
    assert_eq!(DiagnosticCode::CORE_NOT_FOUND.as_str(), "KT_CORE_NOT_FOUND");
    assert_eq!(
        DiagnosticCode::CORE_NOT_IMPLEMENTED.as_str(),
        "KT_CORE_NOT_IMPLEMENTED"
    );
}

#[test]
fn diagnostic_shape_carries_core_fields() {
    let location = DiagnosticLocation {
        instance_guid: Some(InstanceGuid::new(42)),
        scene_path: Some("/Game/Lighting/Sun".to_owned()),
        asset_path: None,
        script_path: Some("scripts/sun.luau".to_owned()),
        source_range: Some(SourceRange::new(1, 2, 3, 4)),
        property_path: Some("Intensity".to_owned()),
    };

    let diagnostic = Diagnostic::new(
        DiagnosticCode::new("KT_TEST_EXAMPLE"),
        DiagnosticSeverity::Warning,
        DiagnosticSource::new("Test"),
        "Example diagnostic",
    )
    .with_blocking_scope(DiagnosticBlockingScope::Test)
    .with_location(location)
    .with_suggested_fix("Update the test fixture")
    .allow_agent_repair();

    assert_eq!(diagnostic.code.as_str(), "KT_TEST_EXAMPLE");
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Warning);
    assert_eq!(diagnostic.source.as_str(), "Test");
    assert_eq!(diagnostic.blocking, Some(DiagnosticBlockingScope::Test));
    assert_eq!(diagnostic.location.instance_guid.unwrap().raw(), 42);
    assert_eq!(
        diagnostic.location.scene_path.as_deref(),
        Some("/Game/Lighting/Sun")
    );
    assert_eq!(
        diagnostic.location.script_path.as_deref(),
        Some("scripts/sun.luau")
    );
    assert_eq!(
        diagnostic.location.property_path.as_deref(),
        Some("Intensity")
    );
    assert_eq!(
        diagnostic.suggested_fix.as_deref(),
        Some("Update the test fixture")
    );
    assert_eq!(diagnostic.agent_repair, AgentRepair::Allowed);
}

#[test]
fn kinetik_errors_map_to_diagnostics() {
    let error = KinetikError::InvalidHandle {
        kind: "InstanceId",
        id: 99,
    };

    let diagnostic = error.to_diagnostic(
        DiagnosticSource::new("Scene"),
        Some(DiagnosticBlockingScope::Play),
    );

    assert_eq!(diagnostic.code, DiagnosticCode::CORE_INVALID_HANDLE);
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
    assert_eq!(diagnostic.source.as_str(), "Scene");
    assert_eq!(diagnostic.blocking, Some(DiagnosticBlockingScope::Play));
    assert_eq!(diagnostic.message, "invalid InstanceId handle: 99");
    assert_eq!(diagnostic.agent_repair, AgentRepair::NotAllowed);
}

#[test]
fn kinetik_error_from_conversion_uses_core_source() {
    let diagnostic = Diagnostic::from(KinetikError::NotImplemented { feature: "Bundles" });

    assert_eq!(diagnostic.code, DiagnosticCode::CORE_NOT_IMPLEMENTED);
    assert_eq!(diagnostic.source, DiagnosticSource::CORE);
    assert_eq!(diagnostic.message, "feature not implemented: Bundles");
}

#[test]
fn typed_ids_do_not_share_types() {
    let instance = InstanceId::new(7);
    let resource = ResourceId::new(7);
    assert_eq!(instance.raw(), resource.raw());
    assert_ne!(format!("{instance:?}"), format!("{resource:?}"));
}

#[test]
fn typed_id_display_includes_kind_and_raw_value() {
    assert_eq!(InstanceId::new(1).to_string(), "InstanceId(1)");
    assert_eq!(InstanceGuid::new(2).to_string(), "InstanceGuid(2)");
    assert_eq!(ResourceId::new(3).to_string(), "ResourceId(3)");
    assert_eq!(SignalId::new(4).to_string(), "SignalId(4)");
    assert_eq!(ScriptId::new(5).to_string(), "ScriptId(5)");
    assert_eq!(BundleId::new(6).to_string(), "BundleId(6)");
}

#[test]
fn typed_ids_reject_zero_raw_values() {
    assert!(std::panic::catch_unwind(|| InstanceId::new(0)).is_err());
    assert!(std::panic::catch_unwind(|| InstanceGuid::new(0)).is_err());
    assert!(std::panic::catch_unwind(|| ResourceId::new(0)).is_err());
    assert!(std::panic::catch_unwind(|| SignalId::new(0)).is_err());
    assert!(std::panic::catch_unwind(|| ScriptId::new(0)).is_err());
    assert!(std::panic::catch_unwind(|| BundleId::new(0)).is_err());
}

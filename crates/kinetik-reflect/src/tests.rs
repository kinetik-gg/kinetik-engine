use super::*;

use kinetik_core::{Aabb, Color, InstanceId, Quat, Rect, ResourceId, Transform, Vec2, Vec3, Vec4};

#[test]
fn exposes_crate_name() {
    assert_eq!(crate_name(), "kinetik-reflect");
}

#[test]
fn descriptor_creation_sets_core_fields() {
    let descriptor = PropertyDescriptor::new("Transform.Position", "Position", PropertyType::Vec3)
        .unwrap()
        .with_default_value(PropertyDefault::Value(PropertyValue::Vec3(Vec3::ZERO)))
        .with_editor_hint(EditorHint::FreeNumber)
        .with_validation_rules(vec![ValidationRule::Required])
        .with_documentation("Local position.");

    assert_eq!(descriptor.path, "Transform.Position");
    assert_eq!(descriptor.display_name, "Position");
    assert_eq!(descriptor.value_type, PropertyType::Vec3);
    assert_eq!(
        descriptor.default_value,
        PropertyDefault::Value(PropertyValue::Vec3(Vec3::ZERO))
    );
    assert_eq!(descriptor.serialization_key, "Transform.Position");
    assert!(descriptor.is_serialized());
    assert!(descriptor.is_editor_editable());
    assert!(descriptor.is_scriptable());
    assert!(descriptor.is_mutable_during_play());
    assert_eq!(descriptor.editor_hint, EditorHint::FreeNumber);
    assert_eq!(descriptor.validation_rules, vec![ValidationRule::Required]);
    assert_eq!(descriptor.documentation, "Local position.");
    descriptor.validate().unwrap();
}

#[test]
fn read_only_descriptors_require_a_reason() {
    let descriptor = PropertyDescriptor::new("RuntimeId", "Runtime ID", PropertyType::InstanceId)
        .unwrap()
        .with_editor_editable(false);

    assert_eq!(
        descriptor.validate().unwrap_err(),
        DescriptorError::MissingReadOnlyReason {
            path: "RuntimeId".to_owned()
        }
    );

    descriptor
        .with_read_only_reason("Assigned by the runtime.")
        .validate()
        .unwrap();
}

#[test]
fn invalid_descriptor_cases_are_reported() {
    assert_eq!(
        PropertyDescriptor::new("", "Name", PropertyType::String).unwrap_err(),
        DescriptorError::EmptyPath
    );
    assert_eq!(
        PropertyDescriptor::new("transform.Position", "Position", PropertyType::Vec3).unwrap_err(),
        DescriptorError::InvalidPath {
            path: "transform.Position".to_owned()
        }
    );
    assert_eq!(
        PropertyDescriptor::new("Name", "   ", PropertyType::String).unwrap_err(),
        DescriptorError::EmptyDisplayName {
            path: "Name".to_owned()
        }
    );
    assert_eq!(
        PropertyDescriptor::new("Name", "Name", PropertyType::String)
            .unwrap()
            .with_serialization_key(" ")
            .validate()
            .unwrap_err(),
        DescriptorError::EmptySerializationKey {
            path: "Name".to_owned()
        }
    );
    assert_eq!(
        PropertyDescriptor::new("Transform.Position", "Position", PropertyType::Vec3)
            .unwrap()
            .with_default_value(PropertyDefault::Value(PropertyValue::String(
                "wrong".to_owned()
            )))
            .validate()
            .unwrap_err(),
        DescriptorError::DefaultTypeMismatch {
            path: "Transform.Position".to_owned(),
            expected: PropertyType::Vec3,
            actual: PropertyType::String
        }
    );
}

#[test]
fn property_values_report_reflected_types() {
    assert_eq!(
        PropertyValue::String("Avala".to_owned()).property_type(),
        PropertyType::String
    );
    assert_eq!(
        PropertyValue::Bool(true).property_type(),
        PropertyType::Bool
    );
    assert_eq!(PropertyValue::F32(1.5).property_type(), PropertyType::F32);
    assert_eq!(
        PropertyValue::Vec2(Vec2::new(1.0, 2.0)).property_type(),
        PropertyType::Vec2
    );
    assert_eq!(
        PropertyValue::Vec3(Vec3::new(1.0, 2.0, 3.0)).property_type(),
        PropertyType::Vec3
    );
    assert_eq!(
        PropertyValue::Vec4(Vec4::new(1.0, 2.0, 3.0, 4.0)).property_type(),
        PropertyType::Vec4
    );
    assert_eq!(
        PropertyValue::Quat(Quat::IDENTITY).property_type(),
        PropertyType::Quat
    );
    assert_eq!(
        PropertyValue::Color(Color::WHITE).property_type(),
        PropertyType::Color
    );
    assert_eq!(
        PropertyValue::Transform(Transform::IDENTITY).property_type(),
        PropertyType::Transform
    );
    assert_eq!(
        PropertyValue::Rect(Rect::ZERO).property_type(),
        PropertyType::Rect
    );
    assert_eq!(
        PropertyValue::Aabb(Aabb::ZERO).property_type(),
        PropertyType::Aabb
    );
    assert_eq!(
        PropertyValue::InstanceId(InstanceId::new(1)).property_type(),
        PropertyType::InstanceId
    );
    assert_eq!(
        PropertyValue::ResourceId(ResourceId::new(1)).property_type(),
        PropertyType::ResourceId
    );
}

#[test]
fn property_values_validate_against_descriptors() {
    let descriptor =
        PropertyDescriptor::new("Transform.Position", "Position", PropertyType::Vec3).unwrap();
    let value = PropertyValue::Vec3(Vec3::new(1.0, 2.0, 3.0));

    assert!(value.is_compatible_with(&descriptor));
    value.validate_for_descriptor(&descriptor).unwrap();

    let mismatch = PropertyValue::String("wrong".to_owned())
        .validate_for_descriptor(&descriptor)
        .unwrap_err();

    assert_eq!(
        mismatch,
        ValueError::TypeMismatch {
            path: "Transform.Position".to_owned(),
            expected: PropertyType::Vec3,
            actual: PropertyType::String
        }
    );
    assert_eq!(mismatch.diagnostic_code(), ValueError::TYPE_MISMATCH_CODE);
    let diagnostic = mismatch.to_diagnostic();
    assert_eq!(diagnostic.code, ValueError::TYPE_MISMATCH_CODE);
    assert_eq!(diagnostic.source.as_str(), "Reflection");
    assert_eq!(
        diagnostic.location.property_path.as_deref(),
        Some("Transform.Position")
    );
}

#[test]
fn property_value_defaults_are_type_aware() {
    assert_eq!(
        PropertyValue::type_default(PropertyType::Transform).unwrap(),
        PropertyValue::Transform(Transform::IDENTITY)
    );
    assert_eq!(
        PropertyValue::type_default(PropertyType::Color).unwrap(),
        PropertyValue::Color(Color::WHITE)
    );
    assert_eq!(
        PropertyValue::type_default(PropertyType::InstanceId).unwrap_err(),
        ValueError::NoTypeDefault {
            value_type: PropertyType::InstanceId
        }
    );
}

#[test]
fn value_validation_rejects_invalid_descriptor_paths() {
    let mut descriptor = PropertyDescriptor::new("Name", "Name", PropertyType::String).unwrap();
    descriptor.path = "name".to_owned();

    assert_eq!(
        PropertyValue::String("Workspace".to_owned())
            .validate_for_descriptor(&descriptor)
            .unwrap_err(),
        ValueError::InvalidDescriptor(DescriptorError::InvalidPath {
            path: "name".to_owned()
        })
    );
}

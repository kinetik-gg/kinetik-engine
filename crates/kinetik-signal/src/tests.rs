use super::*;

use kinetik_core::{
    DiagnosticBlockingScope, DiagnosticLocation, DiagnosticSeverity, InstanceGuid, InstanceId,
    SignalId,
};

#[test]
fn register_signals_in_deterministic_order() {
    let mut registry = SignalRegistry::default();
    let touched = registry.register("Touched").unwrap();
    let stepped = registry
        .register_with_owner(
            SignalOwner::Instance(InstanceId::new(7)),
            "PhysicsTouched",
            SignalFlushDomain::FixedStep,
        )
        .unwrap();

    assert_eq!(touched, SignalId::new(1));
    assert_eq!(stepped, SignalId::new(2));
    assert_eq!(
        registry.signals(),
        &[
            SignalDescriptor {
                id: touched,
                owner: SignalOwner::Global,
                name: "Touched".to_owned(),
                flush_domain: SignalFlushDomain::Frame,
            },
            SignalDescriptor {
                id: stepped,
                owner: SignalOwner::Instance(InstanceId::new(7)),
                name: "PhysicsTouched".to_owned(),
                flush_domain: SignalFlushDomain::FixedStep,
            },
        ]
    );
}

#[test]
fn register_rejects_invalid_and_duplicate_names_per_owner() {
    let mut registry = SignalRegistry::default();

    assert_eq!(
        registry.register("").unwrap_err(),
        SignalError::EmptySignalName
    );
    assert_eq!(
        registry.register("touched").unwrap_err(),
        SignalError::InvalidSignalName {
            name: "touched".to_owned()
        }
    );

    registry.register("Touched").unwrap();
    assert_eq!(
        registry.register("Touched").unwrap_err(),
        SignalError::DuplicateSignal {
            owner: SignalOwner::Global,
            name: "Touched".to_owned()
        }
    );

    registry
        .register_with_owner(
            SignalOwner::Instance(InstanceId::new(1)),
            "Touched",
            SignalFlushDomain::Frame,
        )
        .unwrap();
}

#[test]
fn signal_error_diagnostic_codes_are_stable() {
    assert_eq!(
        SignalError::INVALID_SIGNAL_NAME_CODE.as_str(),
        "KT_SIGNAL_INVALID_NAME"
    );
    assert_eq!(
        SignalError::DUPLICATE_SIGNAL_CODE.as_str(),
        "KT_SIGNAL_DUPLICATE_SIGNAL"
    );
    assert_eq!(
        SignalError::UNKNOWN_SIGNAL_CODE.as_str(),
        "KT_SIGNAL_UNKNOWN_SIGNAL"
    );
    assert_eq!(
        SignalError::UNKNOWN_CONNECTION_CODE.as_str(),
        "KT_SIGNAL_UNKNOWN_CONNECTION"
    );
    assert_eq!(
        SignalError::WRONG_FLUSH_DOMAIN_CODE.as_str(),
        "KT_SIGNAL_WRONG_FLUSH_DOMAIN"
    );
    assert_eq!(SignalError::SIGNAL_SOURCE.as_str(), "Signal");
}

#[test]
fn signal_errors_convert_to_play_blocking_diagnostics() {
    let duplicate = SignalError::DuplicateSignal {
        owner: SignalOwner::Instance(InstanceId::new(7)),
        name: "Touched".to_owned(),
    }
    .to_diagnostic();

    assert_eq!(duplicate.code, SignalError::DUPLICATE_SIGNAL_CODE);
    assert_eq!(duplicate.severity, DiagnosticSeverity::Error);
    assert_eq!(duplicate.source, SignalError::SIGNAL_SOURCE);
    assert_eq!(duplicate.blocking, Some(DiagnosticBlockingScope::Play));
    assert!(duplicate.message.contains("Touched"));
    assert!(duplicate.message.contains("InstanceId(7)"));
    assert_eq!(duplicate.location, DiagnosticLocation::default());
    assert_eq!(duplicate.suggested_fix, None);
}

#[test]
fn signal_handle_diagnostics_include_stable_handle_context() {
    let unknown_signal = SignalError::UnknownSignal {
        id: SignalId::new(99),
    }
    .to_diagnostic();
    let unknown_connection = SignalError::UnknownConnection {
        id: SignalConnectionId::new(42),
    }
    .to_diagnostic();
    let wrong_domain = SignalError::WrongFlushDomain {
        id: SignalId::new(3),
        expected: SignalFlushDomain::Frame,
        actual: SignalFlushDomain::FixedStep,
    }
    .to_diagnostic();

    assert_eq!(unknown_signal.code, SignalError::UNKNOWN_SIGNAL_CODE);
    assert!(unknown_signal.message.contains("SignalId(99)"));
    assert_eq!(
        unknown_connection.code,
        SignalError::UNKNOWN_CONNECTION_CODE
    );
    assert!(unknown_connection
        .message
        .contains("SignalConnectionId(42)"));
    assert_eq!(wrong_domain.code, SignalError::WRONG_FLUSH_DOMAIN_CODE);
    assert!(wrong_domain.message.contains("SignalId(3)"));
    assert!(wrong_domain.message.contains("Frame"));
    assert!(wrong_domain.message.contains("FixedStep"));
}

#[test]
fn connections_use_deterministic_creation_order() {
    let mut registry = SignalRegistry::default();
    let touched = registry.register("Touched").unwrap();
    let changed = registry.register("Changed").unwrap();

    let first = registry.connect(touched).unwrap();
    let second = registry.connect(touched).unwrap();
    let third = registry.connect(changed).unwrap();

    assert_eq!(first, SignalConnectionId::new(1));
    assert_eq!(second, SignalConnectionId::new(2));
    assert_eq!(third, SignalConnectionId::new(3));
    assert_eq!(
        registry
            .connections()
            .iter()
            .map(|connection| connection.order)
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
}

#[test]
fn connect_rejects_unknown_signal() {
    let mut registry = SignalRegistry::default();

    assert_eq!(
        registry.connect(SignalId::new(99)).unwrap_err(),
        SignalError::UnknownSignal {
            id: SignalId::new(99)
        }
    );
}

#[test]
fn disconnect_is_idempotent() {
    let mut registry = SignalRegistry::default();
    let touched = registry.register("Touched").unwrap();
    let connection = registry.connect(touched).unwrap();

    assert_eq!(
        registry.disconnect(connection).unwrap(),
        SignalConnectionState::Disconnected
    );
    assert_eq!(
        registry.disconnect(connection).unwrap(),
        SignalConnectionState::Disconnected
    );
    assert_eq!(
        registry.connection(connection).unwrap().state,
        SignalConnectionState::Disconnected
    );
}

#[test]
fn disconnect_rejects_unknown_connection() {
    let mut registry = SignalRegistry::default();

    assert_eq!(
        registry
            .disconnect(SignalConnectionId::new(99))
            .unwrap_err(),
        SignalError::UnknownConnection {
            id: SignalConnectionId::new(99)
        }
    );
}

#[test]
fn invalidates_connections_for_signal() {
    let mut registry = SignalRegistry::default();
    let touched = registry.register("Touched").unwrap();
    let changed = registry.register("Changed").unwrap();
    let first = registry.connect(touched).unwrap();
    let second = registry.connect(touched).unwrap();
    let other = registry.connect(changed).unwrap();

    assert_eq!(
        registry.invalidate_connections_for_signal(touched).unwrap(),
        2
    );
    assert_eq!(
        registry.connection(first).unwrap().state,
        SignalConnectionState::Invalidated
    );
    assert_eq!(
        registry.connection(second).unwrap().state,
        SignalConnectionState::Invalidated
    );
    assert_eq!(
        registry.connection(other).unwrap().state,
        SignalConnectionState::Connected
    );
}

#[test]
fn frame_events_queue_in_emit_order() {
    let mut registry = SignalRegistry::default();
    let touched = registry.register("Touched").unwrap();
    let changed = registry.register("Changed").unwrap();

    let first = registry
        .queue_frame_event(
            changed,
            10,
            Some(InstanceId::new(7)),
            Some(InstanceGuid::new(70)),
        )
        .unwrap();
    let second = registry.queue_frame_event(touched, 10, None, None).unwrap();

    assert_eq!(first.sequence, 1);
    assert_eq!(second.sequence, 2);
    assert_eq!(registry.events(), &[first, second]);
}

#[test]
fn fixed_step_events_record_fixed_step_context() {
    let mut registry = SignalRegistry::default();
    let touched = registry
        .register_with_owner(
            SignalOwner::Instance(InstanceId::new(3)),
            "Touched",
            SignalFlushDomain::FixedStep,
        )
        .unwrap();

    let event = registry
        .queue_fixed_step_event(touched, 4, 12, Some(InstanceId::new(3)), None)
        .unwrap();

    assert_eq!(event.flush_domain, SignalFlushDomain::FixedStep);
    assert_eq!(event.frame_index, 4);
    assert_eq!(event.fixed_step_index, Some(12));
    assert_eq!(event.sequence, 1);
}

#[test]
fn queue_rejects_unknown_signal_and_wrong_flush_domain() {
    let mut registry = SignalRegistry::default();
    let frame_signal = registry.register("Touched").unwrap();

    assert_eq!(
        registry
            .queue_frame_event(SignalId::new(99), 1, None, None)
            .unwrap_err(),
        SignalError::UnknownSignal {
            id: SignalId::new(99)
        }
    );
    assert_eq!(
        registry
            .queue_fixed_step_event(frame_signal, 1, 1, None, None)
            .unwrap_err(),
        SignalError::WrongFlushDomain {
            id: frame_signal,
            expected: SignalFlushDomain::Frame,
            actual: SignalFlushDomain::FixedStep,
        }
    );
}

#[test]
fn delivery_records_follow_event_then_connection_order() {
    let mut registry = SignalRegistry::default();
    let touched = registry.register("Touched").unwrap();
    let changed = registry.register("Changed").unwrap();
    let first = registry.connect(touched).unwrap();
    let skipped = registry.connect(touched).unwrap();
    let second = registry.connect(touched).unwrap();
    let changed_connection = registry.connect(changed).unwrap();
    registry.disconnect(skipped).unwrap();
    registry.queue_frame_event(changed, 1, None, None).unwrap();
    registry.queue_frame_event(touched, 1, None, None).unwrap();

    let records = registry.delivery_records(SignalFlushDomain::Frame);

    assert_eq!(
        records
            .iter()
            .map(|record| record.connection_id)
            .collect::<Vec<_>>(),
        vec![changed_connection, first, second]
    );
}

#[test]
fn drain_events_filters_by_flush_domain() {
    let mut registry = SignalRegistry::default();
    let frame_signal = registry.register("Touched").unwrap();
    let fixed_signal = registry
        .register_with_owner(
            SignalOwner::Global,
            "PhysicsTouched",
            SignalFlushDomain::FixedStep,
        )
        .unwrap();
    let frame_event = registry
        .queue_frame_event(frame_signal, 1, None, None)
        .unwrap();
    let fixed_event = registry
        .queue_fixed_step_event(fixed_signal, 1, 1, None, None)
        .unwrap();

    assert_eq!(
        registry.drain_events(SignalFlushDomain::Frame),
        vec![frame_event]
    );
    assert_eq!(registry.events(), &[fixed_event]);
    assert_eq!(
        registry.drain_events(SignalFlushDomain::FixedStep),
        vec![fixed_event]
    );
    assert!(registry.events().is_empty());
}

#[test]
fn flush_events_delivers_then_drains_selected_domain() {
    let mut registry = SignalRegistry::default();
    let frame_signal = registry.register("Touched").unwrap();
    let fixed_signal = registry
        .register_with_owner(
            SignalOwner::Global,
            "PhysicsTouched",
            SignalFlushDomain::FixedStep,
        )
        .unwrap();
    let frame_connection = registry.connect(frame_signal).unwrap();
    let fixed_connection = registry.connect(fixed_signal).unwrap();
    let frame_event = registry
        .queue_frame_event(frame_signal, 7, None, None)
        .unwrap();
    let fixed_event = registry
        .queue_fixed_step_event(fixed_signal, 7, 2, None, None)
        .unwrap();

    let records = registry.flush_events(SignalFlushDomain::FixedStep);

    assert_eq!(
        records,
        vec![SignalDeliveryRecord {
            event: fixed_event,
            connection_id: fixed_connection,
            connection_order: 2,
        }]
    );
    assert_eq!(registry.events(), &[frame_event]);
    assert_eq!(
        registry.flush_events(SignalFlushDomain::Frame),
        vec![SignalDeliveryRecord {
            event: frame_event,
            connection_id: frame_connection,
            connection_order: 1,
        }]
    );
    assert!(registry.events().is_empty());
}

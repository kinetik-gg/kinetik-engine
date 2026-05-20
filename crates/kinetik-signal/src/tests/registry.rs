use super::*;

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

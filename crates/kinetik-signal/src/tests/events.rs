use super::*;

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

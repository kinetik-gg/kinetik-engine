use super::*;

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
fn cleanup_owner_removes_owned_signals_connections_and_events() {
    let mut registry = SignalRegistry::default();
    let destroyed_owner = SignalOwner::Instance(InstanceId::new(7));
    let owned_signal = registry
        .register_with_owner(destroyed_owner, "Touched", SignalFlushDomain::Frame)
        .unwrap();
    let global_signal = registry.register("GlobalTouched").unwrap();
    let survivor_signal = registry
        .register_with_owner(
            SignalOwner::Instance(InstanceId::new(8)),
            "Touched",
            SignalFlushDomain::Frame,
        )
        .unwrap();
    let owned_connection = registry.connect(owned_signal).unwrap();
    let global_connection = registry.connect(global_signal).unwrap();
    let survivor_connection = registry.connect(survivor_signal).unwrap();
    let owned_event = registry
        .queue_frame_event(owned_signal, 3, Some(InstanceId::new(9)), None)
        .unwrap();
    let emitted_by_destroyed_owner = registry
        .queue_frame_event(global_signal, 3, Some(InstanceId::new(7)), None)
        .unwrap();
    let survivor_event = registry
        .queue_frame_event(survivor_signal, 3, Some(InstanceId::new(8)), None)
        .unwrap();

    let cleanup = registry.cleanup_owner(destroyed_owner);

    assert_eq!(
        cleanup,
        SignalOwnerCleanup {
            owner: destroyed_owner,
            affected_signals: vec![owned_signal],
            invalidated_connections: vec![owned_connection],
            removed_events: vec![owned_event, emitted_by_destroyed_owner],
        }
    );
    assert_eq!(registry.signal(owned_signal), None);
    assert_eq!(
        registry.connection(owned_connection).unwrap().state,
        SignalConnectionState::Invalidated
    );
    assert_eq!(
        registry.connection(global_connection).unwrap().state,
        SignalConnectionState::Connected
    );
    assert_eq!(
        registry.connection(survivor_connection).unwrap().state,
        SignalConnectionState::Connected
    );
    assert_eq!(registry.events(), &[survivor_event]);
    assert_eq!(
        registry
            .signals()
            .iter()
            .map(|signal| signal.id)
            .collect::<Vec<_>>(),
        vec![global_signal, survivor_signal]
    );

    let next_signal = registry.register("Changed").unwrap();
    assert_eq!(next_signal, SignalId::new(4));
}

#[test]
fn cleanup_owner_without_owned_signals_removes_emitted_events() {
    let mut registry = SignalRegistry::default();
    let signal = registry.register("Touched").unwrap();
    let removed_event = registry
        .queue_frame_event(signal, 1, Some(InstanceId::new(4)), None)
        .unwrap();
    let retained_event = registry
        .queue_frame_event(signal, 1, Some(InstanceId::new(5)), None)
        .unwrap();

    let cleanup = registry.cleanup_owner(SignalOwner::Instance(InstanceId::new(4)));

    assert_eq!(cleanup.affected_signals, Vec::<SignalId>::new());
    assert_eq!(
        cleanup.invalidated_connections,
        Vec::<SignalConnectionId>::new()
    );
    assert_eq!(cleanup.removed_events, vec![removed_event]);
    assert_eq!(registry.signals()[0].id, signal);
    assert_eq!(registry.events(), &[retained_event]);
}

#[test]
fn clear_runtime_state_resets_world_signal_state() {
    let mut registry = SignalRegistry::default();
    let signal = registry.register("Touched").unwrap();
    let connection = registry.connect(signal).unwrap();
    let event = registry.queue_frame_event(signal, 1, None, None).unwrap();

    assert_eq!(signal, SignalId::new(1));
    assert_eq!(connection, SignalConnectionId::new(1));
    assert_eq!(event.sequence, 1);

    registry.clear_runtime_state();

    assert!(registry.signals().is_empty());
    assert!(registry.connections().is_empty());
    assert!(registry.events().is_empty());
    let next_signal = registry.register("Touched").unwrap();
    let next_connection = registry.connect(next_signal).unwrap();
    let next_event = registry
        .queue_frame_event(next_signal, 2, None, None)
        .unwrap();
    assert_eq!(next_signal, SignalId::new(1));
    assert_eq!(next_connection, SignalConnectionId::new(1));
    assert_eq!(next_event.sequence, 1);
}

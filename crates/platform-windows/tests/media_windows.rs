use islandvolt_platform_windows::media::MediaAction;
use std::{sync::mpsc, time::Duration};

#[test]
fn monitor_emits_initial_state() {
    let (monitor, updates) = islandvolt_platform_windows::media::watch();
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = sender.send(updates.recv_blocking());
    });
    let state = receiver.recv_timeout(Duration::from_secs(15));
    drop(monitor);
    assert!(
        matches!(state, Ok(Ok(Ok(_)))),
        "media monitor did not return an initial state: {state:?}"
    );
}

#[test]
fn stale_session_command_is_rejected() {
    let (monitor, updates) = islandvolt_platform_windows::media::watch();
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        while let Ok(result) = updates.recv_blocking() {
            if result.is_err() {
                let _ = sender.send(result);
            }
        }
    });
    for action in [
        MediaAction::Toggle,
        MediaAction::Previous,
        MediaAction::Next,
    ] {
        monitor.control(u64::MAX, action).expect("queue command");
        let result = receiver
            .recv_timeout(Duration::from_secs(15))
            .expect("media monitor response");
        assert_eq!(result.unwrap_err(), "Медиасессия изменилась");
    }
}

#[test]
fn source_selection_validates_ids_and_supports_automatic_mode() {
    let (monitor, updates) = islandvolt_platform_windows::media::watch();
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        while let Ok(state) = updates.recv_blocking() {
            if sender.send(state).is_err() {
                break;
            }
        }
    });
    let initial = receiver
        .recv_timeout(Duration::from_secs(5))
        .expect("initial update")
        .expect("media state");
    let ids: std::collections::HashSet<_> =
        initial.sources.iter().map(|source| source.id).collect();
    assert_eq!(ids.len(), initial.sources.len());
    if let Some(source) = initial.sources.first() {
        monitor.select(Some(source.id)).expect("select source");
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            let state = receiver
                .recv_timeout(deadline.saturating_duration_since(std::time::Instant::now()))
                .expect("selected state")
                .expect("media state");
            if state.selected == Some(source.id) {
                break;
            }
        }
    }
    monitor.select(None).expect("automatic mode");
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        let state = receiver
            .recv_timeout(deadline.saturating_duration_since(std::time::Instant::now()))
            .expect("automatic state")
            .expect("media state");
        if state.selected.is_none() {
            break;
        }
    }
    monitor
        .select(Some(u64::MAX))
        .expect("queue invalid selection");
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        let state = receiver
            .recv_timeout(deadline.saturating_duration_since(std::time::Instant::now()))
            .expect("invalid selection result");
        if let Err(error) = state {
            assert_eq!(error, "Плеер больше недоступен");
            break;
        }
    }
}

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

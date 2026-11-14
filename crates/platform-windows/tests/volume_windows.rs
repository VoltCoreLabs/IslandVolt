use std::{sync::mpsc, time::Duration};

#[test]
fn monitor_reads_initial_endpoint_without_modifying_volume() {
    let (monitor, updates) = islandvolt_platform_windows::volume::watch();
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = sender.send(updates.recv_blocking());
    });
    let result = receiver
        .recv_timeout(Duration::from_secs(15))
        .expect("volume monitor timeout");
    let state = result
        .expect("volume update channel")
        .expect("default audio device");
    assert!(state.percent <= 100);
    drop(monitor);
}

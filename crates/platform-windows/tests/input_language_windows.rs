use islandvolt_platform_windows::input_language;
use std::time::{Duration, Instant};
use windows::Win32::UI::{
    Input::KeyboardAndMouse::GetKeyboardLayout,
    WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId},
};

#[test]
fn snapshot_reads_the_foreground_threads_layout() {
    for _ in 0..4 {
        // SAFETY: read-only queries of current foreground input state.
        let (before, thread, layout) = unsafe {
            let before = GetForegroundWindow();
            let thread = GetWindowThreadProcessId(before, None);
            (before, thread, GetKeyboardLayout(thread).0 as usize)
        };
        if before.is_invalid() || thread == 0 {
            continue;
        }
        let snapshot = input_language::current().expect("foreground language");
        // SAFETY: comparison avoids racing a user switch to another foreground app.
        if before != unsafe { GetForegroundWindow() } {
            continue;
        }
        assert_eq!(snapshot.layout_id, layout);
        assert!(!snapshot.label.is_empty());
        return;
    }
    panic!("Foreground window did not stabilize during test");
}

#[test]
fn monitor_emits_initial_language_and_releases_subscription() -> anyhow::Result<()> {
    for _ in 0..3 {
        let (monitor, updates) = input_language::watch();
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            if let Ok(update) = updates.try_recv() {
                let language = update.map_err(anyhow::Error::msg)?;
                assert!(language.is_some());
                break;
            }
            anyhow::ensure!(
                Instant::now() < deadline,
                "No initial language notification"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        drop(monitor);
        while !updates.is_closed() {
            anyhow::ensure!(
                Instant::now() < deadline,
                "Language subscription did not close"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    Ok(())
}

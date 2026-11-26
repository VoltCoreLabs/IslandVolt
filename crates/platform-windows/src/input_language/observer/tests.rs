use super::*;
use std::{
    sync::atomic::AtomicUsize,
    time::{Duration, Instant},
};

#[test]
fn timer_delivers_changed_language_without_a_shell_event() -> anyhow::Result<()> {
    let value = Arc::new(AtomicUsize::new(0x0409));
    let worker_value = value.clone();
    let stop = Arc::new(Stop::default());
    let guard = crate::input_language::LanguageMonitor(stop.clone());
    let (sender, receiver) = async_channel::bounded(1);
    let worker = std::thread::spawn(move || {
        run_with_reader(stop, sender, move || {
            let layout_id = worker_value.load(Ordering::Acquire);
            Some(InputLanguage {
                layout_id,
                label: format!("{layout_id:X}"),
            })
        })
    });
    let wait = |id| -> anyhow::Result<()> {
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            if let Ok(Ok(Some(layout))) = receiver.try_recv()
                && layout.layout_id == id
            {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        anyhow::bail!("No timer update for {id:X}")
    };
    wait(0x0409)?;
    value.store(0x0419, Ordering::Release);
    wait(0x0419)?;
    value.store(0x0409, Ordering::Release);
    wait(0x0409)?;
    drop(guard);
    worker.join().expect("observer thread")?;
    Ok(())
}

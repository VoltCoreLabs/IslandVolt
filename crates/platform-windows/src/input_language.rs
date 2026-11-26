//! Input language of the foreground window, delivered by desktop shell notifications.
mod observer;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU32, Ordering},
};
use windows::Win32::{
    Globalization::{GetLocaleInfoW, LOCALE_SISO639LANGNAME},
    UI::{
        Input::KeyboardAndMouse::GetKeyboardLayout,
        WindowsAndMessaging::{
            GetForegroundWindow, GetWindowThreadProcessId, PostThreadMessageW, WM_QUIT,
        },
    },
};

/// Language is derived from the foreground thread's HKL, including IME language IDs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputLanguage {
    pub layout_id: usize,
    pub label: String,
}

/// Read a fresh foreground snapshot without changing any window or input layout.
pub fn current() -> Option<InputLanguage> {
    // SAFETY: these queries borrow OS handles and write only into a bounded local array.
    unsafe {
        let window = GetForegroundWindow();
        if window.is_invalid() {
            return None;
        }
        let thread = GetWindowThreadProcessId(window, None);
        if thread == 0 {
            return None;
        }
        let layout = GetKeyboardLayout(thread).0 as usize;
        if layout == 0 {
            return None;
        }
        let mut buffer = [0u16; 16];
        let length = GetLocaleInfoW(
            (layout & 0xffff) as u32,
            LOCALE_SISO639LANGNAME,
            Some(&mut buffer),
        );
        let label = if length > 1 {
            String::from_utf16_lossy(&buffer[..length as usize - 1]).to_uppercase()
        } else {
            format!("{:04X}", layout & 0xffff)
        };
        Some(InputLanguage {
            layout_id: layout,
            label,
        })
    }
}

#[derive(Default)]
pub(super) struct Stop {
    requested: AtomicBool,
    thread: AtomicU32,
}

/// Dropping the monitor wakes its message loop and releases the shell subscription.
pub struct LanguageMonitor(Arc<Stop>);

impl Drop for LanguageMonitor {
    fn drop(&mut self) {
        self.0.requested.store(true, Ordering::Release);
        let thread = self.0.thread.load(Ordering::Acquire);
        if thread != 0 {
            // SAFETY: the ID is published only after the worker creates its message queue.
            let _ = unsafe {
                PostThreadMessageW(thread, WM_QUIT, Default::default(), Default::default())
            };
        }
    }
}

/// Observe shell events with a 100 ms fallback for missed input-language notifications.
pub fn watch() -> (
    LanguageMonitor,
    async_channel::Receiver<Result<Option<InputLanguage>, String>>,
) {
    let stop = Arc::new(Stop::default());
    let worker_stop = stop.clone();
    let (sender, updates) = async_channel::bounded(1);
    std::thread::spawn(move || {
        let result = observer::run(worker_stop.clone(), sender.clone());
        worker_stop.thread.store(0, Ordering::Release);
        if result.is_err() {
            let _ = sender.force_send(Err("Язык ввода недоступен в этом сеансе Windows".into()));
        }
    });
    (LanguageMonitor(stop), updates)
}

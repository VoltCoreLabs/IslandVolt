use super::{InputLanguage, Stop, current};
use async_channel::Sender;
use std::sync::{Arc, atomic::Ordering};
use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        System::{LibraryLoader::GetModuleHandleW, Threading::GetCurrentThreadId},
        UI::WindowsAndMessaging::*,
    },
    core::{HSTRING, w},
};

struct State {
    read: Box<dyn Fn() -> Option<InputLanguage>>,
    shell_message: u32,
    sender: Sender<Result<Option<InputLanguage>, String>>,
    last: Option<InputLanguage>,
}

const REFRESH_TIMER: usize = 1;

impl State {
    fn refresh(&mut self) {
        let next = (self.read)();
        if next != self.last {
            self.last = next.clone();
            let _ = self.sender.force_send(Ok(next));
        }
    }
}

pub(super) fn run(
    stop: Arc<Stop>,
    sender: Sender<Result<Option<InputLanguage>, String>>,
) -> windows::core::Result<()> {
    run_with_reader(stop, sender, current)
}

fn run_with_reader(
    stop: Arc<Stop>,
    sender: Sender<Result<Option<InputLanguage>, String>>,
    read: impl Fn() -> Option<InputLanguage> + 'static,
) -> windows::core::Result<()> {
    // SAFETY: all window/class pointers remain valid through the message loop. The state is
    // stack-owned on this thread, installed in WM_NCCREATE, and detached before it is dropped.
    unsafe {
        let instance = GetModuleHandleW(None)?;
        let class = HSTRING::from(format!("IslandVoltLanguage-{}", GetCurrentThreadId()));
        let mut state = State {
            shell_message: RegisterWindowMessageW(w!("SHELLHOOK")),
            sender,
            last: read(),
            read: Box::new(read),
        };
        let mut message = MSG::default();
        let _ = PeekMessageW(&mut message, None, 0, 0, PM_NOREMOVE);
        stop.thread.store(GetCurrentThreadId(), Ordering::Release);
        if stop.requested.load(Ordering::Acquire) {
            return Ok(());
        }
        let descriptor = WNDCLASSW {
            lpfnWndProc: Some(procedure),
            hInstance: instance.into(),
            lpszClassName: windows::core::PCWSTR(class.as_ptr()),
            ..Default::default()
        };
        if state.shell_message == 0 || RegisterClassW(&descriptor) == 0 {
            return Err(windows::core::Error::from_win32());
        }
        let created = CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            &class,
            w!("IslandVolt input language"),
            WS_POPUP,
            0,
            0,
            0,
            0,
            None,
            None,
            Some(instance.into()),
            Some(std::ptr::from_mut(&mut state).cast()),
        );
        let result = match created {
            Ok(window) => {
                let subscribed = RegisterShellHookWindow(window).as_bool();
                // Shell does not reliably publish HSHELL_LANGUAGE on current Windows builds.
                // A 100 ms native timer reads the foreground HKL; only changes reach the UI.
                let timer = SetTimer(Some(window), REFRESH_TIMER, 100, None);
                let result = if timer != 0 {
                    let _ = state.sender.force_send(Ok(state.last.clone()));
                    loop {
                        let received = GetMessageW(&mut message, None, 0, 0).0;
                        if received == -1 {
                            break Err(windows::core::Error::from_win32());
                        }
                        if received == 0 {
                            break Ok(());
                        }
                        DispatchMessageW(&message);
                    }
                } else {
                    Err(windows::core::Error::from_win32())
                };
                if timer != 0 {
                    let _ = KillTimer(Some(window), REFRESH_TIMER);
                }
                if subscribed {
                    let _ = DeregisterShellHookWindow(window);
                }
                SetWindowLongPtrW(window, GWLP_USERDATA, 0);
                let _ = DestroyWindow(window);
                result
            }
            Err(error) => Err(error),
        };
        let _ = UnregisterClassW(&class, Some(instance.into()));
        result
    }
}

#[cfg(test)]
mod tests;

unsafe extern "system" fn procedure(
    window: HWND,
    message: u32,
    param: WPARAM,
    data: LPARAM,
) -> LRESULT {
    // SAFETY: WM_NCCREATE contains the creation structure. Later calls use the thread-local
    // live State pointer; no callback stores or transfers this pointer to another thread.
    unsafe {
        if message == WM_NCCREATE {
            let creation = &*(data.0 as *const CREATESTRUCTW);
            SetWindowLongPtrW(window, GWLP_USERDATA, creation.lpCreateParams as isize);
        }
        let state = GetWindowLongPtrW(window, GWLP_USERDATA) as *mut State;
        if let Some(state) = state.as_mut() {
            // HSHELL_LANGUAGE (8), WINDOWACTIVATED (4), RUDEAPPACTIVATED (0x8004).
            if (message == state.shell_message && matches!(param.0, 4 | 8 | 0x8004))
                || (message == WM_TIMER && param.0 == REFRESH_TIMER)
            {
                state.refresh();
            }
        }
        DefWindowProcW(window, message, param, data)
    }
}

//! Global shortcut registration, owned by the Windows event-loop thread.
use global_hotkey::{
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
    hotkey::{Code, HotKey, Modifiers},
};

/// Owns the native registration; dropping it releases the key and closes updates.
pub struct Shortcut {
    manager: GlobalHotKeyManager,
    key: HotKey,
    sender: async_channel::Sender<()>,
}

impl Shortcut {
    /// Register once on the application's Windows message-loop thread.
    pub fn register() -> anyhow::Result<(Self, async_channel::Receiver<()>)> {
        let manager = GlobalHotKeyManager::new()?;
        let key = HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::Space);
        manager.register(key)?;
        let (sender, receiver) = async_channel::bounded(1);
        let id = key.id();
        let weak_sender = sender.downgrade();
        GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
            if event.id == id
                && event.state == HotKeyState::Pressed
                && let Some(sender) = weak_sender.upgrade()
            {
                let _ = sender.try_send(());
            }
        }));
        Ok((
            Self {
                manager,
                key,
                sender,
            },
            receiver,
        ))
    }
}

impl Drop for Shortcut {
    fn drop(&mut self) {
        let _ = self.manager.unregister(self.key);
        self.sender.close();
        // The library's dispatcher is process-lifetime; it only holds a weak sender.
    }
}

/// Foreground window remembered for a keyboard-opened panel.
pub struct FocusRestore(windows::Win32::Foundation::HWND);

impl FocusRestore {
    pub fn capture() -> Self {
        // SAFETY: retrieving the current foreground handle does not transfer ownership.
        Self(unsafe { windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow() })
    }

    pub fn restore_if_current(&self, island: &impl raw_window_handle::HasWindowHandle) {
        use windows::Win32::UI::WindowsAndMessaging::{
            GetForegroundWindow, IsWindow, SetForegroundWindow,
        };
        let Ok(island) = super::hwnd(island) else {
            return;
        };
        // SAFETY: HWND is validated immediately before use; no ownership is taken.
        unsafe {
            if GetForegroundWindow() == island && IsWindow(Some(self.0)).as_bool() {
                let _ = SetForegroundWindow(self.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_registration_detects_conflict_and_releases_key() -> anyhow::Result<()> {
        // Do not intercept the product shortcut or common desktop shortcuts in tests.
        let key = HotKey::new(
            Some(Modifiers::CONTROL | Modifiers::ALT | Modifiers::SHIFT),
            Code::F24,
        );
        let first = GlobalHotKeyManager::new()?;
        let second = GlobalHotKeyManager::new()?;
        first.register(key)?;
        assert!(matches!(
            second.register(key),
            Err(global_hotkey::Error::AlreadyRegistered(_))
        ));
        drop(first);
        second.register(key)?;
        second.unregister(key)?;
        Ok(())
    }
}

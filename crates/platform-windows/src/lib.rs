//! Windows window integration for the island prototype.

mod frame;
pub mod input_language;
pub mod launcher;
pub mod media;
pub mod mixer;
pub mod shortcut;
pub mod tray;
pub mod volume;

/// Read the Windows client-area animation preference without changing it.
pub fn animations_enabled() -> bool {
    use windows::Win32::UI::WindowsAndMessaging::{
        SPI_GETCLIENTAREAANIMATION, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SystemParametersInfoW,
    };
    let mut enabled = windows::core::BOOL(1);
    // SAFETY: the API writes one BOOL into the provided live local variable.
    let result = unsafe {
        SystemParametersInfoW(
            SPI_GETCLIENTAREAANIMATION,
            0,
            Some((&mut enabled as *mut windows::core::BOOL).cast()),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
    };
    result.is_err() || enabled.as_bool()
}

use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows::Win32::{
    Foundation::HWND,
    Graphics::Gdi::{GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow},
    UI::WindowsAndMessaging::{HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SetWindowPos},
};

fn hwnd(window: &impl HasWindowHandle) -> Result<HWND, String> {
    let handle = window.window_handle().map_err(|error| error.to_string())?;
    let RawWindowHandle::Win32(raw) = handle.as_raw() else {
        return Err("Ожидалось окно Windows".into());
    };
    Ok(HWND(raw.hwnd.get() as *mut _))
}

/// Change visibility without stealing focus from the active application.
pub fn set_visible(window: &impl HasWindowHandle, visible: bool) -> Result<(), String> {
    use windows::Win32::UI::WindowsAndMessaging::{SW_HIDE, SW_SHOWNA, ShowWindow};
    let hwnd = hwnd(window)?;
    // SAFETY: the handle is borrowed from the live GPUI window.
    unsafe {
        let _ = ShowWindow(hwnd, if visible { SW_SHOWNA } else { SW_HIDE });
    }
    Ok(())
}

/// Keep the native window topmost; GPUI owns its visual shape and antialiasing.
pub fn configure_island(window: &impl HasWindowHandle) -> Result<(), String> {
    let hwnd = hwnd(window)?;
    frame::remove_native_frame(hwnd);
    // SAFETY: the handle is borrowed from the live GPUI window.
    unsafe {
        SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        )
    }
    .map_err(|error| error.to_string())
}

/// Anchor at monitor top/center without resizing; GPUI owns the content size.
pub fn position_island(window: &impl HasWindowHandle, width: i32) -> Result<(), String> {
    let hwnd = hwnd(window)?;
    // SAFETY: all handles and the output buffer remain valid throughout the calls.
    unsafe {
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if !GetMonitorInfoW(monitor, &mut info).as_bool() {
            return Err("Не удалось определить границы монитора".into());
        }
        let left = (info.rcMonitor.left + info.rcMonitor.right - width) / 2;
        SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            left,
            info.rcMonitor.top,
            0,
            0,
            SWP_NOACTIVATE | SWP_NOSIZE,
        )
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}

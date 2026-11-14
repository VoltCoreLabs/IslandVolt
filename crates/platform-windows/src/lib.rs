//! Windows window integration for the island prototype.

pub mod media;
pub mod volume;

use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows::Win32::{
    Foundation::{HWND, RECT},
    Graphics::Gdi::{CreateRoundRectRgn, DeleteObject, HGDIOBJ, SetWindowRgn},
    UI::WindowsAndMessaging::{
        GetWindowRect, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SetWindowPos,
    },
};

/// Make the native window topmost and clip it to a capsule at its current DPI.
pub fn configure_island(window: &impl HasWindowHandle) -> Result<(), String> {
    let handle = window.window_handle().map_err(|error| error.to_string())?;
    let RawWindowHandle::Win32(raw) = handle.as_raw() else {
        return Err("Ожидалось окно Windows".into());
    };
    let hwnd = HWND(raw.hwnd.get() as *mut _);
    let mut rect = RECT::default();
    // SAFETY: GPUI owns a live HWND for the duration of this call. The region is
    // transferred to Windows on success and destroyed locally on failure.
    unsafe {
        GetWindowRect(hwnd, &mut rect).map_err(|error| error.to_string())?;
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        let region = CreateRoundRectRgn(0, 0, width + 1, height + 1, height, height);
        if region.is_invalid() {
            return Err("Не удалось создать форму острова".into());
        }
        if SetWindowRgn(hwnd, Some(region), true) == 0 {
            let _ = DeleteObject(HGDIOBJ(region.0));
            return Err("Не удалось установить форму острова".into());
        }
        SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
        )
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}

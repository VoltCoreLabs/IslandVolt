//! Windows window integration for the island prototype.

pub mod media;
mod outline;
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
    Foundation::{HWND, RECT},
    Graphics::Gdi::{
        CreatePolygonRgn, CreateRoundRectRgn, DeleteObject, GetMonitorInfoW, HGDIOBJ,
        MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow, SetWindowRgn, WINDING,
    },
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

/// Resize around the window's horizontal center and update its rounded region.
pub fn resize_island(
    window: &impl HasWindowHandle,
    width: i32,
    height: i32,
    radius: i32,
) -> Result<(), String> {
    let handle = window.window_handle().map_err(|error| error.to_string())?;
    let RawWindowHandle::Win32(raw) = handle.as_raw() else {
        return Err("Ожидалось окно Windows".into());
    };
    let hwnd = HWND(raw.hwnd.get() as *mut _);
    let mut rect = RECT::default();
    // SAFETY: the handle is borrowed from the live GPUI window; ownership of the
    // newly created region passes to Windows only after SetWindowRgn succeeds.
    unsafe {
        GetWindowRect(hwnd, &mut rect).map_err(|error| error.to_string())?;
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
            width,
            height,
            SWP_NOACTIVATE,
        )
        .map_err(|error| error.to_string())?;
        let region = CreatePolygonRgn(&outline::points(width, height, radius), WINDING);
        if region.is_invalid() {
            return Err("Не удалось создать форму острова".into());
        }
        if SetWindowRgn(hwnd, Some(region), true) == 0 {
            let _ = DeleteObject(HGDIOBJ(region.0));
            return Err("Не удалось установить форму острова".into());
        }
    }
    Ok(())
}

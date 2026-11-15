//! Disable the OS frame; the GPUI silhouette owns all visible edges.
use windows::Win32::{
    Foundation::HWND,
    Graphics::Dwm::{
        DWMNCRP_DISABLED, DWMWA_BORDER_COLOR, DWMWA_NCRENDERING_POLICY,
        DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_DONOTROUND, DwmSetWindowAttribute,
    },
};

pub(super) fn remove_native_frame(hwnd: HWND) {
    // SAFETY: hwnd belongs to the live GPUI window; each buffer matches the DWM attribute.
    // Windows 10 may reject the Windows 11 border/corner hints; those are optional.
    unsafe {
        let policy = DWMNCRP_DISABLED;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_NCRENDERING_POLICY,
            std::ptr::from_ref(&policy).cast(),
            std::mem::size_of_val(&policy) as u32,
        );
        let border: u32 = 0xffff_fffe; // DWMWA_COLOR_NONE
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_BORDER_COLOR,
            (&border as *const u32).cast(),
            std::mem::size_of_val(&border) as u32,
        );
        let corners = DWMWCP_DONOTROUND;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            std::ptr::from_ref(&corners).cast(),
            std::mem::size_of_val(&corners) as u32,
        );
    }
}

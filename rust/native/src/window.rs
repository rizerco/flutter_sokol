use raw_window_handle::RawWindowHandle;

/// Sets the window background colour.
pub fn set_background_color(
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
    window_handle: &RawWindowHandle,
) {
    #[cfg(target_os = "macos")]
    unsafe {
        use objc2::rc::Retained;
        use objc2_app_kit::{NSColor, NSView};

        let handle = match window_handle {
            RawWindowHandle::AppKit(handle) => handle,
            handle => unreachable!("unknown handle {handle:?} for platform"),
        };
        let ns_view = handle.ns_view.as_ptr();
        let ns_view: Retained<NSView> = Retained::retain(ns_view.cast()).unwrap();
        let ns_window = ns_view
            .window()
            .expect("view was not installed in a window");
        let max = u8::MAX as f64;
        let red = red as f64 / max;
        let green = green as f64 / max;
        let blue = blue as f64 / max;
        let alpha = alpha as f64 / max;
        ns_window.setBackgroundColor(Some(&NSColor::colorWithSRGBRed_green_blue_alpha(
            red, green, blue, alpha,
        )));
    }
}

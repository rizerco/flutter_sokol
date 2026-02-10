use std::ffi;

unsafe extern "C" {
    fn fl_dart_project_new() -> *mut ffi::c_void;
    fn fl_view_new(ptr: *mut ffi::c_void) -> *mut ffi::c_void;
    fn fl_view_set_background_color(view: *const ffi::c_void, color: *const gdk::RGBA);
}

/// Creates a Flutter view.
pub fn create_flutter_view() -> *const ffi::c_void {
    unsafe {
        let project = fl_dart_project_new();
        let view = fl_view_new(project);

        let color = gdk::RGBA::new(0.0, 0.0, 0.0, 0.0);
        fl_view_set_background_color(view, &color);

        view
    }
}

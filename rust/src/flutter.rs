use std::ffi::CStr;

use std::ffi;

unsafe extern "C" {
    fn fl_dart_project_new() -> *mut ffi::c_void;
    fn fl_view_new(ptr: *mut ffi::c_void) -> *mut ffi::c_void;
    fn fl_dart_project_get_assets_path(ptr: *const ffi::c_void) -> *const ffi::c_char;
    fn fl_view_set_background_color(view: *const ffi::c_void, color: *const gdk::RGBA);
    fn fl_view_get_engine(view: *const ffi::c_void) -> *const ffi::c_void;
    fn fl_standard_message_codec_new() -> *mut ffi::c_void;
    fn fl_engine_get_binary_messenger(engine: *const ffi::c_void) -> *const ffi::c_void;
    fn fl_method_channel_new(
        binary_messenger: *const ffi::c_void,
        name: *const u8,
        codec: *const ffi::c_void,
    ) -> *const ffi::c_void;
}

/// Creates a Flutter view.
pub fn create_flutter_view() -> *const ffi::c_void {
    println!("arg:");
    unsafe {
        let project = fl_dart_project_new();
        let output = fl_dart_project_get_assets_path(project);
        let c_str = CStr::from_ptr(output);
        let string = c_str.to_str().unwrap();
        let view = fl_view_new(project);

        let color = gdk::RGBA::new(0.0, 0.0, 0.0, 0.0);
        fl_view_set_background_color(view, &color);

        let engine = fl_view_get_engine(view);
        create_method_channel(engine);
        println!("🏜️ made new FL project {:?}", string);
        view
    }
}

fn create_method_channel(engine: *const ffi::c_void) {
    unsafe {
        let codec = fl_standard_message_codec_new();
        let messenger = fl_engine_get_binary_messenger(engine);
        let channel = fl_method_channel_new(messenger, "decider".as_ptr(), codec);
    }
}

extern "C" fn method_call_cb(
    channel: *const ffi::c_void,
    method_call: *const ffi::c_char,
    user_data: *const ffi::c_void,
) {
    println!("methoc called");
    // const gchar *method = fl_method_call_get_name(method_call);
    // if (strcmp(method, "my_method") == 0) {
    //     // Code for "my_method"
    // } else {
    //   // Create response
    //   g_autoptr(FlMethodResponse) response = FL_METHOD_RESPONSE(fl_method_not_implemented_response_new());

    //   // Create error, in this case null
    //   g_autoptr(GError) error = nullptr;

    //   // Send response back to dart
    //   fl_method_call_respond(method_call, response, &error);
    // }
}

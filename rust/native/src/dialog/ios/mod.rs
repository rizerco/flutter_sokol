use objc2::{MainThreadMarker, rc::Retained};
use objc2_ui_kit::{UIApplication, UIViewController, UIWindowScene};

pub mod file;
pub mod message;
pub mod share;

/// Returns the root view controller.
fn root_view_controller(mtm: MainThreadMarker) -> Retained<UIViewController> {
    let application = UIApplication::sharedApplication(mtm);
    let scene = application
        .connectedScenes()
        .iter()
        .last()
        .unwrap()
        .downcast::<UIWindowScene>()
        .unwrap();
    let window = scene.keyWindow().unwrap();
    window.rootViewController().unwrap()
}

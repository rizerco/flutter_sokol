use objc2::{MainThreadMarker, MainThreadOnly, define_class, msg_send, rc::Retained};
use objc2_foundation::{NSArray, NSMutableArray, NSURL};
use objc2_ui_kit::{UIActivity, UIActivityViewController, UIModalPresentationStyle};

use crate::dialog::share::ShareDialog;

/// Show an iOS share sheet.
pub fn show_share_dialog(dialog: &ShareDialog) {
    dispatch2::run_on_main(|mtm| {
        let root = super::root_view_controller(mtm);
        let activity_view_controller = ModalActivityViewController::new(mtm, dialog);
        root.presentViewController_animated_completion(&activity_view_controller, true, None);
    });
}

struct ModalActivityViewControllerIvars {}

define_class!(
    /// An activity view controller that always presents modally, not from a popover.
    #[unsafe(super(UIActivityViewController))]
    #[thread_kind = MainThreadOnly]
    #[ivars = ModalActivityViewControllerIvars]
    struct ModalActivityViewController;

    impl ModalActivityViewController {

        #[unsafe(method(modalPresentationStyle))]
        fn modal_presentation_style(&self) -> UIModalPresentationStyle {
            UIModalPresentationStyle::FormSheet
        }
    }
);

impl ModalActivityViewController {
    /// Creates a new modal activity view controller.
    fn new(mtm: MainThreadMarker, dialog: &ShareDialog) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(ModalActivityViewControllerIvars {});
        let activities: Option<&NSArray<UIActivity>> = None;
        let items: Retained<NSMutableArray<NSURL>> = NSMutableArray::new();
        for path in dialog.paths.iter() {
            let Some(url) = NSURL::from_path(&path, false, None) else {
                continue;
            };
            items.addObject(&url);
        }
        unsafe {
            msg_send![super(this), initWithActivityItems: &*items, applicationActivities: activities]
        }
    }
}

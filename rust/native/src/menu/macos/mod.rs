use objc2::rc::Retained;
use objc2::runtime::{ProtocolObject, Sel};
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSApplication, NSControlStateValueOn, NSEvent, NSEventModifierFlags, NSEventType, NSMenu,
    NSMenuDelegate, NSMenuItem,
};
use objc2_foundation::{NSObject, NSObjectProtocol, NSPoint, NSString, ns_string};

use super::Menu;
use super::menu_item::MenuItem;

struct KeyEquivalent<'a> {
    key: &'a NSString,
    masks: Option<NSEventModifierFlags>,
}

/// Shows a native context menu.
pub fn show_context_menu(menu: Menu, callback: extern "C" fn(u64)) {
    dispatch2::run_on_main(|mtm| unsafe {
        let ns_menu = NSMenu::new(mtm);

        let Some(window) = NSApplication::sharedApplication(mtm).keyWindow() else {
            eprintln!("No key window found.");
            return;
        };

        // The current event should be the mouse down event if shown on mouse down.
        let mouse_down_event = window.currentEvent();

        let content_view = window.contentView();

        let action_handler = ContextMenuActionHandler::new(mtm, callback);
        ns_menu.setDelegate(Some(&ProtocolObject::from_ref(&*action_handler)));

        let location = NSPoint {
            x: menu.location.x,
            y: menu.location.y,
        };
        let mut selected_item = None;
        for item in menu.items {
            match item {
                MenuItem::Action(action_item) => {
                    let menu_item = NSMenuItem::new(mtm);
                    menu_item.setTitle(&NSString::from_str(&action_item.title));
                    menu_item.setAction(Some(sel!(handleAction:)));
                    menu_item.setTarget(Some(&action_handler));
                    menu_item.setTag(action_item.id as isize);
                    if action_item.is_selected {
                        menu_item.setState(NSControlStateValueOn);
                        selected_item = Some(menu_item.clone());
                    }
                    ns_menu.addItem(&menu_item);
                }
                MenuItem::Separator => {
                    ns_menu.addItem(&NSMenuItem::separatorItem(mtm));
                }
            }
        }

        ns_menu.popUpMenuPositioningItem_atLocation_inView(
            selected_item.as_deref(),
            location,
            content_view.as_deref(),
        );

        if let Some(mouse_down_event) = mouse_down_event {
            // If the menu is presented on a mouse down, a mouse up event won’t be created, which
            // will confuse Flutter, so we need to generate one and send it.
            match mouse_down_event.r#type() {
                NSEventType::LeftMouseDown | NSEventType::RightMouseDown => {
                    let up_event_type = match mouse_down_event.r#type() {
                        NSEventType::RightMouseDown => NSEventType::RightMouseUp,
                        _ => NSEventType::LeftMouseUp,
                    };
                    if let Some(mouse_up_event) = NSEvent::mouseEventWithType_location_modifierFlags_timestamp_windowNumber_context_eventNumber_clickCount_pressure(
                        up_event_type,
                        mouse_down_event.locationInWindow(),
                        mouse_down_event.modifierFlags(),
                        mouse_down_event.timestamp() + 0.3,
                        mouse_down_event.windowNumber(),
                        None,
                        0,
                        mouse_down_event.clickCount(),
                        mouse_down_event.pressure()) {
                         window.sendEvent(&mouse_up_event);
                    }
                }
                _ => (),
            }
        }
    });
}

/// Variables for the context menu action handler.
struct ContextMenuActionHandlerIvars {
    /// The callback.
    callback: extern "C" fn(u64),
}

define_class!(
    /// Defines a context menu action handler.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = ContextMenuActionHandlerIvars]
    struct ContextMenuActionHandler;

    unsafe impl NSObjectProtocol for ContextMenuActionHandler {}

    unsafe impl NSMenuDelegate for ContextMenuActionHandler {}

    impl ContextMenuActionHandler {
        /// Handles the action when the menu item is selected.
        #[unsafe(method(handleAction:))]
        fn handle_action(&self, menu_item: &NSMenuItem) {
            let id = unsafe { menu_item.tag() as u64 };
            self.send_event(id);
        }
    }
);

impl ContextMenuActionHandler {
    /// Creates a new context action menu handler.
    fn new(mtm: MainThreadMarker, callback: extern "C" fn(u64)) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(ContextMenuActionHandlerIvars { callback });
        unsafe { msg_send![super(this), init] }
    }

    /// Sends the event when an item is selected.
    fn send_event(&self, id: u64) {
        let callback = self.ivars().callback;
        callback(id);
    }
}

/// Initialise the default menu.
pub fn initialize_default() {
    dispatch2::run_on_main(|mtm| {
        let app = NSApplication::sharedApplication(mtm);
        let menubar = NSMenu::new(mtm);
        let app_menu_item = NSMenuItem::new(mtm);
        menubar.addItem(&app_menu_item);

        let app_menu = NSMenu::new(mtm);

        // About menu item
        let about_item_title = ns_string!("About APP_NAME");
        let about_item = menu_item(
            mtm,
            &about_item_title,
            Some(sel!(orderFrontStandardAboutPanel:)),
            None,
        );

        // Services menu item
        let services_menu = NSMenu::new(mtm);
        let services_item = menu_item(mtm, ns_string!("Services"), None, None);
        services_item.setSubmenu(Some(&services_menu));

        // Separator menu item
        let sep_first = NSMenuItem::separatorItem(mtm);

        // Hide application menu item
        let hide_item_title = ns_string!("Hide APP_NAME");
        let hide_item = menu_item(
            mtm,
            &hide_item_title,
            Some(sel!(hide:)),
            Some(KeyEquivalent {
                key: ns_string!("h"),
                masks: None,
            }),
        );

        // Hide other applications menu item
        let hide_others_item_title = ns_string!("Hide Others");
        let hide_others_item = menu_item(
            mtm,
            hide_others_item_title,
            Some(sel!(hideOtherApplications:)),
            Some(KeyEquivalent {
                key: ns_string!("h"),
                masks: Some(NSEventModifierFlags::Option | NSEventModifierFlags::Command),
            }),
        );

        // Show applications menu item
        let show_all_item_title = ns_string!("Show All");
        let show_all_item = menu_item(
            mtm,
            show_all_item_title,
            Some(sel!(unhideAllApplications:)),
            None,
        );

        // Separator menu item
        let sep = NSMenuItem::separatorItem(mtm);

        // Quit application menu item
        let quit_item_title = ns_string!("Quit APP_NAME");
        let quit_item = menu_item(
            mtm,
            &quit_item_title,
            Some(sel!(terminate:)),
            Some(KeyEquivalent {
                key: ns_string!("q"),
                masks: None,
            }),
        );

        app_menu.addItem(&about_item);
        app_menu.addItem(&sep_first);
        app_menu.addItem(&services_item);
        app_menu.addItem(&hide_item);
        app_menu.addItem(&hide_others_item);
        app_menu.addItem(&show_all_item);
        app_menu.addItem(&sep);
        app_menu.addItem(&quit_item);
        app_menu_item.setSubmenu(Some(&app_menu));

        unsafe { app.setServicesMenu(Some(&services_menu)) };

        // View menu
        let view_menu_item = NSMenuItem::new(mtm);
        unsafe { view_menu_item.setTitle(ns_string!("View")) };
        menubar.addItem(&view_menu_item);

        let view_menu = NSMenu::new(mtm);
        unsafe { view_menu.setTitle(ns_string!("View")) };

        let fullscreen_item = menu_item(
            mtm,
            ns_string!("Enter Full Screen"),
            Some(sel!(toggleFullScreen:)),
            Some(KeyEquivalent {
                key: ns_string!("f"),
                masks: Some(NSEventModifierFlags::Control | NSEventModifierFlags::Command),
            }),
        );
        view_menu.addItem(&fullscreen_item);
        view_menu_item.setSubmenu(Some(&view_menu));

        // Window menu
        let window_menu_item = NSMenuItem::new(mtm);
        unsafe { window_menu_item.setTitle(ns_string!("Window")) };
        menubar.addItem(&window_menu_item);

        let window_menu = NSMenu::new(mtm);
        unsafe { window_menu.setTitle(ns_string!("Window")) };

        let minimize_item = menu_item(
            mtm,
            ns_string!("Minimize"),
            Some(sel!(performMiniaturize:)),
            Some(KeyEquivalent {
                key: ns_string!("m"),
                masks: None,
            }),
        );
        let zoom_item = menu_item(mtm, ns_string!("Zoom"), Some(sel!(performZoom:)), None);
        let arrange_in_front_item = menu_item(
            mtm,
            ns_string!("Bring All to Front"),
            Some(sel!(arrangeInFront:)),
            None,
        );

        window_menu.addItem(&minimize_item);
        window_menu.addItem(&zoom_item);
        window_menu.addItem(&NSMenuItem::separatorItem(mtm));
        window_menu.addItem(&arrange_in_front_item);
        window_menu_item.setSubmenu(Some(&window_menu));

        // Help menu
        let help_menu_item = NSMenuItem::new(mtm);
        unsafe { help_menu_item.setTitle(ns_string!("Help")) };
        menubar.addItem(&help_menu_item);

        let help_menu = NSMenu::new(mtm);
        unsafe { help_menu.setTitle(ns_string!("Help")) };
        help_menu_item.setSubmenu(Some(&help_menu));

        app.setMainMenu(Some(&menubar));
    });
}

/// Creates a menu item.
fn menu_item(
    mtm: MainThreadMarker,
    title: &NSString,
    selector: Option<Sel>,
    key_equivalent: Option<KeyEquivalent<'_>>,
) -> Retained<NSMenuItem> {
    let (key, masks) = match key_equivalent {
        Some(ke) => (ke.key, ke.masks),
        None => (ns_string!(""), None),
    };
    let item = unsafe {
        NSMenuItem::initWithTitle_action_keyEquivalent(mtm.alloc(), title, selector, key)
    };
    if let Some(masks) = masks {
        item.setKeyEquivalentModifierMask(masks)
    }

    item
}

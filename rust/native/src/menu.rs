pub use menu::Menu;

#[cfg(target_os = "macos")]
pub mod macos;
mod menu;
mod menu_item;

/// Shows a native context menu.
pub fn show_context_menu(menu: Menu, callback: extern "C" fn(u64)) {
    #[cfg(target_os = "macos")]
    macos::show_context_menu(menu, callback);
}

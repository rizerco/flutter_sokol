use super::menu_item::MenuItem;

/// Defines a native menu.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Menu {
    /// The location at which to show the menu.
    pub location: Location,
    /// The items in the menu.
    pub items: Vec<MenuItem>,
}

/// The location to show the menu at.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Location {
    /// The x coordinate.
    pub x: f64,
    /// The y coordinate.
    pub y: f64,
}

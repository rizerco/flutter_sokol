/// An item that can be shown in a menu.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub enum MenuItem {
    /// An action item.
    Action(ActionItem),
    /// A separator item.
    Separator,
}

/// A menu item that performs an action.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ActionItem {
    /// The ID to identify the item.
    pub id: u64,
    /// The title for the action.
    pub title: String,
    /// Whether or not the item is selected.
    pub is_selected: bool,
}

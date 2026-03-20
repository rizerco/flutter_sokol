use std::path::PathBuf;

/// Defines a share dialogue.
#[derive(Clone)]
pub struct ShareDialog {
    /// The paths of the items to share.
    pub paths: Vec<PathBuf>,
}

impl ShareDialog {
    /// Shows the share dialogue.
    pub fn show(&self) {
        #[cfg(target_os = "ios")]
        {
            crate::dialog::ios::share::show_share_dialog(&self);
        }

        #[cfg(not(target_os = "ios"))]
        {
            println!("Share dialogue not supported on the current platform.");
        }
    }
}

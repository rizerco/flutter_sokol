use std::path::{Path, PathBuf};

/// Defines a file picker dialogue.
#[derive(Clone)]
pub struct FileDialog<'a> {
    /// The action the file dialogue is to perform.
    pub action: FileDialogAction<'a>,
}

impl<'a> FileDialog<'a> {
    /// Creates a new file dialogue for exporting a file.
    pub fn export<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        let action = FileDialogAction::Export {
            paths: vec![path.as_ref().to_path_buf()],
        };
        Self { action }
    }

    /// Creates a new file browser dialogue.
    pub fn browser() -> Self {
        Self {
            action: FileDialogAction::Browse,
        }
    }

    /// Creates a new file picker dialogue.
    pub fn picker(extensions: Vec<&'a str>) -> Self {
        Self {
            action: FileDialogAction::Pick { extensions },
        }
    }

    /// Creates a new image picker dialogue.
    pub fn image_picker() -> Self {
        Self {
            action: FileDialogAction::PickImage { use_camera: false },
        }
    }

    /// Launches the camera to import an image.
    pub fn camera() -> Self {
        Self {
            action: FileDialogAction::PickImage { use_camera: false },
        }
    }

    /// Shows the file picker dialogue.
    pub async fn show(&self) -> FileDialogResult {
        #[cfg(target_os = "ios")]
        {
            crate::dialog::ios::file::show_file_picker(&self).await
        }

        #[cfg(not(target_os = "ios"))]
        {
            crate::dialog::desktop::file::show_file_picker(&self).await
        }
    }
}

/// The action to configure the file dialogue for.
#[derive(Clone)]
pub enum FileDialogAction<'a> {
    /// Pick a file or files.
    Pick { extensions: Vec<&'a str> },
    /// Pick an image.
    PickImage { use_camera: bool },
    /// Browse for files. Could be the same as `Pick`,
    /// but on iOS it gives a more full featured file
    /// browser.
    Browse,
    /// Export files.
    Export { paths: Vec<PathBuf> },
}

/// The result when a file dialogue completes.
#[derive(Debug, Clone)]
pub enum FileDialogResult {
    /// The result when file picking was cancelled.
    Cancelled,
    /// The result when the file dialogue completes.
    Complete { selected_paths: Vec<PathBuf> },
}

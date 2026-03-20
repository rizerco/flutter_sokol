pub mod file {
    use std::path::PathBuf;

    use native_dialog::DialogBuilder;

    use crate::dialog::file::{FileDialog, FileDialogAction, FileDialogResult};

    /// Show a desktop file picker.
    pub async fn show_file_picker<'a>(dialog: &FileDialog<'a>) -> FileDialogResult {
        match &dialog.action {
            FileDialogAction::Pick { extensions } => select_file(extensions.clone()).await,
            FileDialogAction::PickImage { use_camera } => todo!(),
            FileDialogAction::Browse => select_file(Vec::new()).await,
            FileDialogAction::Export { paths } => export_file(paths.clone()).await,
        }
    }

    /// Shows the file picker for selecting a file.
    async fn select_file(extensions: Vec<&str>) -> FileDialogResult {
        let Ok(selected_file) = DialogBuilder::file().open_single_file().spawn().await else {
            eprintln!("Error creating file dialogue.");
            return FileDialogResult::Cancelled;
        };
        match selected_file {
            Some(path) => FileDialogResult::Complete {
                selected_paths: vec![path],
            },
            None => FileDialogResult::Cancelled,
        }
    }

    /// Shows the file picker for export a file.
    async fn export_file(paths: Vec<PathBuf>) -> FileDialogResult {
        if paths.len() != 1 {
            todo!("Handle multple paths.");
        }
        let mut action = DialogBuilder::file().save_single_file();
        action.location = paths.first().cloned();
        // How can we get formats in here? We’ll probably need to fork the plugin.
        action.filename = Some("Untitled.pixaki".to_string());
        let Ok(new_path) = action.spawn().await else {
            eprintln!("Error creating file dialogue.");
            return FileDialogResult::Cancelled;
        };
        match new_path {
            Some(path) => FileDialogResult::Complete {
                selected_paths: vec![path],
            },
            None => FileDialogResult::Cancelled,
        }
    }
}

pub(crate) mod message {
    use native_dialog::DialogBuilder;

    use crate::dialog::message::{MessageDialog, MessageDialogOutput, MessageDialogResult};

    pub(crate) async fn show_alert(dialog: &MessageDialog) -> MessageDialogResult {
        let result = DialogBuilder::message()
            .set_title(&dialog.title)
            .set_text(&dialog.message)
            .confirm()
            .spawn()
            .await;
        let Ok(result) = result else {
            eprintln!("Error creating message dialogue.");
            return MessageDialogResult::Dismissed(MessageDialogOutput {
                selected_index: 0,
                entered_text: Vec::new(),
            });
        };
        match result {
            true => MessageDialogResult::Confirmed(MessageDialogOutput {
                selected_index: 0,
                entered_text: Vec::new(),
            }),
            false => MessageDialogResult::Dismissed(MessageDialogOutput {
                selected_index: 0,
                entered_text: Vec::new(),
            }),
        }
    }
}

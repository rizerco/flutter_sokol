use std::pin::Pin;
use std::ptr::NonNull;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

use block2::RcBlock;
use objc2::{self, sel};
use objc2_foundation::{NSObjectNSDelayedPerforming, NSString};
use objc2_ui_kit::{
    self, UIAlertAction, UIAlertActionStyle, UIAlertController, UIAlertControllerStyle,
    UITextField, UITextInputTraits,
};

use crate::dialog::message::{
    ActionStyle, ActionType, MessageDialog, MessageDialogOutput, MessageDialogResult,
};

/// Defines the shared state for the alert future.
struct SharedState {
    /// The result.
    result: Option<MessageDialogResult>,
    /// The waker for the future.
    waker: Option<Waker>,
}

/// The future for the alert.
pub struct AlertFuture {
    state: Arc<Mutex<SharedState>>,
}

impl Future for AlertFuture {
    type Output = MessageDialogResult;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.lock().unwrap();
        if let Some(result) = state.result.take() {
            Poll::Ready(result)
        } else {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

/// Show an iOS alert.
pub(crate) fn show_alert(dialog: &MessageDialog) -> AlertFuture {
    unsafe {
        let state = Arc::new(Mutex::new(SharedState {
            result: None,
            waker: None,
        }));
        dispatch2::run_on_main(|mtm| {
            let root = super::root_view_controller(mtm);

            let title = NSString::from_str(&dialog.title);
            let message = NSString::from_str(&dialog.message);
            let alert = UIAlertController::alertControllerWithTitle_message_preferredStyle(
                Some(&title),
                Some(&message),
                UIAlertControllerStyle::Alert,
                mtm,
            );

            for text_field in dialog.text_fields.iter() {
                let text_field = text_field.clone();
                alert.addTextFieldWithConfigurationHandler(Some(&RcBlock::new(
                    move |ui_text_field: NonNull<UITextField>| {
                        let ui_text_field = ui_text_field.as_ref();
                        let text = text_field
                            .text
                            .clone()
                            .map(|text| NSString::from_str(&text));
                        ui_text_field.setText(text.as_deref());

                        let placeholder = text_field
                            .placeholder
                            .clone()
                            .map(|placeholder| NSString::from_str(&placeholder));
                        ui_text_field.setPlaceholder(placeholder.as_deref());

                        ui_text_field.setSecureTextEntry(text_field.is_secure);

                        if text_field.autoselect {
                            // If I could work out how to dispatch onto the main thread
                            // and pass the UITextField in, that would work better.
                            // This workd with a delay of 0.0, but it means that one of the
                            // selection handles gets cut off by the text field. With a delay
                            // of 0.3, it usually doesn’t, but sometimes does.
                            ui_text_field.performSelector_withObject_afterDelay(
                                sel!(selectAll:),
                                None,
                                0.3,
                            );
                        }
                    },
                )));
            }

            for (index, action) in dialog.actions.iter().enumerate() {
                let action = action.clone();
                let title = NSString::from_str(&action.title);
                let alert_clone = alert.clone();
                let state_clone = Arc::clone(&state);
                let block = RcBlock::new(move |_| {
                    let mut state = state_clone.lock().unwrap();
                    if state.result.is_none() {
                        let entered_text: Vec<String> = alert_clone
                            .textFields()
                            .unwrap_or_default()
                            .iter()
                            .filter_map(|text_field| {
                                text_field.text().map(|ns_string| ns_string.to_string())
                            })
                            .collect();
                        let output = MessageDialogOutput {
                            selected_index: index as _,
                            entered_text,
                        };
                        let result = match action.action_type {
                            ActionType::Dismiss => MessageDialogResult::Dismissed(output),
                            ActionType::Confirm => MessageDialogResult::Confirmed(output),
                        };
                        state.result = Some(result);
                        if let Some(waker) = state.waker.take() {
                            waker.wake();
                        }
                    }
                });
                let style = match action.style {
                    ActionStyle::Default => UIAlertActionStyle::Default,
                    ActionStyle::Cancel => UIAlertActionStyle::Cancel,
                    ActionStyle::Destructive => UIAlertActionStyle::Destructive,
                };
                let alert_action = UIAlertAction::actionWithTitle_style_handler(
                    Some(&title),
                    style,
                    Some(&block),
                    mtm,
                );
                alert.addAction(&alert_action);
            }

            root.presentViewController_animated_completion(&alert, true, None);
        });
        AlertFuture { state }
    }
}

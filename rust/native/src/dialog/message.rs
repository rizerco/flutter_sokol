/// Defines a message dialogue.
#[derive(Default, Clone)]
pub struct MessageDialog {
    /// The dialogue title.
    pub title: String,
    /// The dialoge message.
    pub message: String,
    /// The actions.
    pub actions: Vec<MessageDialogAction>,
    /// The text fields.
    pub text_fields: Vec<TextInputConfiguration>,
}

impl MessageDialog {
    /// Creates a new message dialogue with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the title.
    pub fn set_title<S>(&mut self, title: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.title = title.as_ref().to_string().clone();
        self
    }

    /// Sets the message.
    pub fn set_message<S>(&mut self, message: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.message = message.as_ref().to_string().clone();
        self
    }

    /// Adds an action.
    pub fn add_action<S>(
        &mut self,
        title: S,
        action_type: ActionType,
        style: ActionStyle,
    ) -> &mut Self
    where
        S: AsRef<str>,
    {
        let action = MessageDialogAction {
            title: title.as_ref().to_string().clone(),
            action_type,
            style,
        };
        self.actions.push(action);
        self
    }

    /// Adds a confirmation button.
    pub fn add_confirm<S>(&mut self, title: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.add_action(title, ActionType::Confirm, ActionStyle::Default)
    }

    /// Adds a destructive confirmation button.
    pub fn add_destructive_confirm<S>(&mut self, title: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.add_action(title, ActionType::Confirm, ActionStyle::Destructive)
    }

    /// Adds a dismiss button.
    pub fn add_dismiss<S>(&mut self, title: S) -> &mut Self
    where
        S: AsRef<str>,
    {
        self.add_action(title, ActionType::Dismiss, ActionStyle::Cancel)
    }

    /// Adds an OK button.
    pub fn add_ok(&mut self) -> &mut Self {
        self.add_dismiss("OK")
    }

    /// Adds a cancel button.
    pub fn add_cancel(&mut self) -> &mut Self {
        self.add_dismiss("Cancel")
    }

    /// Adds a text field.
    pub fn add_text_input(&mut self, configuration: TextInputConfiguration) -> &mut Self {
        self.text_fields.push(configuration);
        self
    }

    /// Shows the dialogue.
    pub async fn show(&self) -> MessageDialogResult {
        #[cfg(target_os = "ios")]
        {
            crate::dialog::ios::message::show_alert(self).await
        }

        #[cfg(not(target_os = "ios"))]
        {
            crate::dialog::desktop::message::show_alert(self).await
        }
    }
}

/// The configuration for a text input box.
#[derive(Debug, Default, Clone)]
pub struct TextInputConfiguration {
    /// The existing text.
    pub text: Option<String>,
    /// The placeholder text.
    pub placeholder: Option<String>,
    /// Whether or not to automatically select the text when the dialogue appears.
    pub autoselect: bool,
    /// Whether or not this is a secure text input.
    pub is_secure: bool,
}

/// An action that can be added to the message dialogue.
#[derive(Clone)]
pub struct MessageDialogAction {
    /// The action title.
    pub title: String,
    /// The action type.
    pub action_type: ActionType,
    /// The action style.
    pub style: ActionStyle,
}

/// The type of action.
#[derive(Debug, Clone)]
pub enum ActionType {
    /// Dismisses the dialogue.
    Dismiss,
    /// Confirms the action.
    Confirm,
}

/// The style for an action.
#[derive(Debug, Clone)]
pub enum ActionStyle {
    /// The default action style.
    Default,
    /// The style for cancel buttons.
    Cancel,
    /// The style for destuctive actions.
    Destructive,
}

/// The output when a button is pressed on the message dialogue.
#[derive(Debug, Clone)]
pub struct MessageDialogOutput {
    /// The selected action index.
    pub selected_index: u32,
    /// The entered text, if any.
    pub entered_text: Vec<String>,
}

/// The result when a button is pressed on the message dialogue.
#[derive(Debug, Clone)]
pub enum MessageDialogResult {
    /// The result when a dismiss action was triggered.
    Dismissed(MessageDialogOutput),
    /// The result when a confirmation action was triggered.
    Confirmed(MessageDialogOutput),
}

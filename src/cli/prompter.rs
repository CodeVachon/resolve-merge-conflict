//! The real [`Prompter`]: inquire for questions, `$EDITOR` for amending the prompt.

use std::ffi::OsString;
use std::io::IsTerminal;

use anyhow::{Result, anyhow};
use inquire::error::InquireError;
use inquire::{Editor, Select, Text};

use crate::runner::Prompter;
use crate::ui;

/// The user backed out (Esc or Ctrl-C) at a question. Not a failure: nothing was changed.
#[derive(Debug, thiserror::Error)]
#[error("cancelled")]
pub struct Cancelled;

fn map_error(error: InquireError) -> anyhow::Error {
    match error {
        InquireError::OperationCanceled | InquireError::OperationInterrupted => {
            anyhow!(Cancelled)
        }
        other => anyhow!(other),
    }
}

/// True when both stdin and stdout are terminals, which interactive questions need.
pub fn is_interactive() -> bool {
    std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}

pub struct InquirePrompter {
    editor: Option<String>,
    /// Send status text to stderr, keeping stdout free for `--json`.
    to_stderr: bool,
}

impl InquirePrompter {
    /// `editor` is the config's `editor`; otherwise `$VISUAL`, then `$EDITOR`.
    pub fn new(editor: Option<String>, to_stderr: bool) -> Self {
        Self { editor, to_stderr }
    }

    fn say(&self, text: &str) {
        if self.to_stderr {
            eprintln!("{text}");
        } else {
            println!("{text}");
        }
    }
}

/// `"code --wait"` as a program and its arguments.
pub fn split_editor(command: &str) -> Option<(OsString, Vec<OsString>)> {
    let mut parts = command.split_whitespace();
    let program = parts.next()?;
    Some((program.into(), parts.map(OsString::from).collect()))
}

impl Prompter for InquirePrompter {
    fn select(
        &mut self,
        question: &str,
        options: &[String],
        default: Option<usize>,
    ) -> Result<usize> {
        let mut select = Select::new(question, options.to_vec());
        if let Some(index) = default {
            select = select.with_starting_cursor(index);
        }
        select
            .raw_prompt()
            .map(|choice| choice.index)
            .map_err(map_error)
    }

    fn text(&mut self, question: &str, default: Option<&str>) -> Result<String> {
        let mut text = Text::new(question);
        if let Some(default) = default {
            text = text.with_default(default);
        }
        text.prompt().map_err(map_error)
    }

    fn edit(&mut self, question: &str, initial: &str) -> Result<String> {
        let mut editor = Editor::new(question)
            .with_predefined_text(initial)
            .with_file_extension(".md");
        let parsed = self.editor.as_deref().and_then(split_editor);
        if let Some((program, args)) = &parsed {
            let args: Vec<&std::ffi::OsStr> = args.iter().map(|arg| arg.as_os_str()).collect();
            editor = editor.with_editor_command(program).with_args(&args);
            return editor.prompt().map_err(map_error);
        }
        editor.prompt().map_err(map_error)
    }

    fn show(&mut self, title: &str, body: &str) {
        self.say(&ui::block(title, body));
    }

    fn info(&mut self, message: &str) {
        self.say(message);
    }

    fn working(&mut self, message: &str) -> Box<dyn std::any::Any> {
        Box::new(ui::Spinner::start(message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_editor_command_is_split_into_program_and_arguments() {
        let (program, args) = split_editor("code --wait --new-window").unwrap();
        assert_eq!(program, "code");
        assert_eq!(args, ["--wait", "--new-window"]);
        assert!(split_editor("   ").is_none());
    }

    #[test]
    fn cancelling_a_question_is_not_a_failure() {
        let error = map_error(InquireError::OperationCanceled);
        assert!(error.downcast_ref::<Cancelled>().is_some());
        let error = map_error(InquireError::OperationInterrupted);
        assert!(error.downcast_ref::<Cancelled>().is_some());
        let error = map_error(InquireError::NotTTY);
        assert!(error.downcast_ref::<Cancelled>().is_none());
    }
}

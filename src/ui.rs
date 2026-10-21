//! Small helpers for what goes to the terminal: colors, titles, blocks of text.
//!
//! Colors are on only when the stream is a terminal and `NO_COLOR` is unset, so piped output and
//! test output stay plain.

use std::io::IsTerminal;

use owo_colors::OwoColorize;

fn color_on(stream_is_terminal: bool) -> bool {
    stream_is_terminal && std::env::var_os("NO_COLOR").is_none()
}

pub fn stdout_color() -> bool {
    color_on(std::io::stdout().is_terminal())
}

pub fn stderr_color() -> bool {
    color_on(std::io::stderr().is_terminal())
}

pub fn bold(text: &str) -> String {
    if stdout_color() {
        text.bold().to_string()
    } else {
        text.to_string()
    }
}

pub fn dim(text: &str) -> String {
    if stdout_color() {
        text.dimmed().to_string()
    } else {
        text.to_string()
    }
}

pub fn green(text: &str) -> String {
    if stdout_color() {
        text.green().to_string()
    } else {
        text.to_string()
    }
}

pub fn yellow(text: &str) -> String {
    if stdout_color() {
        text.yellow().to_string()
    } else {
        text.to_string()
    }
}

pub fn red(text: &str) -> String {
    if stdout_color() {
        text.red().to_string()
    } else {
        text.to_string()
    }
}

/// The banner printed at the start of an interactive run.
pub fn title() -> String {
    format!(
        "{} {}",
        bold("resolve-merge-conflicts"),
        dim(&format!("v{}", crate::VERSION))
    )
}

/// `text` indented under a rule, for showing the prompt.
pub fn block(heading: &str, body: &str) -> String {
    let rule = dim(&"─".repeat(60));
    let mut out = format!("{rule}\n{}\n{rule}\n", bold(heading));
    out.push_str(body.trim_end());
    out.push('\n');
    out.push_str(&rule);
    out
}

/// The error shape used everywhere: a red heading and the message.
pub fn error_block(message: &str) -> String {
    let heading = if stderr_color() {
        "Error".red().bold().to_string()
    } else {
        "Error".to_string()
    };
    format!("{heading}: {message}")
}

/// A line on stderr that shows elapsed time while something slow runs. Does nothing unless stderr
/// is a terminal, so logs and pipes stay clean. Stops when dropped.
pub struct Spinner {
    stop: Option<std::sync::mpsc::Sender<()>>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Spinner {
    pub fn start(message: &str) -> Self {
        if !std::io::stderr().is_terminal() {
            eprintln!("{message}...");
            return Self {
                stop: None,
                handle: None,
            };
        }
        let (stop, rx) = std::sync::mpsc::channel::<()>();
        let message = message.to_string();
        let handle = std::thread::spawn(move || {
            let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let started = std::time::Instant::now();
            let mut tick = 0;
            while rx
                .recv_timeout(std::time::Duration::from_millis(100))
                .is_err()
            {
                let elapsed = started.elapsed().as_secs();
                eprint!(
                    "\r{} {message} ({}m{:02}s)  ",
                    frames[tick % frames.len()],
                    elapsed / 60,
                    elapsed % 60
                );
                tick += 1;
            }
            eprint!("\r{}\r", " ".repeat(message.len() + 24));
        });
        Self {
            stop: Some(stop),
            handle: Some(handle),
        }
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_needs_a_terminal() {
        assert!(!color_on(false));
    }

    #[test]
    fn block_wraps_the_body_between_rules() {
        let block = block("Prompt", "line one\nline two\n");
        assert!(block.contains("Prompt"));
        assert!(block.contains("line one\nline two\n"));
        assert!(block.matches('─').count() >= 120);
    }
}

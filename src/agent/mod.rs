//! Agents: the CLI tools (Claude Code, Codex) that actually resolve the conflicts.
//!
//! An [`Agent`] only *describes* how to invoke its CLI ([`Invocation`]) and how to read what it
//! printed; [`run`] executes that description. Keeping the two apart makes the exact flags
//! testable without a real agent installed.
//!
//! The contract with every agent: edit the conflicted files, nothing else. Staging is done by this
//! tool afterwards, and no agent is ever given a way to commit, push, or continue an operation.

pub mod claude;
pub mod codex;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

/// What to run an agent on.
#[derive(Debug, Clone)]
pub struct AgentRequest {
    pub prompt: String,
    /// Model name or alias, passed through as given.
    pub model: String,
    /// The repository root; the agent works here.
    pub cwd: PathBuf,
}

/// A fully described process: program, arguments, and what to feed its stdin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invocation {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub stdin: String,
    pub cwd: PathBuf,
    /// A file the agent writes its final message to, read after it exits in place of stdout.
    /// Removed once read.
    pub output_file: Option<PathBuf>,
}

/// What came back from an agent run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentOutcome {
    /// The process exited 0 and did not report an error of its own.
    pub success: bool,
    pub exit_code: Option<i32>,
    /// The agent's final message: the structured summary, when it followed the prompt.
    pub final_message: String,
    pub stderr: String,
}

pub trait Agent {
    /// Short name, as shown in the agent picker and used for `--agent`.
    fn name(&self) -> &'static str;

    /// The executable looked up on `PATH` (unless overridden).
    fn binary(&self) -> &'static str;

    /// The program actually run: `binary()`, or the override from the environment.
    fn program(&self) -> &Path;

    /// How to run `request`.
    fn invocation(&self, request: &AgentRequest) -> Invocation;

    /// Extract the final message from the process's stdout, and whether the agent itself
    /// reported failure.
    fn parse_output(&self, stdout: &str) -> ParsedOutput;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedOutput {
    pub final_message: String,
    pub is_error: bool,
}

/// Run `agent` for `request` and wait for it to finish.
pub fn run(agent: &dyn Agent, request: &AgentRequest) -> Result<AgentOutcome> {
    let invocation = agent.invocation(request);
    let mut child = Command::new(&invocation.program)
        .args(&invocation.args)
        .current_dir(&invocation.cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("could not start {}", invocation.program.display()))?;

    // Feed the prompt on a thread: a prompt larger than the pipe buffer would otherwise deadlock
    // against an agent that writes output before it has finished reading.
    let mut stdin = child.stdin.take().context("agent stdin was not piped")?;
    let prompt = invocation.stdin.clone();
    let writer = std::thread::spawn(move || {
        let _ = stdin.write_all(prompt.as_bytes());
    });
    let output = child.wait_with_output().context("waiting for the agent")?;
    let _ = writer.join();

    let mut stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    if let Some(file) = &invocation.output_file {
        if let Ok(contents) = std::fs::read_to_string(file)
            && !contents.trim().is_empty()
        {
            stdout = contents;
        }
        let _ = std::fs::remove_file(file);
    }
    let parsed = agent.parse_output(&stdout);
    Ok(AgentOutcome {
        success: output.status.success() && !parsed.is_error,
        exit_code: output.status.code(),
        final_message: parsed.final_message,
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

/// The agents this tool knows, by name.
pub fn names() -> &'static [&'static str] {
    &["claude", "codex"]
}

/// Whether `binary` resolves to an executable on `PATH`.
pub fn is_installed(binary: &str) -> bool {
    find_on_path(binary).is_some()
}

/// Whether `program` can be run: an executable file if it has a directory part, else on `PATH`.
pub fn program_available(program: &Path) -> bool {
    if program.components().count() > 1 {
        is_executable(program)
    } else {
        find_on_path(&program.to_string_lossy()).is_some()
    }
}

fn find_on_path(binary: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(binary))
        .find(|candidate| is_executable(candidate))
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .map(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file() || path.with_extension("exe").is_file()
}

/// Fail with a clear message when `agent`'s binary is missing.
pub fn ensure_installed(agent: &dyn Agent) -> Result<()> {
    if program_available(agent.program()) {
        Ok(())
    } else {
        bail!(
            "`{}` was not found on PATH; install {} first",
            agent.binary(),
            agent.name()
        )
    }
}

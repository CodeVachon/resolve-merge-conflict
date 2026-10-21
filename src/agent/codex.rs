//! Codex, driven non-interactively with `codex exec`.
//!
//! The sandbox is `workspace-write` with approvals off, so Codex can edit the conflicted files but
//! is never stopped to ask permission. It does not need git write access: this tool stages the
//! files itself afterwards, which is what removes Codex's `git add` approval prompt.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use super::{Agent, AgentRequest, Invocation, ParsedOutput};

static COUNTER: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone)]
pub struct CodexAgent {
    program: PathBuf,
}

impl CodexAgent {
    pub fn new() -> Self {
        Self {
            program: std::env::var_os("RESOLVE_MERGE_CONFLICTS_CODEX_BIN")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("codex")),
        }
    }

    /// Use a specific executable, for tests.
    pub fn with_program(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
        }
    }
}

impl Default for CodexAgent {
    fn default() -> Self {
        Self::new()
    }
}

fn last_message_path() -> PathBuf {
    std::env::temp_dir().join(format!(
        "resolve-merge-conflicts-codex-{}-{}.txt",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ))
}

impl Agent for CodexAgent {
    fn name(&self) -> &'static str {
        "codex"
    }

    fn binary(&self) -> &'static str {
        "codex"
    }

    fn invocation(&self, request: &AgentRequest) -> Invocation {
        let output_file = last_message_path();
        let args = vec![
            "exec".to_string(),
            "--model".to_string(),
            request.model.clone(),
            "--sandbox".to_string(),
            "workspace-write".to_string(),
            "-c".to_string(),
            "approval_policy=\"never\"".to_string(),
            "--ephemeral".to_string(),
            "--cd".to_string(),
            request.cwd.to_string_lossy().into_owned(),
            "--output-last-message".to_string(),
            output_file.to_string_lossy().into_owned(),
            // `-` reads the prompt from stdin.
            "-".to_string(),
        ];
        Invocation {
            program: self.program.clone(),
            args,
            stdin: request.prompt.clone(),
            cwd: request.cwd.clone(),
            output_file: Some(output_file),
        }
    }

    fn parse_output(&self, stdout: &str) -> ParsedOutput {
        ParsedOutput {
            final_message: stdout.trim().to_string(),
            is_error: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invocation_is_sandboxed_never_asks_and_reads_the_prompt_from_stdin() {
        let request = AgentRequest {
            prompt: "resolve it".into(),
            model: "gpt-x".into(),
            cwd: "/repo".into(),
        };
        let invocation = CodexAgent::with_program("codex").invocation(&request);
        let args = &invocation.args;
        let after = |flag: &str| &args[args.iter().position(|a| a == flag).unwrap() + 1];

        assert_eq!(args[0], "exec");
        assert_eq!(after("--model"), "gpt-x");
        assert_eq!(after("--sandbox"), "workspace-write");
        assert_eq!(after("-c"), "approval_policy=\"never\"");
        assert_eq!(after("--cd"), "/repo");
        assert_eq!(args.last().unwrap(), "-");
        assert_eq!(invocation.stdin, "resolve it");
        assert!(args.contains(&"--ephemeral".to_string()));
        assert!(
            !args.iter().any(|a| a.contains("dangerously")),
            "must never bypass the sandbox"
        );
        let file = invocation.output_file.unwrap();
        assert_eq!(after("--output-last-message"), &file.to_string_lossy());
    }

    #[test]
    fn each_invocation_gets_its_own_output_file() {
        let request = AgentRequest {
            prompt: String::new(),
            model: "m".into(),
            cwd: "/r".into(),
        };
        let agent = CodexAgent::with_program("codex");
        assert_ne!(
            agent.invocation(&request).output_file,
            agent.invocation(&request).output_file
        );
    }
}

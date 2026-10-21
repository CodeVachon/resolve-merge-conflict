//! Claude Code, driven non-interactively with `claude -p`.

use std::path::PathBuf;

use super::{Agent, AgentRequest, Invocation, ParsedOutput};

/// Read-only git the agent may use to understand the conflict.
const ALLOWED_TOOLS: &[&str] = &[
    "Read",
    "Edit",
    "Write",
    "Grep",
    "Glob",
    "Bash(git diff:*)",
    "Bash(git show:*)",
    "Bash(git log:*)",
    "Bash(git status:*)",
    "Bash(git ls-files:*)",
    "Bash(git blame:*)",
];

/// Everything that changes git state. Denied even though `--permission-prompts none` already
/// refuses anything not allowed above: the promise not to commit or push should not rest on one
/// flag.
const DISALLOWED_TOOLS: &[&str] = &[
    "Bash(git add:*)",
    "Bash(git commit:*)",
    "Bash(git push:*)",
    "Bash(git merge:*)",
    "Bash(git rebase:*)",
    "Bash(git cherry-pick:*)",
    "Bash(git revert:*)",
    "Bash(git reset:*)",
    "Bash(git checkout:*)",
    "Bash(git switch:*)",
    "Bash(git restore:*)",
    "Bash(git stash:*)",
    "Bash(git clean:*)",
    "Bash(git rm:*)",
    "Bash(git mv:*)",
    "Bash(git tag:*)",
    "Bash(git branch:*)",
    "Bash(git config:*)",
];

#[derive(Debug, Clone)]
pub struct ClaudeAgent {
    program: PathBuf,
}

impl ClaudeAgent {
    pub fn new() -> Self {
        Self {
            program: std::env::var_os("RESOLVE_MERGE_CONFLICTS_CLAUDE_BIN")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("claude")),
        }
    }

    /// Use a specific executable, for tests.
    pub fn with_program(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
        }
    }
}

impl Default for ClaudeAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl Agent for ClaudeAgent {
    fn name(&self) -> &'static str {
        "claude"
    }

    fn program(&self) -> &std::path::Path {
        &self.program
    }

    fn binary(&self) -> &'static str {
        "claude"
    }

    fn invocation(&self, request: &AgentRequest) -> Invocation {
        let args = vec![
            "--print".to_string(),
            "--model".to_string(),
            request.model.clone(),
            "--output-format".to_string(),
            "json".to_string(),
            "--permission-mode".to_string(),
            "acceptEdits".to_string(),
            "--permission-prompts".to_string(),
            "none".to_string(),
            "--no-session-persistence".to_string(),
            "--allowedTools".to_string(),
            ALLOWED_TOOLS.join(","),
            "--disallowedTools".to_string(),
            DISALLOWED_TOOLS.join(","),
        ];
        Invocation {
            program: self.program.clone(),
            args,
            // On stdin, not as an argument: the tool lists above are variadic and would swallow a
            // trailing positional prompt.
            stdin: request.prompt.clone(),
            cwd: request.cwd.clone(),
            output_file: None,
        }
    }

    fn parse_output(&self, stdout: &str) -> ParsedOutput {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(stdout.trim()) else {
            return ParsedOutput {
                final_message: stdout.trim().to_string(),
                is_error: false,
            };
        };
        ParsedOutput {
            final_message: value
                .get("result")
                .and_then(|result| result.as_str())
                .unwrap_or_default()
                .to_string(),
            is_error: value
                .get("is_error")
                .and_then(|flag| flag.as_bool())
                .unwrap_or(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> AgentRequest {
        AgentRequest {
            prompt: "resolve it".into(),
            model: "sonnet".into(),
            cwd: "/repo".into(),
        }
    }

    #[test]
    fn invocation_is_non_interactive_and_cannot_change_git_state() {
        let invocation = ClaudeAgent::with_program("claude").invocation(&request());
        let args = &invocation.args;
        let after = |flag: &str| &args[args.iter().position(|a| a == flag).unwrap() + 1];

        assert_eq!(after("--model"), "sonnet");
        assert_eq!(after("--permission-mode"), "acceptEdits");
        assert_eq!(after("--permission-prompts"), "none");
        assert!(args.contains(&"--print".to_string()));
        assert_eq!(invocation.stdin, "resolve it");
        assert_eq!(invocation.cwd, PathBuf::from("/repo"));

        let allowed = after("--allowedTools");
        for needed in ["Edit", "Read", "Bash(git diff:*)"] {
            assert!(allowed.split(',').any(|tool| tool == needed), "{needed}");
        }
        for forbidden in [
            "git add",
            "git commit",
            "git push",
            "git merge",
            "git rebase",
        ] {
            assert!(
                !allowed.contains(forbidden),
                "{forbidden} must not be allowed"
            );
            assert!(
                after("--disallowedTools").contains(forbidden),
                "{forbidden} must be disallowed"
            );
        }
    }

    #[test]
    fn parses_the_result_and_the_error_flag() {
        let agent = ClaudeAgent::with_program("claude");
        let ok = agent.parse_output(r#"{"type":"result","is_error":false,"result":"done"}"#);
        assert_eq!(ok.final_message, "done");
        assert!(!ok.is_error);

        let failed = agent.parse_output(r#"{"is_error":true,"result":"boom"}"#);
        assert!(failed.is_error);
    }

    #[test]
    fn non_json_output_is_kept_as_the_message() {
        let parsed = ClaudeAgent::with_program("claude").parse_output("plain text\n");
        assert_eq!(parsed.final_message, "plain text");
        assert!(!parsed.is_error);
    }
}

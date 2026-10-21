//! The prompt handed to the agent: a template with `{{placeholders}}` filled from the conflict.
//!
//! Plain substitution, no template engine. The user can replace the whole template in config (or
//! amend the rendered prompt at run time); what they cannot do by accident is leave a typo'd
//! placeholder in it, because unknown names are an error rather than being sent to the agent.

use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::config::Config;
use crate::conflicts::ConflictState;

/// Every placeholder a template may use.
pub const PLACEHOLDERS: &[&str] = &[
    "files",
    "operation",
    "ours",
    "theirs",
    "branch",
    "summary_format",
];

/// What the agent is asked to end with, so the report can say what was chosen and why. Parsed
/// tolerantly: an agent that does not follow it still has its raw message shown.
pub const SUMMARY_FORMAT: &str = r#"When you are done, finish with a single fenced JSON block and nothing after it, in exactly this shape:

```json
{"files": [{"path": "<path>", "strategy": "ours | theirs | merged | deleted", "summary": "<one sentence: what you kept from each side and why>"}]}
```"#;

pub const DEFAULT_TEMPLATE: &str = r#"This repository is in the middle of a git {{operation}} that stopped on conflicts. Resolve them.

In this {{operation}}, "ours" is {{ours}} and "theirs" is {{theirs}}.

Conflicted files:
{{files}}

How to work:
- Open each conflicted file, understand what each side was trying to do, and write the correct merged result. Keep the intent of both sides; do not just pick one unless one side is clearly superseded.
- Remove every conflict marker (<<<<<<<, =======, >>>>>>>, and ||||||| if present). The result must be valid, coherent code.
- Where a file was deleted on one side and modified on the other, decide whether it should exist and either keep your merged version or delete the file.
- If the conflict touches code that other files depend on (imports, call sites, types, tests), read enough of the surrounding code to make the result consistent.
- Only edit the conflicted files listed above. Do not touch any other file.
- Do NOT run git add, git commit, git push, git merge, git rebase, git checkout, git reset, git stash or any other command that changes git state, and do not continue or abort the {{operation}}. Staging and committing are left to the user. Read-only commands such as git diff, git show and git log are fine.

{{summary_format}}"#;

/// What a template is filled from.
pub struct PromptContext<'a> {
    pub state: &'a ConflictState,
    /// The current branch, if `HEAD` is on one.
    pub branch: Option<&'a str>,
}

/// Names used in `template` that are not in [`PLACEHOLDERS`].
pub fn unknown_placeholders(template: &str) -> Vec<String> {
    let mut unknown = Vec::new();
    for name in placeholder_names(template) {
        if !PLACEHOLDERS.contains(&name.as_str()) && !unknown.contains(&name) {
            unknown.push(name);
        }
    }
    unknown
}

fn placeholder_names(template: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        let after = &rest[start + 2..];
        let Some(end) = after.find("}}") else { break };
        names.push(after[..end].trim().to_string());
        rest = &after[end + 2..];
    }
    names
}

/// Fill `template`. Unknown placeholders are an error naming them.
pub fn render(template: &str, context: &PromptContext<'_>) -> Result<String> {
    let unknown = unknown_placeholders(template);
    if !unknown.is_empty() {
        bail!(
            "unknown placeholder{} in the prompt: {} (known: {})",
            if unknown.len() == 1 { "" } else { "s" },
            unknown
                .iter()
                .map(|name| format!("{{{{{name}}}}}"))
                .collect::<Vec<_>>()
                .join(", "),
            PLACEHOLDERS
                .iter()
                .map(|name| format!("{{{{{name}}}}}"))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    let files = context
        .state
        .files
        .iter()
        .map(|file| format!("- {} ({})", file.path, file.kind.describe()))
        .collect::<Vec<_>>()
        .join("\n");

    let mut out = String::with_capacity(template.len() + files.len());
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let Some(end) = after.find("}}") else {
            out.push_str(&rest[start..]);
            rest = "";
            break;
        };
        let value = match after[..end].trim() {
            "files" => files.clone(),
            "operation" => context.state.operation.describe().to_string(),
            "ours" => context.state.ours.clone(),
            "theirs" => context.state.theirs.clone(),
            "branch" => context.branch.unwrap_or("(detached HEAD)").to_string(),
            "summary_format" => SUMMARY_FORMAT.to_string(),
            _ => unreachable!("validated above"),
        };
        out.push_str(&value);
        rest = &after[end + 2..];
    }
    out.push_str(rest);
    Ok(out)
}

/// The template to use: `promptFile` (relative paths resolve against `base`, normally the config
/// file's directory), else `prompt`, else the built-in default.
pub fn template(config: &Config, base: Option<&Path>) -> Result<String> {
    if let Some(file) = &config.prompt_file {
        let path = if file.is_absolute() {
            file.clone()
        } else if let Some(stripped) = file
            .strip_prefix("~")
            .ok()
            .and_then(|rest| dirs::home_dir().map(|home| home.join(rest)))
        {
            stripped
        } else {
            base.map(|base| base.join(file))
                .unwrap_or_else(|| file.clone())
        };
        return std::fs::read_to_string(&path)
            .with_context(|| format!("could not read promptFile {}", path.display()));
    }
    Ok(config
        .prompt
        .clone()
        .unwrap_or_else(|| DEFAULT_TEMPLATE.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conflicts::Operation;
    use crate::git::{ConflictKind, UnmergedEntry};

    fn state() -> ConflictState {
        ConflictState {
            operation: Operation::Merge,
            files: vec![
                UnmergedEntry {
                    path: "src/a.rs".into(),
                    kind: ConflictKind::BothModified,
                },
                UnmergedEntry {
                    path: "b.txt".into(),
                    kind: ConflictKind::DeletedByThem,
                },
            ],
            ours: "the current branch (main)".into(),
            theirs: "the branch being merged in (feature)".into(),
        }
    }

    fn context(state: &ConflictState) -> PromptContext<'_> {
        PromptContext {
            state,
            branch: Some("main"),
        }
    }

    #[test]
    fn the_default_template_uses_only_known_placeholders() {
        assert!(unknown_placeholders(DEFAULT_TEMPLATE).is_empty());
    }

    #[test]
    fn renders_every_placeholder() {
        let state = state();
        let out = render(
            "{{operation}}|{{ours}}|{{theirs}}|{{branch}}|\n{{files}}",
            &context(&state),
        )
        .unwrap();
        assert!(out.starts_with(
            "merge|the current branch (main)|the branch being merged in (feature)|main|"
        ));
        assert!(out.contains("- src/a.rs (both modified)\n- b.txt (deleted by them)"));
        assert!(!out.contains("{{"));
    }

    #[test]
    fn the_default_prompt_renders_and_forbids_git_writes() {
        let state = state();
        let out = render(DEFAULT_TEMPLATE, &context(&state)).unwrap();
        assert!(!out.contains("{{"));
        assert!(out.contains("Do NOT run git add, git commit, git push"));
        assert!(out.contains("```json"));
        assert!(out.contains("src/a.rs"));
    }

    #[test]
    fn whitespace_inside_braces_is_tolerated() {
        let state = state();
        assert_eq!(
            render("{{ operation }}", &context(&state)).unwrap(),
            "merge"
        );
    }

    #[test]
    fn an_unknown_placeholder_is_an_error_naming_it_and_the_known_ones() {
        let state = state();
        let error = render("fix {{fils}}", &context(&state))
            .unwrap_err()
            .to_string();
        assert!(error.contains("{{fils}}"));
        assert!(error.contains("{{files}}"));
    }

    #[test]
    fn a_dangling_opener_is_left_alone() {
        let state = state();
        assert_eq!(render("a {{ b", &context(&state)).unwrap(), "a {{ b");
    }

    #[test]
    fn template_prefers_prompt_file_then_prompt_then_default() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("p.md"), "from file {{files}}").unwrap();

        let mut config = Config::default();
        assert_eq!(template(&config, None).unwrap(), DEFAULT_TEMPLATE);

        config.prompt = Some("inline".into());
        assert_eq!(template(&config, None).unwrap(), "inline");

        config.prompt_file = Some("p.md".into());
        assert_eq!(
            template(&config, Some(dir.path())).unwrap(),
            "from file {{files}}"
        );
    }

    #[test]
    fn a_missing_prompt_file_is_an_error_naming_it() {
        let mut config = Config::default();
        config.prompt_file = Some("/definitely/not/here.md".into());
        let error = template(&config, None).unwrap_err();
        assert!(format!("{error:#}").contains("here.md"));
    }
}

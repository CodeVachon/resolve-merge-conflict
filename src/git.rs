//! Thin wrapper over the `git` executable via std::process::Command.
//!
//! Command execution sits behind [`CommandRunner`] so output parsing can be unit-tested with
//! canned stdout. Nothing here commits, pushes, or continues/aborts an operation: the tool's
//! promise is that it only reads state and stages files, and the surface of this module reflects
//! that.

use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Errors from running git.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GitError {
    /// git ran and exited non-zero.
    #[error("git {} failed{}: {}", render_args(args), status.map(|s| format!(" (exit {s})")).unwrap_or_default(), stderr.trim())]
    Command {
        args: Vec<String>,
        status: Option<i32>,
        stdout: String,
        stderr: String,
    },
    /// git could not be started at all.
    #[error("could not run git {}: {reason}", render_args(args))]
    Spawn { args: Vec<String>, reason: String },
    /// git printed something this module could not make sense of.
    #[error("unexpected git output: {0}")]
    Parse(String),
}

/// Raw result of one git invocation.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CommandOutput {
    pub status: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl CommandOutput {
    pub fn success(&self) -> bool {
        self.status == Some(0)
    }
}

/// Executes `git <args>` in a directory. Swap for a fake in tests.
pub trait CommandRunner {
    fn run(
        &mut self,
        cwd: &Path,
        args: &[String],
        envs: &[(String, String)],
    ) -> Result<CommandOutput, GitError>;
}

/// The real thing.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemRunner;

impl CommandRunner for SystemRunner {
    fn run(
        &mut self,
        cwd: &Path,
        args: &[String],
        envs: &[(String, String)],
    ) -> Result<CommandOutput, GitError> {
        let mut command = Command::new("git");
        command.args(args).current_dir(cwd);
        for (key, value) in envs {
            command.env(key, value);
        }
        let output = command.output().map_err(|error| GitError::Spawn {
            args: args.to_vec(),
            reason: error.to_string(),
        })?;
        Ok(CommandOutput {
            status: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}

/// `git a "b c"` — how a command is shown to the user.
pub fn render_args(args: &[String]) -> String {
    args.iter()
        .map(|arg| {
            if arg.chars().any(char::is_whitespace) || arg.contains('"') {
                format!("{arg:?}")
            } else {
                arg.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// How a path is unmerged, from the two-letter `XY` code of a porcelain v2 `u` line
/// (X is "ours" vs the merge base, Y is "theirs").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictKind {
    /// `UU`: both sides modified the file.
    BothModified,
    /// `AA`: both sides added the file.
    BothAdded,
    /// `DD`: both sides deleted the file.
    BothDeleted,
    /// `AU`: added by us.
    AddedByUs,
    /// `UA`: added by them.
    AddedByThem,
    /// `DU`: deleted by us, modified by them.
    DeletedByUs,
    /// `UD`: modified by us, deleted by them.
    DeletedByThem,
}

impl ConflictKind {
    pub fn from_code(code: &str) -> Option<Self> {
        Some(match code {
            "UU" => Self::BothModified,
            "AA" => Self::BothAdded,
            "DD" => Self::BothDeleted,
            "AU" => Self::AddedByUs,
            "UA" => Self::AddedByThem,
            "DU" => Self::DeletedByUs,
            "UD" => Self::DeletedByThem,
            _ => return None,
        })
    }

    /// Human wording, as `git status` phrases it.
    pub fn describe(self) -> &'static str {
        match self {
            Self::BothModified => "both modified",
            Self::BothAdded => "both added",
            Self::BothDeleted => "both deleted",
            Self::AddedByUs => "added by us",
            Self::AddedByThem => "added by them",
            Self::DeletedByUs => "deleted by us",
            Self::DeletedByThem => "deleted by them",
        }
    }
}

/// One unmerged path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnmergedEntry {
    pub path: String,
    pub kind: ConflictKind,
}

/// Which side of a conflict an index stage holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    /// Stage 1: the common ancestor.
    Base = 1,
    /// Stage 2: "ours", the side being merged into.
    Ours = 2,
    /// Stage 3: "theirs", the side being merged in.
    Theirs = 3,
}

/// A git working directory.
pub struct Git {
    cwd: PathBuf,
    runner: Box<dyn CommandRunner>,
    envs: Vec<(String, String)>,
}

impl fmt::Debug for Git {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Git")
            .field("cwd", &self.cwd)
            .field("envs", &self.envs)
            .finish()
    }
}

impl Git {
    /// Real git in `cwd`.
    pub fn new(cwd: impl Into<PathBuf>) -> Self {
        Self::with_runner(cwd, Box::new(SystemRunner))
    }

    /// Git in `cwd` with a custom [`CommandRunner`].
    pub fn with_runner(cwd: impl Into<PathBuf>, runner: Box<dyn CommandRunner>) -> Self {
        Self {
            cwd: cwd.into(),
            runner,
            envs: Vec::new(),
        }
    }

    /// Extra environment for every git call (tests use this to isolate global config).
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.envs.push((key.into(), value.into()));
        self
    }

    pub fn cwd(&self) -> &Path {
        &self.cwd
    }

    /// Run `git <args>` and return the raw output whatever the exit status.
    pub fn try_call<S: AsRef<str>>(&mut self, args: &[S]) -> Result<CommandOutput, GitError> {
        let args: Vec<String> = args.iter().map(|arg| arg.as_ref().to_string()).collect();
        self.runner.run(&self.cwd, &args, &self.envs)
    }

    /// Run `git <args>`; returns trimmed stdout, or [`GitError::Command`] on non-zero exit.
    pub fn call<S: AsRef<str>>(&mut self, args: &[S]) -> Result<String, GitError> {
        let output = self.try_call(args)?;
        if !output.success() {
            return Err(GitError::Command {
                args: args.iter().map(|arg| arg.as_ref().to_string()).collect(),
                status: output.status,
                stdout: output.stdout,
                stderr: output.stderr,
            });
        }
        Ok(output.stdout.trim_end().to_string())
    }

    /// True when `cwd` is inside a git work tree.
    pub fn is_work_tree(&mut self) -> bool {
        matches!(
            self.call(&["rev-parse", "--is-inside-work-tree"])
                .as_deref(),
            Ok("true")
        )
    }

    /// The repository root.
    pub fn top_level(&mut self) -> Result<PathBuf, GitError> {
        self.call(&["rev-parse", "--show-toplevel"])
            .map(PathBuf::from)
    }

    /// The absolute path of the git directory (where `MERGE_HEAD`, `rebase-merge/` etc. live).
    pub fn git_dir(&mut self) -> Result<PathBuf, GitError> {
        self.call(&["rev-parse", "--absolute-git-dir"])
            .map(PathBuf::from)
    }

    /// Commit hash of `HEAD`, or `None` in a repository with no commits yet.
    pub fn head(&mut self) -> Option<String> {
        self.call(&["rev-parse", "--verify", "HEAD"]).ok()
    }

    /// Current branch, or `None` when `HEAD` is detached (as during a rebase).
    pub fn current_branch(&mut self) -> Option<String> {
        self.call(&["symbolic-ref", "--short", "-q", "HEAD"]).ok()
    }

    /// Every unmerged path with how it conflicts, sorted by path. Read from
    /// `git status --porcelain=v2 -z`, whose `u` lines carry the `XY` code.
    pub fn unmerged_entries(&mut self) -> Result<Vec<UnmergedEntry>, GitError> {
        let output = self.call(&["status", "--porcelain=v2", "-z", "--untracked-files=no"])?;
        parse_unmerged(&output)
    }

    /// Paths with changes that are not conflicts: staged or unstaged edits, and untracked files.
    pub fn other_changes(&mut self) -> Result<Vec<String>, GitError> {
        let output = self.call(&["status", "--porcelain=v2", "-z", "--untracked-files=all"])?;
        Ok(parse_other_changes(&output))
    }

    /// The content of `path` at index `stage`, or `None` when that side has no such file (for
    /// example, "theirs" of a file they deleted).
    pub fn stage_blob(&mut self, stage: Stage, path: &str) -> Result<Option<String>, GitError> {
        let spec = format!(":{}:{path}", stage as u8);
        let output = self.try_call(&["show", &spec])?;
        Ok(output.success().then_some(output.stdout))
    }

    /// Stage `paths` (`git add -A -- <paths>`, so a resolved deletion is staged too).
    pub fn add<S: AsRef<str>>(&mut self, paths: &[S]) -> Result<(), GitError> {
        if paths.is_empty() {
            return Ok(());
        }
        let mut args = vec!["add".to_string(), "-A".to_string(), "--".to_string()];
        args.extend(paths.iter().map(|path| path.as_ref().to_string()));
        self.call(&args).map(|_| ())
    }

    /// `git diff --numstat` for `paths`, as `(added, removed, path)`; `staged` diffs the index
    /// against `HEAD`, otherwise the work tree against the index. Binary files report `0, 0`.
    pub fn numstat<S: AsRef<str>>(
        &mut self,
        staged: bool,
        paths: &[S],
    ) -> Result<Vec<(usize, usize, String)>, GitError> {
        let mut args = vec!["diff".to_string(), "--numstat".to_string()];
        if staged {
            args.push("--cached".to_string());
        }
        args.push("--".to_string());
        args.extend(paths.iter().map(|path| path.as_ref().to_string()));
        let output = self.call(&args)?;
        Ok(output
            .lines()
            .filter_map(|line| {
                let mut parts = line.splitn(3, '\t');
                let added = parts.next()?;
                let removed = parts.next()?;
                let path = parts.next()?;
                Some((
                    added.parse().unwrap_or(0),
                    removed.parse().unwrap_or(0),
                    path.to_string(),
                ))
            })
            .collect())
    }
}

/// Entries of `status --porcelain=v2 -z`: NUL-separated records. Rename/copy records (`2`) are
/// followed by one extra NUL-separated field, the original path, which is skipped here.
fn records(output: &str) -> Vec<&str> {
    let mut parts = output.split('\0').filter(|record| !record.is_empty());
    let mut records = Vec::new();
    while let Some(record) = parts.next() {
        if record.starts_with("2 ") {
            parts.next();
        }
        records.push(record);
    }
    records
}

fn parse_unmerged(output: &str) -> Result<Vec<UnmergedEntry>, GitError> {
    let mut entries = Vec::new();
    for record in records(output) {
        let Some(rest) = record.strip_prefix("u ") else {
            continue;
        };
        // u <XY> <sub> <m1> <m2> <m3> <mW> <h1> <h2> <h3> <path>
        let fields: Vec<&str> = rest.splitn(10, ' ').collect();
        let (Some(code), Some(path)) = (fields.first(), fields.get(9)) else {
            return Err(GitError::Parse(record.to_string()));
        };
        let kind = ConflictKind::from_code(code)
            .ok_or_else(|| GitError::Parse(format!("unknown conflict code in: {record}")))?;
        entries.push(UnmergedEntry {
            path: (*path).to_string(),
            kind,
        });
    }
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(entries)
}

fn parse_other_changes(output: &str) -> Vec<String> {
    let mut paths = Vec::new();
    for record in records(output) {
        if let Some(rest) = record.strip_prefix("1 ") {
            // 1 <XY> <sub> <mH> <mI> <mW> <hH> <hI> <path>
            if let Some(path) = rest.splitn(8, ' ').nth(7) {
                paths.push(path.to_string());
            }
        } else if let Some(rest) = record.strip_prefix("2 ") {
            // 2 <XY> <sub> <mH> <mI> <mW> <hH> <hI> <Xscore> <path>
            if let Some(path) = rest.splitn(9, ' ').nth(8) {
                paths.push(path.to_string());
            }
        } else if let Some(path) = record.strip_prefix("? ") {
            paths.push(path.to_string());
        }
    }
    paths.sort();
    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    const UU: &str = "u UU N... 100644 100644 100644 100644 aaa bbb ccc src/a file.txt";

    #[test]
    fn parses_unmerged_records_including_paths_with_spaces() {
        let output = format!("{UU}\0u DU N... 100644 000000 100644 100644 aaa 000 ccc b.txt\0");
        let entries = parse_unmerged(&output).unwrap();
        assert_eq!(
            entries,
            [
                UnmergedEntry {
                    path: "b.txt".into(),
                    kind: ConflictKind::DeletedByUs
                },
                UnmergedEntry {
                    path: "src/a file.txt".into(),
                    kind: ConflictKind::BothModified
                },
            ]
        );
    }

    #[test]
    fn an_unknown_conflict_code_is_an_error_not_a_silent_skip() {
        let output = "u ZZ N... 100644 100644 100644 100644 a b c x.txt\0";
        assert!(matches!(parse_unmerged(output), Err(GitError::Parse(_))));
    }

    #[test]
    fn other_changes_skip_conflicts_and_handle_renames() {
        let output = format!(
            "{UU}\01 .M N... 100644 100644 100644 aaa bbb docs/readme.md\0\
             2 R. N... 100644 100644 100644 aaa bbb R100 new.txt\0old.txt\0? scratch.txt\0"
        );
        assert_eq!(
            parse_other_changes(&output),
            ["docs/readme.md", "new.txt", "scratch.txt"]
        );
    }

    #[test]
    fn render_args_quotes_only_what_needs_it() {
        let args = ["add".to_string(), "my file".to_string()];
        assert_eq!(render_args(&args), "add \"my file\"");
    }

    #[test]
    fn conflict_kind_round_trips_every_code() {
        for code in ["UU", "AA", "DD", "AU", "UA", "DU", "UD"] {
            assert!(ConflictKind::from_code(code).is_some(), "{code}");
        }
        assert!(ConflictKind::from_code("MM").is_none());
    }
}

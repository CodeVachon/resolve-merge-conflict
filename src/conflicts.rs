//! Finding out whether a repository is in the middle of a conflicted operation, and which.
//!
//! This only reads state. It never starts, continues or aborts anything.

use anyhow::{Result, bail};

use crate::git::{Git, UnmergedEntry};

/// The git operation that stopped on conflicts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Merge,
    Rebase,
    CherryPick,
    Revert,
    /// Unmerged paths with no operation marker: `git stash pop`/`apply`, `checkout -m`, and the
    /// like leave conflicts without recording what caused them.
    Other,
}

impl Operation {
    pub fn describe(self) -> &'static str {
        match self {
            Self::Merge => "merge",
            Self::Rebase => "rebase",
            Self::CherryPick => "cherry-pick",
            Self::Revert => "revert",
            Self::Other => "stash pop or similar",
        }
    }
}

/// A repository stopped on conflicts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConflictState {
    pub operation: Operation,
    /// The conflicted files, sorted by path.
    pub files: Vec<UnmergedEntry>,
    /// Plain-language name of "our" side, for the prompt.
    pub ours: String,
    /// Plain-language name of "their" side, for the prompt.
    pub theirs: String,
}

/// The conflicted state of the repository `git` points into, or `None` when nothing is conflicted.
///
/// A repository mid-merge whose conflicts are all already resolved and staged is reported as
/// `None`: there is nothing left for an agent to do, and finishing the merge is the user's step.
pub fn detect(git: &mut Git) -> Result<Option<ConflictState>> {
    if !git.is_work_tree() {
        bail!("{} is not inside a git repository", git.cwd().display());
    }
    let files = git.unmerged_entries()?;
    if files.is_empty() {
        return Ok(None);
    }
    let git_dir = git.git_dir()?;
    let operation = if git_dir.join("MERGE_HEAD").exists() {
        Operation::Merge
    } else if git_dir.join("rebase-merge").exists() || git_dir.join("rebase-apply").exists() {
        Operation::Rebase
    } else if git_dir.join("CHERRY_PICK_HEAD").exists() {
        Operation::CherryPick
    } else if git_dir.join("REVERT_HEAD").exists() {
        Operation::Revert
    } else {
        Operation::Other
    };

    let branch = git.current_branch();
    let (ours, theirs) = match operation {
        Operation::Merge => (
            match &branch {
                Some(branch) => format!("the current branch ({branch})"),
                None => "the current commit".to_string(),
            },
            match merge_source(&git_dir) {
                Some(source) => format!("the branch being merged in ({source})"),
                None => "the branch being merged in".to_string(),
            },
        ),
        // During a rebase "ours" is the branch being rebased onto and "theirs" is the commit being
        // replayed: the reverse of what the words suggest.
        Operation::Rebase => (
            "the branch being rebased onto".to_string(),
            "your commit being replayed".to_string(),
        ),
        Operation::CherryPick => (
            "the current branch".to_string(),
            "the commit being cherry-picked".to_string(),
        ),
        Operation::Revert => (
            "the current branch".to_string(),
            "the revert of the commit".to_string(),
        ),
        Operation::Other => (
            "the current branch".to_string(),
            "the incoming changes (for example the stash)".to_string(),
        ),
    };

    Ok(Some(ConflictState {
        operation,
        files,
        ours,
        theirs,
    }))
}

/// The branch name being merged in, from `MERGE_MSG` ("Merge branch 'x'"), if it says.
fn merge_source(git_dir: &std::path::Path) -> Option<String> {
    let message = std::fs::read_to_string(git_dir.join("MERGE_MSG")).ok()?;
    let first = message.lines().next()?;
    let start = first.find('\'')? + 1;
    let end = first[start..].find('\'')? + start;
    Some(first[start..end].to_string())
}

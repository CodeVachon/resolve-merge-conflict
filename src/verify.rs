//! After the agent has run: check its work, then stage what is really resolved.
//!
//! The agent is trusted to edit files and nothing more. This module is where that is enforced and
//! where staging happens: a file is staged only if it no longer carries conflict markers, and
//! nothing is ever committed, pushed, continued or aborted.

use std::collections::BTreeMap;
use std::hash::{DefaultHasher, Hash, Hasher};

use anyhow::Result;

use crate::conflicts::ConflictState;
use crate::git::{ConflictKind, Git, Stage};

/// How the final content relates to the two sides, for the report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    /// Identical to "ours".
    Ours,
    /// Identical to "theirs".
    Theirs,
    /// Something else: a real merge of the two, or new content.
    Merged,
    /// The file was removed.
    Deleted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileStatus {
    /// Free of conflict markers (or deleted where a deletion is a valid resolution).
    Resolved,
    /// Conflict markers are still in the file.
    MarkersRemain,
    /// The file is gone but this kind of conflict does not call for deleting it.
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileResult {
    pub path: String,
    pub kind: ConflictKind,
    pub status: FileStatus,
    pub strategy: Option<Strategy>,
    pub staged: bool,
    pub added: usize,
    pub removed: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verification {
    pub files: Vec<FileResult>,
    /// `HEAD` is not where it was before the agent ran: something committed or moved it.
    pub head_moved: bool,
    /// Files outside the conflicted set that changed (or appeared) while the agent ran.
    pub touched_outside: Vec<String>,
}

impl Verification {
    pub fn resolved(&self) -> impl Iterator<Item = &FileResult> {
        self.files
            .iter()
            .filter(|file| file.status == FileStatus::Resolved)
    }

    pub fn unresolved(&self) -> impl Iterator<Item = &FileResult> {
        self.files
            .iter()
            .filter(|file| file.status != FileStatus::Resolved)
    }

    pub fn all_resolved(&self) -> bool {
        self.unresolved().next().is_none()
    }
}

/// What the repository looked like before the agent ran.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    head: Option<String>,
    /// Non-conflicted changed files, by a hash of their content (`None`: file absent).
    others: BTreeMap<String, Option<u64>>,
}

impl Snapshot {
    pub fn take(git: &mut Git) -> Result<Self> {
        let others = git
            .other_changes()?
            .into_iter()
            .map(|path| {
                let hash = content_hash(git, &path);
                (path, hash)
            })
            .collect();
        Ok(Self {
            head: git.head(),
            others,
        })
    }
}

fn content_hash(git: &Git, path: &str) -> Option<u64> {
    let bytes = std::fs::read(git.cwd().join(path)).ok()?;
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    Some(hasher.finish())
}

/// True when `text` still contains a conflict block: a `<<<<<<<` line, then a `=======` line, then
/// a `>>>>>>>` line, in that order. Requiring all three keeps a stray `=======` underline in a
/// document from counting.
pub fn has_conflict_markers(text: &str) -> bool {
    let mut stage = 0;
    for line in text.lines() {
        let line = line.trim_end();
        match stage {
            0 if line == "<<<<<<<" || line.starts_with("<<<<<<< ") => stage = 1,
            1 if line == "=======" => stage = 2,
            2 if line == ">>>>>>>" || line.starts_with(">>>>>>> ") => return true,
            _ => {}
        }
    }
    false
}

fn deletion_is_valid(kind: ConflictKind) -> bool {
    matches!(
        kind,
        ConflictKind::DeletedByUs | ConflictKind::DeletedByThem | ConflictKind::BothDeleted
    )
}

/// Check every originally conflicted file and, when `stage` is set, `git add` the resolved ones.
pub fn verify_and_stage(
    git: &mut Git,
    state: &ConflictState,
    before: &Snapshot,
    stage: bool,
) -> Result<Verification> {
    let mut files = Vec::new();
    let mut to_stage = Vec::new();

    for entry in &state.files {
        let absolute = git.cwd().join(&entry.path);
        let content = std::fs::read(&absolute).ok();
        let (status, strategy) = match &content {
            None if deletion_is_valid(entry.kind) => {
                (FileStatus::Resolved, Some(Strategy::Deleted))
            }
            None => (FileStatus::Missing, None),
            Some(bytes) => {
                let text = String::from_utf8_lossy(bytes);
                if has_conflict_markers(&text) {
                    (FileStatus::MarkersRemain, None)
                } else {
                    (
                        FileStatus::Resolved,
                        Some(strategy_of(git, &entry.path, bytes)?),
                    )
                }
            }
        };
        if status == FileStatus::Resolved {
            to_stage.push(entry.path.clone());
        }
        files.push(FileResult {
            path: entry.path.clone(),
            kind: entry.kind,
            status,
            strategy,
            staged: false,
            added: 0,
            removed: 0,
        });
    }

    if stage && !to_stage.is_empty() {
        git.add(&to_stage)?;
        let stats = git.numstat(true, &to_stage)?;
        for file in files
            .iter_mut()
            .filter(|f| f.status == FileStatus::Resolved)
        {
            file.staged = true;
            if let Some((added, removed, _)) = stats.iter().find(|(_, _, path)| *path == file.path)
            {
                file.added = *added;
                file.removed = *removed;
            }
        }
    }

    let conflicted: Vec<&str> = state.files.iter().map(|f| f.path.as_str()).collect();
    let after = git.other_changes()?;
    let mut touched_outside = Vec::new();
    for path in after {
        if conflicted.contains(&path.as_str()) {
            continue;
        }
        let now = content_hash(git, &path);
        if before.others.get(&path) != Some(&now) {
            touched_outside.push(path);
        }
    }
    // A file that had pending changes and was reverted by the agent drops out of `other_changes`
    // entirely; catch that too.
    for (path, was) in &before.others {
        if !conflicted.contains(&path.as_str())
            && !touched_outside.contains(path)
            && content_hash(git, path) != *was
        {
            touched_outside.push(path.clone());
        }
    }
    touched_outside.sort();

    Ok(Verification {
        files,
        head_moved: git.head() != before.head,
        touched_outside,
    })
}

/// Compare final `content` with the two sides still held in the index.
fn strategy_of(git: &mut Git, path: &str, content: &[u8]) -> Result<Strategy> {
    let ours = git.stage_blob(Stage::Ours, path)?;
    let theirs = git.stage_blob(Stage::Theirs, path)?;
    let text = String::from_utf8_lossy(content);
    Ok(if ours.as_deref() == Some(&*text) {
        Strategy::Ours
    } else if theirs.as_deref() == Some(&*text) {
        Strategy::Theirs
    } else {
        Strategy::Merged
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_full_conflict_block_is_detected() {
        assert!(has_conflict_markers(
            "a\n<<<<<<< HEAD\nx\n=======\ny\n>>>>>>> feature\nb\n"
        ));
        assert!(has_conflict_markers("<<<<<<<\nx\n=======\ny\n>>>>>>>\n"));
    }

    #[test]
    fn diff3_style_blocks_are_detected() {
        assert!(has_conflict_markers(
            "<<<<<<< HEAD\nx\n||||||| base\nb\n=======\ny\n>>>>>>> f\n"
        ));
    }

    #[test]
    fn lone_separators_and_incomplete_blocks_are_not_conflicts() {
        assert!(!has_conflict_markers("Title\n=======\ntext\n"));
        assert!(!has_conflict_markers("<<<<<<< HEAD\nonly an opener\n"));
        assert!(!has_conflict_markers(">>>>>>> x\n=======\n<<<<<<< y\n"));
        assert!(!has_conflict_markers("a <<<<<<< inline\n"));
    }
}

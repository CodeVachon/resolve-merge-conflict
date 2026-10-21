//! Hermetic git fixtures for integration tests. Nothing here touches the network or needs a real
//! agent: each scenario builds a throwaway repository left in a genuine conflicted state.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use resolve_merge_conflicts::git::Git;
use tempfile::TempDir;

/// Env that isolates git from the user's global/system config, hooks and signing.
pub fn isolation_env(hooks_dir: &Path) -> Vec<(String, String)> {
    vec![
        ("GIT_CONFIG_GLOBAL".into(), "/dev/null".into()),
        ("GIT_CONFIG_NOSYSTEM".into(), "1".into()),
        ("GIT_AUTHOR_NAME".into(), "Fixture".into()),
        ("GIT_AUTHOR_EMAIL".into(), "fixture@example.com".into()),
        ("GIT_COMMITTER_NAME".into(), "Fixture".into()),
        ("GIT_COMMITTER_EMAIL".into(), "fixture@example.com".into()),
        ("GIT_TERMINAL_PROMPT".into(), "0".into()),
        ("GIT_EDITOR".into(), "true".into()),
        ("GIT_CONFIG_COUNT".into(), "3".into()),
        ("GIT_CONFIG_KEY_0".into(), "commit.gpgsign".into()),
        ("GIT_CONFIG_VALUE_0".into(), "false".into()),
        ("GIT_CONFIG_KEY_1".into(), "core.hooksPath".into()),
        (
            "GIT_CONFIG_VALUE_1".into(),
            hooks_dir.to_string_lossy().into_owned(),
        ),
        ("GIT_CONFIG_KEY_2".into(), "init.defaultBranch".into()),
        ("GIT_CONFIG_VALUE_2".into(), "main".into()),
    ]
}

/// The operation a conflicted fixture was left in the middle of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Merge,
    Rebase,
    CherryPick,
    Revert,
    StashPop,
}

/// A single work repository on `main` with one file, `a.txt`, three lines long.
pub struct Fixture {
    pub temp: TempDir,
    pub work: PathBuf,
    pub hooks: PathBuf,
}

/// The conflicted file every scenario produces (besides the delete/modify one).
pub const CONFLICTED: &str = "a.txt";

impl Fixture {
    /// A clean repository with one commit; no conflict yet.
    pub fn new() -> Self {
        let temp = tempfile::tempdir().expect("temp dir");
        let work = temp.path().join("work");
        let hooks = temp.path().join("no-hooks");
        std::fs::create_dir_all(&work).unwrap();
        std::fs::create_dir_all(&hooks).unwrap();
        let fixture = Self { temp, work, hooks };
        fixture.raw(&["init", "--initial-branch=main"]);
        fixture.write(CONFLICTED, "one\nbase\nthree\n");
        fixture.write("b.txt", "bee\n");
        fixture.commit_all("initial");
        fixture
    }

    /// A repository left mid-`operation` with `a.txt` conflicted: `main` says `main`, the other
    /// side says `feature`.
    pub fn conflicted(operation: Operation) -> Self {
        let fixture = Self::new();
        match operation {
            Operation::Merge => {
                fixture.diverge();
                fixture.expect_conflict(&["merge", "feature"]);
            }
            Operation::Rebase => {
                fixture.diverge();
                fixture.raw(&["checkout", "feature"]);
                fixture.expect_conflict(&["rebase", "main"]);
            }
            Operation::CherryPick => {
                fixture.diverge();
                fixture.expect_conflict(&["cherry-pick", "feature"]);
            }
            Operation::Revert => {
                fixture.write(CONFLICTED, "one\nfirst\nthree\n");
                fixture.commit_all("first change");
                let first = fixture.raw(&["rev-parse", "HEAD"]);
                fixture.write(CONFLICTED, "one\nsecond\nthree\n");
                fixture.commit_all("second change");
                fixture.expect_conflict(&["revert", &first]);
            }
            Operation::StashPop => {
                fixture.write(CONFLICTED, "one\nfeature\nthree\n");
                fixture.raw(&["stash"]);
                fixture.write(CONFLICTED, "one\nmain\nthree\n");
                fixture.commit_all("main change");
                fixture.expect_conflict(&["stash", "pop"]);
            }
        }
        fixture
    }

    /// A merge where `main` modified `b.txt` and `feature` deleted it ("deleted by them").
    pub fn delete_modify_conflict() -> Self {
        let fixture = Self::new();
        fixture.raw(&["checkout", "-b", "feature"]);
        fixture.raw(&["rm", "b.txt"]);
        fixture.raw(&["commit", "-m", "feature deletes b"]);
        fixture.raw(&["checkout", "main"]);
        fixture.write("b.txt", "bee, modified\n");
        fixture.commit_all("main modifies b");
        fixture.expect_conflict(&["merge", "feature"]);
        fixture
    }

    /// Give `main` and a new `feature` branch conflicting edits to `a.txt`, ending on `main`.
    fn diverge(&self) {
        self.raw(&["checkout", "-b", "feature"]);
        self.write(CONFLICTED, "one\nfeature\nthree\n");
        self.commit_all("feature change");
        self.raw(&["checkout", "main"]);
        self.write(CONFLICTED, "one\nmain\nthree\n");
        self.commit_all("main change");
    }

    /// A [`Git`] on the work repo with the same isolation applied.
    pub fn git(&self) -> Git {
        let mut git = Git::new(&self.work);
        for (key, value) in isolation_env(&self.hooks) {
            git = git.env(key, value);
        }
        git
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new("git");
        command.args(args).current_dir(&self.work);
        for (key, value) in isolation_env(&self.hooks) {
            command.env(key, value);
        }
        command
    }

    /// Run git, panicking on failure; returns trimmed stdout.
    pub fn raw(&self, args: &[&str]) -> String {
        let output = self.command(args).output().expect("git runs");
        assert!(
            output.status.success(),
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string()
    }

    /// Run git and return the whole output whatever the exit status.
    pub fn try_raw(&self, args: &[&str]) -> Output {
        self.command(args).output().expect("git runs")
    }

    /// Run git and require that it stops on conflicts.
    fn expect_conflict(&self, args: &[&str]) {
        let output = self.try_raw(args);
        assert!(
            !output.status.success(),
            "git {} unexpectedly succeeded; no conflict was produced",
            args.join(" ")
        );
        assert!(
            !self.unmerged().is_empty(),
            "git {} failed without leaving unmerged paths: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    /// Paths git currently reports as unmerged, sorted.
    pub fn unmerged(&self) -> Vec<String> {
        let mut paths: Vec<String> = self
            .raw(&["diff", "--name-only", "--diff-filter=U"])
            .lines()
            .map(str::to_owned)
            .collect();
        paths.sort();
        paths
    }

    /// Paths staged in the index, sorted.
    pub fn staged(&self) -> Vec<String> {
        let mut paths: Vec<String> = self
            .raw(&["diff", "--cached", "--name-only"])
            .lines()
            .map(str::to_owned)
            .collect();
        paths.sort();
        paths
    }

    pub fn head(&self) -> String {
        self.raw(&["rev-parse", "HEAD"])
    }

    pub fn write(&self, relative: &str, contents: &str) {
        let path = self.work.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, contents).unwrap();
    }

    pub fn read(&self, relative: &str) -> String {
        std::fs::read_to_string(self.work.join(relative)).unwrap()
    }

    pub fn commit_all(&self, message: &str) {
        self.raw(&["add", "-A"]);
        self.raw(&["commit", "-m", message]);
    }
}

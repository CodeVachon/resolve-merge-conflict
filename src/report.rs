//! What happened, as data: the facts this tool verified side by side with the agent's own account.
//!
//! The two are kept separate on purpose. `actual` and `status` are checked by this tool against
//! the working tree; `claimed` and `explanation` are only what the agent said. Rendering and the
//! comparison between them live in `render`.

use serde::Serialize;

use crate::conflicts::Operation;
use crate::git::ConflictKind;
use crate::runner::Finished;
use crate::summary::{self, AgentSummary};
use crate::verify::{FileStatus, Strategy};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FileReport {
    pub path: String,
    pub kind: ConflictKind,
    /// Checked by this tool.
    pub status: FileStatus,
    /// Checked by this tool: how the final content relates to the two sides.
    pub actual: Option<Strategy>,
    pub staged: bool,
    pub added: usize,
    pub removed: usize,
    /// What the agent said it did, if it said anything usable about this file.
    pub claimed: Option<Strategy>,
    /// The agent's one-line reason.
    pub explanation: String,
    /// The agent mentioned this file in its summary at all.
    pub mentioned: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Report {
    pub agent: String,
    pub model: String,
    pub operation: Operation,
    /// The agent process exited cleanly and did not report an error.
    pub agent_succeeded: bool,
    pub exit_code: Option<i32>,
    /// Staging was requested for this run.
    pub staging_enabled: bool,
    pub files: Vec<FileReport>,
    /// `HEAD` moved while the agent ran: it committed, or otherwise changed history.
    pub head_moved: bool,
    /// Files outside the conflicted set that changed while the agent ran.
    pub touched_outside: Vec<String>,
    /// The agent gave a usable structured summary.
    pub has_agent_summary: bool,
    /// Summary entries for paths that were never in conflict.
    pub unexpected_summary_paths: Vec<String>,
    /// The agent's final message, shown when there is no structured summary.
    pub agent_message: String,
    /// The tail of the agent's stderr, kept only when the run failed.
    pub agent_stderr: String,
}

impl Report {
    pub fn build(finished: &Finished) -> Self {
        let summary = summary::parse(&finished.outcome.final_message);
        Self::build_with(finished, summary)
    }

    pub fn build_with(finished: &Finished, summary: Option<AgentSummary>) -> Self {
        let files = finished
            .verification
            .files
            .iter()
            .map(|file| {
                let entry = summary.as_ref().and_then(|s| s.entry(&file.path));
                FileReport {
                    path: file.path.clone(),
                    kind: file.kind,
                    status: file.status,
                    actual: file.strategy,
                    staged: file.staged,
                    added: file.added,
                    removed: file.removed,
                    claimed: entry.and_then(|e| e.strategy),
                    explanation: entry.map(|e| e.summary.clone()).unwrap_or_default(),
                    mentioned: entry.is_some(),
                }
            })
            .collect();

        let unexpected_summary_paths = summary
            .as_ref()
            .map(|s| {
                s.files
                    .iter()
                    .filter(|entry| {
                        !finished
                            .verification
                            .files
                            .iter()
                            .any(|file| file.path == entry.path)
                    })
                    .map(|entry| entry.path.clone())
                    .collect()
            })
            .unwrap_or_default();

        Self {
            agent: finished.plan.agent.clone(),
            model: finished.plan.model.clone(),
            operation: finished.plan.state.operation,
            agent_succeeded: finished.outcome.success,
            exit_code: finished.outcome.exit_code,
            staging_enabled: finished.staged,
            files,
            head_moved: finished.verification.head_moved,
            touched_outside: finished.verification.touched_outside.clone(),
            has_agent_summary: summary.is_some(),
            unexpected_summary_paths,
            agent_message: finished.outcome.final_message.clone(),
            agent_stderr: if finished.outcome.success {
                String::new()
            } else {
                tail(&finished.outcome.stderr, 20)
            },
        }
    }

    pub fn resolved_count(&self) -> usize {
        self.files
            .iter()
            .filter(|file| file.status == FileStatus::Resolved)
            .count()
    }

    pub fn all_resolved(&self) -> bool {
        self.resolved_count() == self.files.len()
    }
}

/// A place where the agent's account and the verified facts disagree, or where something happened
/// that the user should look at before trusting the result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Mismatch {
    /// The agent described this file as handled, but it is still conflicted.
    ClaimedDoneButUnresolved { path: String },
    /// The agent said it took one approach; the content says another.
    StrategyDiffers {
        path: String,
        claimed: Strategy,
        actual: Strategy,
    },
    /// The agent gave a summary but never mentioned this file.
    NotMentioned { path: String },
    /// The agent's summary covers a file that was never in conflict.
    UnexpectedPath { path: String },
    /// The agent reported failure although every file is resolved.
    ReportedFailureButResolved,
    /// `HEAD` moved: the agent committed or changed history, which it was told not to.
    HeadMoved,
    /// Files outside the conflict changed.
    TouchedOutside { paths: Vec<String> },
}

impl Report {
    /// Everything worth a second look, most serious first.
    pub fn mismatches(&self) -> Vec<Mismatch> {
        let mut found = Vec::new();
        if self.head_moved {
            found.push(Mismatch::HeadMoved);
        }
        if !self.touched_outside.is_empty() {
            found.push(Mismatch::TouchedOutside {
                paths: self.touched_outside.clone(),
            });
        }
        for file in &self.files {
            if file.mentioned && file.status != FileStatus::Resolved {
                found.push(Mismatch::ClaimedDoneButUnresolved {
                    path: file.path.clone(),
                });
            }
            if let (Some(claimed), Some(actual)) = (file.claimed, file.actual)
                && claimed != actual
            {
                found.push(Mismatch::StrategyDiffers {
                    path: file.path.clone(),
                    claimed,
                    actual,
                });
            }
            if self.has_agent_summary && !file.mentioned {
                found.push(Mismatch::NotMentioned {
                    path: file.path.clone(),
                });
            }
        }
        for path in &self.unexpected_summary_paths {
            found.push(Mismatch::UnexpectedPath { path: path.clone() });
        }
        if !self.agent_succeeded && self.all_resolved() && !self.files.is_empty() {
            found.push(Mismatch::ReportedFailureButResolved);
        }
        found
    }
}

/// The last `lines` lines of `text`.
fn tail(text: &str, lines: usize) -> String {
    let all: Vec<&str> = text.lines().collect();
    all[all.len().saturating_sub(lines)..].join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tail_keeps_the_last_lines() {
        assert_eq!(tail("a\nb\nc\nd", 2), "c\nd");
        assert_eq!(tail("a", 5), "a");
        assert_eq!(tail("", 5), "");
    }
}

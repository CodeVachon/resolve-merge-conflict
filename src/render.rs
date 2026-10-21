//! Turning a [`Report`] into text for a person, or JSON for a script.

use serde::Serialize;

use crate::conflicts::Operation;
use crate::report::{Mismatch, Report};
use crate::ui;
use crate::verify::{FileStatus, Strategy};

fn strategy_word(strategy: Strategy) -> &'static str {
    match strategy {
        Strategy::Ours => "kept ours",
        Strategy::Theirs => "took theirs",
        Strategy::Merged => "merged both",
        Strategy::Deleted => "deleted",
    }
}

fn describe(mismatch: &Mismatch) -> String {
    match mismatch {
        Mismatch::ClaimedDoneButUnresolved { path } => format!(
            "{path}: the agent described it as handled, but conflict markers (or a missing file) remain"
        ),
        Mismatch::StrategyDiffers {
            path,
            claimed,
            actual,
        } => format!(
            "{path}: the agent says it {}, but the content looks like it {}",
            strategy_word(*claimed),
            strategy_word(*actual)
        ),
        Mismatch::NotMentioned { path } => {
            format!("{path}: missing from the agent's summary; review it yourself")
        }
        Mismatch::UnexpectedPath { path } => {
            format!("{path}: in the agent's summary but was never in conflict")
        }
        Mismatch::ReportedFailureButResolved => {
            "the agent reported a failure, yet every file is resolved; check its output".to_string()
        }
        Mismatch::HeadMoved => {
            "HEAD moved while the agent ran: it committed or changed history, which it was told not to. Inspect `git log` and `git reflog`".to_string()
        }
        Mismatch::TouchedOutside { paths } => format!(
            "the agent changed files outside the conflict: {}",
            paths.join(", ")
        ),
    }
}

fn continue_command(operation: Operation) -> Option<&'static str> {
    match operation {
        Operation::Merge => Some("git merge --continue"),
        Operation::Rebase => Some("git rebase --continue"),
        Operation::CherryPick => Some("git cherry-pick --continue"),
        Operation::Revert => Some("git revert --continue"),
        Operation::Other => None,
    }
}

/// The report as it appears in the terminal.
pub fn text(report: &Report) -> String {
    let mut out = String::new();
    let total = report.files.len();
    let resolved = report.resolved_count();

    out.push_str(&ui::bold("Resolution report"));
    out.push('\n');
    out.push_str(&ui::dim(&format!(
        "{} ({}) · {} · {} of {} file{} resolved\n\n",
        report.agent,
        report.model,
        report.operation.describe(),
        resolved,
        total,
        if total == 1 { "" } else { "s" }
    )));

    let width = report.files.iter().map(|f| f.path.len()).max().unwrap_or(0);
    for file in &report.files {
        let (mark, state) = match file.status {
            FileStatus::Resolved => (ui::green("✓"), String::new()),
            FileStatus::MarkersRemain => (ui::red("✗"), ui::red("conflict markers remain")),
            FileStatus::Missing => (ui::red("✗"), ui::red("file is missing")),
        };
        let mut line = format!(
            "  {mark} {:<width$}  {}",
            file.path,
            ui::dim(file.kind.describe())
        );
        if file.status == FileStatus::Resolved {
            if let Some(actual) = file.actual {
                line.push_str(&format!("  {}", strategy_word(actual)));
            }
            if file.staged {
                line.push_str(&format!(
                    "  +{} −{}  {}",
                    file.added,
                    file.removed,
                    ui::dim("staged")
                ));
            }
        } else {
            line.push_str(&format!("  {state}"));
        }
        out.push_str(&line);
        out.push('\n');
        if !file.explanation.is_empty() {
            out.push_str(&format!("      {}\n", ui::dim(&file.explanation)));
        }
    }

    if !report.has_agent_summary && !report.agent_message.trim().is_empty() {
        out.push('\n');
        out.push_str(&ui::yellow(
            "The agent did not give a structured summary; its own words:",
        ));
        out.push('\n');
        for line in report.agent_message.trim().lines() {
            out.push_str(&format!("  {line}\n"));
        }
    }

    let mismatches = report.mismatches();
    if !mismatches.is_empty() {
        out.push('\n');
        out.push_str(&ui::yellow("Look closer at:"));
        out.push('\n');
        for mismatch in &mismatches {
            out.push_str(&format!("  ! {}\n", describe(mismatch)));
        }
    }

    if !report.agent_succeeded {
        out.push('\n');
        out.push_str(&ui::red(&format!(
            "The agent did not finish cleanly (exit {}).",
            report
                .exit_code
                .map(|code| code.to_string())
                .unwrap_or_else(|| "none".into())
        )));
        out.push('\n');
        for line in report.agent_stderr.lines() {
            out.push_str(&format!("  {line}\n"));
        }
    }

    out.push('\n');
    out.push_str(&ui::bold("Next"));
    out.push('\n');
    let unresolved: Vec<&str> = report
        .files
        .iter()
        .filter(|file| file.status != FileStatus::Resolved)
        .map(|file| file.path.as_str())
        .collect();
    if !unresolved.is_empty() {
        out.push_str(&format!(
            "  Still conflicted: {}. Fix them by hand or run this again.\n",
            unresolved.join(", ")
        ));
    }
    if report.staging_enabled {
        out.push_str("  Review what was resolved:  git diff --staged\n");
    } else {
        out.push_str("  Review the changes, then stage them:  git diff && git add <files>\n");
    }
    if unresolved.is_empty()
        && let Some(command) = continue_command(report.operation)
    {
        out.push_str(&format!("  When you are happy:        {command}\n"));
    }
    out.push_str(&ui::dim(
        "Nothing was committed or pushed; that is left to you.\n",
    ));
    out
}

#[derive(Serialize)]
struct JsonReport<'a> {
    #[serde(flatten)]
    report: &'a Report,
    mismatches: Vec<Mismatch>,
}

/// The report as pretty-printed JSON, with the mismatches included.
pub fn json(report: &Report) -> String {
    serde_json::to_string_pretty(&JsonReport {
        report,
        mismatches: report.mismatches(),
    })
    .expect("a report always serialises")
}

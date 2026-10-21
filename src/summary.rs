//! Reading the agent's own account of what it did.
//!
//! The prompt asks the agent to end with a fenced JSON block (see `prompt::SUMMARY_FORMAT`). Agents
//! do not always comply, so this is deliberately forgiving about where the JSON is and strict about
//! what counts as a usable answer. Failing to find one is normal, not an error: the report then
//! shows the agent's raw message instead.

use serde_json::Value;

use crate::verify::Strategy;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub path: String,
    /// What the agent says it did; `None` if it said something we do not recognise.
    pub strategy: Option<Strategy>,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AgentSummary {
    pub files: Vec<Entry>,
}

impl AgentSummary {
    pub fn entry(&self, path: &str) -> Option<&Entry> {
        self.files.iter().find(|entry| entry.path == path)
    }
}

/// The structured summary in `message`, if there is one.
pub fn parse(message: &str) -> Option<AgentSummary> {
    candidates(message)
        .into_iter()
        .rev()
        .find_map(|candidate| from_json(&candidate))
}

/// Every plausible JSON text in `message`, in the order they appear: fenced blocks, then the whole
/// message, then balanced `{...}` objects.
fn candidates(message: &str) -> Vec<String> {
    let mut found = Vec::new();

    let mut rest = message;
    while let Some(start) = rest.find("```") {
        let after = &rest[start + 3..];
        let Some(end) = after.find("```") else { break };
        let block = &after[..end];
        // Drop the language tag line ("json") if there is one.
        let body = match block.split_once('\n') {
            Some((first, body)) if !first.trim().starts_with('{') => body,
            _ => block,
        };
        found.push(body.trim().to_string());
        rest = &after[end + 3..];
    }

    found.push(message.trim().to_string());

    let bytes = message.as_bytes();
    for (index, _) in message.match_indices('{') {
        if let Some(end) = balanced_end(bytes, index) {
            found.push(message[index..=end].to_string());
        }
    }
    found
}

/// Index of the `}` closing the `{` at `start`, honouring strings and escapes.
fn balanced_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (offset, &byte) in bytes[start..].iter().enumerate() {
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            continue;
        }
        match byte {
            b'"' => in_string = true,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(start + offset);
                }
            }
            _ => {}
        }
    }
    None
}

fn from_json(text: &str) -> Option<AgentSummary> {
    let value: Value = serde_json::from_str(text).ok()?;
    let files = value.get("files")?.as_array()?;
    let entries: Vec<Entry> = files
        .iter()
        .filter_map(|file| {
            let path = file.get("path")?.as_str()?.trim();
            if path.is_empty() {
                return None;
            }
            let summary = ["summary", "reason", "explanation"]
                .iter()
                .find_map(|key| file.get(*key).and_then(Value::as_str))
                .unwrap_or_default()
                .trim()
                .to_string();
            Some(Entry {
                path: path.to_string(),
                strategy: file
                    .get("strategy")
                    .and_then(Value::as_str)
                    .and_then(parse_strategy),
                summary,
            })
        })
        .collect();
    if entries.is_empty() && !files.is_empty() {
        return None;
    }
    Some(AgentSummary { files: entries })
}

fn parse_strategy(text: &str) -> Option<Strategy> {
    let text = text.trim().to_lowercase();
    // "ours | theirs | merged | deleted" copied verbatim from the prompt is not an answer.
    if text.contains('|') {
        return None;
    }
    if text.contains("delet") || text.contains("remov") {
        Some(Strategy::Deleted)
    } else if text.contains("both") || text.contains("merg") || text.contains("combin") {
        Some(Strategy::Merged)
    } else if text.contains("ours") || text.contains("our ") || text == "our" {
        Some(Strategy::Ours)
    } else if text.contains("theirs") || text.contains("their") {
        Some(Strategy::Theirs)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const JSON: &str = r#"{"files":[{"path":"a.rs","strategy":"merged","summary":"kept both"},{"path":"b.txt","strategy":"ours","summary":"theirs was stale"}]}"#;

    fn check(summary: AgentSummary) {
        assert_eq!(summary.files.len(), 2);
        assert_eq!(
            summary.entry("a.rs").unwrap().strategy,
            Some(Strategy::Merged)
        );
        assert_eq!(summary.entry("b.txt").unwrap().summary, "theirs was stale");
    }

    #[test]
    fn reads_a_fenced_json_block_at_the_end() {
        check(parse(&format!("All done.\n\n```json\n{JSON}\n```")).unwrap());
    }

    #[test]
    fn reads_a_fence_without_a_language_tag() {
        check(parse(&format!("```\n{JSON}\n```")).unwrap());
    }

    #[test]
    fn reads_bare_json_and_json_buried_in_prose() {
        check(parse(JSON).unwrap());
        check(parse(&format!("Here is what I did: {JSON} Hope that helps.")).unwrap());
    }

    #[test]
    fn the_last_valid_block_wins() {
        let message = format!(
            "```json\n{{\"files\":[{{\"path\":\"old\",\"summary\":\"x\"}}]}}\n```\nrevised:\n```json\n{JSON}\n```"
        );
        check(parse(&message).unwrap());
    }

    #[test]
    fn braces_inside_strings_do_not_break_extraction() {
        let message = r#"x {"files":[{"path":"a","strategy":"merged","summary":"kept {braces} and \"quotes\""}]} y"#;
        let summary = parse(message).unwrap();
        assert_eq!(summary.files[0].summary, "kept {braces} and \"quotes\"");
    }

    #[test]
    fn no_json_or_the_wrong_shape_is_none_not_an_error() {
        assert_eq!(parse("I resolved everything, you're welcome."), None);
        assert_eq!(parse(r#"{"result": "ok"}"#), None);
        assert_eq!(parse(r#"{"files": "nope"}"#), None);
        assert_eq!(parse(r#"{"files": [{"nopath": 1}]}"#), None);
        assert_eq!(parse(""), None);
    }

    #[test]
    fn an_empty_files_list_is_a_valid_empty_summary() {
        assert_eq!(parse(r#"{"files": []}"#), Some(AgentSummary::default()));
    }

    #[test]
    fn strategy_wording_is_understood_and_the_template_placeholder_is_not() {
        assert_eq!(parse_strategy("Ours"), Some(Strategy::Ours));
        assert_eq!(parse_strategy("took theirs"), Some(Strategy::Theirs));
        assert_eq!(parse_strategy("merged both"), Some(Strategy::Merged));
        assert_eq!(parse_strategy("deleted"), Some(Strategy::Deleted));
        assert_eq!(parse_strategy("ours | theirs | merged | deleted"), None);
        assert_eq!(parse_strategy("shrug"), None);
    }

    #[test]
    fn accepts_reason_as_a_synonym_for_summary() {
        let summary = parse(r#"{"files":[{"path":"a","reason":"because"}]}"#).unwrap();
        assert_eq!(summary.files[0].summary, "because");
    }
}

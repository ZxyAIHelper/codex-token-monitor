use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodexEvent {
    TokenCount(TokenCountEvent),
    ToolOutput(ToolOutputEvent),
    SessionMeta(SessionMetaEvent),
    Ignored,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenCountEvent {
    pub timestamp: String,
    pub input_tokens: i64,
    pub cached_input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_output_tokens: i64,
    pub total_tokens: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolOutputEvent {
    pub timestamp: String,
    pub output_bytes: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionMetaEvent {
    pub session_id: String,
    pub cwd: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MessageDetail {
    pub timestamp: String,
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
struct RawLine {
    timestamp: Option<String>,
    #[serde(rename = "type")]
    kind: Option<String>,
    payload: Option<serde_json::Value>,
}

pub fn parse_jsonl_line(line: &str) -> Result<CodexEvent, serde_json::Error> {
    let raw: RawLine = serde_json::from_str(line)?;
    let timestamp = raw.timestamp.unwrap_or_default();
    let Some(payload) = raw.payload else {
        return Ok(CodexEvent::Ignored);
    };

    if raw.kind.as_deref() == Some("event_msg")
        && payload.get("type").and_then(|value| value.as_str()) == Some("token_count")
    {
        let Some(usage) = payload
            .get("info")
            .and_then(|info| info.get("last_token_usage"))
        else {
            return Ok(CodexEvent::Ignored);
        };

        return Ok(CodexEvent::TokenCount(TokenCountEvent {
            timestamp,
            input_tokens: token_field(usage, "input_tokens"),
            cached_input_tokens: token_field(usage, "cached_input_tokens"),
            output_tokens: token_field(usage, "output_tokens"),
            reasoning_output_tokens: token_field(usage, "reasoning_output_tokens"),
            total_tokens: token_field(usage, "total_tokens"),
        }));
    }

    if raw.kind.as_deref() == Some("response_item")
        && payload.get("type").and_then(|value| value.as_str()) == Some("function_call_output")
    {
        let output = payload
            .get("output")
            .and_then(|value| value.as_str())
            .unwrap_or("");

        return Ok(CodexEvent::ToolOutput(ToolOutputEvent {
            timestamp,
            output_bytes: output.len() as i64,
        }));
    }

    if raw.kind.as_deref() == Some("session_meta") {
        let session_id = payload
            .get("id")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string();
        let cwd = payload
            .get("cwd")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string();
        if !session_id.is_empty() || !cwd.is_empty() {
            return Ok(CodexEvent::SessionMeta(SessionMetaEvent {
                session_id,
                cwd,
            }));
        }
    }

    Ok(CodexEvent::Ignored)
}

pub fn read_message_details(path: &Path) -> Result<Vec<MessageDetail>, String> {
    let reader = BufReader::new(File::open(path).map_err(|err| err.to_string())?);
    let mut messages = Vec::new();

    for line in reader.lines() {
        let line = line.map_err(|err| err.to_string())?;
        match parse_message_line(line.trim_end_matches(['\r', '\n'])) {
            Ok(Some(message)) => messages.push(message),
            Ok(None) => {}
            Err(_) => {}
        }
    }

    Ok(messages)
}

pub fn parse_message_line(line: &str) -> Result<Option<MessageDetail>, serde_json::Error> {
    let raw: RawLine = serde_json::from_str(line)?;
    let timestamp = raw.timestamp.unwrap_or_default();
    let Some(payload) = raw.payload else {
        return Ok(None);
    };

    if raw.kind.as_deref() != Some("response_item") {
        return Ok(None);
    }

    match payload.get("type").and_then(|value| value.as_str()) {
        Some("message") => {
            let role = payload
                .get("role")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown")
                .to_string();
            let content = message_content(&payload);
            if content.is_empty() {
                return Ok(None);
            }
            Ok(Some(MessageDetail {
                timestamp,
                role,
                content,
            }))
        }
        Some("function_call") => {
            let name = payload
                .get("name")
                .and_then(|value| value.as_str())
                .unwrap_or("tool");
            let arguments = payload
                .get("arguments")
                .and_then(|value| value.as_str())
                .unwrap_or("");
            let content = if arguments.is_empty() {
                name.to_string()
            } else {
                format!("{name}\n{arguments}")
            };
            Ok(Some(MessageDetail {
                timestamp,
                role: "tool".to_string(),
                content,
            }))
        }
        Some("function_call_output") => {
            let content = payload
                .get("output")
                .and_then(|value| value.as_str())
                .unwrap_or("")
                .to_string();
            if content.is_empty() {
                return Ok(None);
            }
            Ok(Some(MessageDetail {
                timestamp,
                role: "tool".to_string(),
                content,
            }))
        }
        _ => Ok(None),
    }
}

fn message_content(payload: &serde_json::Value) -> String {
    payload
        .get("content")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("text").and_then(|value| value.as_str()))
                .filter(|text| !text.is_empty())
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
}

fn token_field(usage: &serde_json::Value, field: &str) -> i64 {
    usage
        .get(field)
        .and_then(|value| value.as_i64())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_role_message_content() {
        let line = r#"{"timestamp":"2026-06-28T01:02:03Z","type":"response_item","payload":{"type":"message","role":"developer","content":[{"type":"input_text","text":"Use concise output."},{"type":"input_text","text":"Do the task."}]}}"#;

        let message = parse_message_line(line).unwrap().unwrap();

        assert_eq!(message.timestamp, "2026-06-28T01:02:03Z");
        assert_eq!(message.role, "developer");
        assert_eq!(message.content, "Use concise output.\nDo the task.");
    }

    #[test]
    fn parses_tool_output_as_tool_role() {
        let line = r#"{"timestamp":"2026-06-28T01:02:04Z","type":"response_item","payload":{"type":"function_call_output","output":"command output"}}"#;

        let message = parse_message_line(line).unwrap().unwrap();

        assert_eq!(message.role, "tool");
        assert_eq!(message.content, "command output");
    }
}

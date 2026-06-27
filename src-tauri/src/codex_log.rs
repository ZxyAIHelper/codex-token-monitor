use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodexEvent {
    TokenCount(TokenCountEvent),
    ToolOutput(ToolOutputEvent),
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
        let usage = payload
            .get("info")
            .and_then(|info| info.get("last_token_usage"));

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

    Ok(CodexEvent::Ignored)
}

fn token_field(usage: Option<&serde_json::Value>, field: &str) -> i64 {
    usage
        .and_then(|value| value.get(field))
        .and_then(|value| value.as_i64())
        .unwrap_or(0)
}

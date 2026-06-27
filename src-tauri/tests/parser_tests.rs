use codex_token_monitor_lib::codex_log::{parse_jsonl_line, CodexEvent};

#[test]
fn parser_tests_parses_token_count_last_usage() {
    let line = r#"{"timestamp":"2026-06-27T12:00:00Z","type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":100,"cached_input_tokens":40,"output_tokens":7,"reasoning_output_tokens":3,"total_tokens":107}}}}"#;
    let event = parse_jsonl_line(line).unwrap();

    match event {
        CodexEvent::TokenCount(e) => {
            assert_eq!(e.timestamp, "2026-06-27T12:00:00Z");
            assert_eq!(e.input_tokens, 100);
            assert_eq!(e.cached_input_tokens, 40);
            assert_eq!(e.output_tokens, 7);
            assert_eq!(e.reasoning_output_tokens, 3);
            assert_eq!(e.total_tokens, 107);
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn parser_tests_parses_function_call_output_size_as_bytes() {
    let output = "Exit code: 0\nOutput: hello 世界";
    let line = r#"{"timestamp":"2026-06-27T12:00:01Z","type":"response_item","payload":{"type":"function_call_output","call_id":"call_1","output":"Exit code: 0\nOutput: hello 世界"}}"#;
    let event = parse_jsonl_line(line).unwrap();

    match event {
        CodexEvent::ToolOutput(e) => {
            assert_eq!(e.timestamp, "2026-06-27T12:00:01Z");
            assert_eq!(e.output_bytes, output.len() as i64);
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn parser_tests_ignores_token_count_with_null_info() {
    let line = r#"{"timestamp":"2026-06-27T12:00:00Z","type":"event_msg","payload":{"type":"token_count","info":null}}"#;
    let event = parse_jsonl_line(line).unwrap();

    assert!(matches!(event, CodexEvent::Ignored));
}

#[test]
fn parser_tests_ignores_token_count_missing_last_usage() {
    let line = r#"{"timestamp":"2026-06-27T12:00:00Z","type":"event_msg","payload":{"type":"token_count","info":{}}}"#;
    let event = parse_jsonl_line(line).unwrap();

    assert!(matches!(event, CodexEvent::Ignored));
}

#[test]
fn parser_tests_ignores_unrelated_json_lines() {
    let line = r#"{"timestamp":"2026-06-27T12:00:02Z","type":"response_item","payload":{"type":"message","role":"assistant","content":[]}}"#;
    let event = parse_jsonl_line(line).unwrap();

    assert!(matches!(event, CodexEvent::Ignored));
}

#[test]
fn parser_tests_invalid_json_returns_error() {
    let err = parse_jsonl_line(r#"{"timestamp":"2026-06-27T12:00:03Z""#).unwrap_err();

    assert!(err.is_syntax() || err.is_eof());
}

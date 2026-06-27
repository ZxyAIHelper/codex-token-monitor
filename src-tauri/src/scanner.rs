use std::{
    fs::File,
    io::{BufRead, BufReader, Seek, SeekFrom},
    path::{Path, PathBuf},
};

use crate::{
    codex_log::{parse_jsonl_line, CodexEvent},
    usage_store::UsageStore,
};

pub fn scan_file(store: &UsageStore, path: &Path, offset: u64) -> Result<u64, String> {
    let session_id = extract_session_id(path)
        .ok_or_else(|| format!("cannot extract session id from {}", path.display()))?;
    let path_text = path.to_string_lossy().to_string();
    let mut reader = BufReader::new(File::open(path).map_err(|err| err.to_string())?);
    reader
        .seek(SeekFrom::Start(offset))
        .map_err(|err| err.to_string())?;

    let mut line = String::new();
    let mut current_offset = offset;
    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).map_err(|err| err.to_string())?;
        if bytes_read == 0 {
            break;
        }
        let line_start_offset = current_offset;
        current_offset += bytes_read as u64;
        let complete_line = line.ends_with('\n');

        match parse_jsonl_line(line.trim_end_matches(['\r', '\n'])) {
            Ok(CodexEvent::TokenCount(event)) => {
                tauri::async_runtime::block_on(store.record_token_count(
                    &session_id,
                    &path_text,
                    event,
                ))
                .map_err(|err| err.to_string())?;
            }
            Ok(CodexEvent::ToolOutput(event)) => {
                tauri::async_runtime::block_on(store.record_tool_output(
                    &session_id,
                    &path_text,
                    line_start_offset,
                    event,
                ))
                .map_err(|err| err.to_string())?;
            }
            Ok(CodexEvent::Ignored) => {}
            // Complete corrupt rows are skipped. An invalid unterminated final row
            // may still be in progress, so leave the offset at the row start.
            Err(_) if complete_line => {}
            Err(_) => return Ok(line_start_offset),
        }
    }

    Ok(current_offset)
}

pub fn default_codex_sessions_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".codex").join("sessions"))
}

pub fn should_scan_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("jsonl"))
}

pub fn extract_session_id(path: &Path) -> Option<String> {
    if !should_scan_path(path) {
        return None;
    }

    let stem = path.file_stem()?.to_str()?;
    let rest = stem.strip_prefix("rollout-")?;
    let timestamp = rest.get(..19)?;
    let session_id = rest.get(20..)?;
    if rest.get(19..20)? != "-" {
        return None;
    }
    if !is_rollout_timestamp(timestamp) {
        return None;
    }

    if is_uuid_like(session_id) {
        Some(session_id.to_string())
    } else {
        None
    }
}

fn is_rollout_timestamp(value: &str) -> bool {
    value.len() == 19
        && value.char_indices().all(|(idx, ch)| match idx {
            4 | 7 | 13 | 16 => ch == '-',
            10 => ch == 'T',
            _ => ch.is_ascii_digit(),
        })
}

fn is_uuid_like(value: &str) -> bool {
    value.len() == 36
        && value.char_indices().all(|(idx, ch)| match idx {
            8 | 13 | 18 | 23 => ch == '-',
            _ => ch.is_ascii_hexdigit(),
        })
}

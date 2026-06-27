use std::path::Path;

pub fn should_scan_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("jsonl"))
}

pub fn extract_session_id(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_str()?;
    if !stem.starts_with("rollout-") {
        return None;
    }

    let session_id = stem.get(stem.len().checked_sub(36)?..)?;
    if is_uuid_like(session_id) {
        Some(session_id.to_string())
    } else {
        None
    }
}

fn is_uuid_like(value: &str) -> bool {
    value.len() == 36
        && value.char_indices().all(|(idx, ch)| match idx {
            8 | 13 | 18 | 23 => ch == '-',
            _ => ch.is_ascii_hexdigit(),
        })
}

use std::path::Path;

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

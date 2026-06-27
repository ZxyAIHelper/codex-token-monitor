use serde::Serialize;

pub const HOURLY_WARNING_TOKENS: i64 = 1_000_000;
pub const HOURLY_CRITICAL_TOKENS: i64 = 3_000_000;
pub const SESSION_WARNING_TOKENS: i64 = 3_000_000;
pub const SESSION_CRITICAL_TOKENS: i64 = 10_000_000;
pub const TOOL_OUTPUT_WARNING_BYTES: i64 = 50 * 1024;
pub const TOOL_OUTPUT_CRITICAL_BYTES: i64 = 200 * 1024;
// The current parser persists tool output byte counts, not payload content, so
// image/base64-specific alerting is not available beyond large output alerts.
pub const BASE64_WARNING_BYTES: i64 = 100 * 1024;
pub const BASE64_CRITICAL_BYTES: i64 = 500 * 1024;

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertLevel {
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum AlertKind {
    LargeToolOutput,
    HighSessionUsage,
    HighHourlyUsage,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AlertItem {
    pub level: AlertLevel,
    pub kind: AlertKind,
    pub message: String,
    pub timestamp: String,
    pub session_id: Option<String>,
}

pub fn token_level(value: i64, warning: i64, critical: i64) -> Option<AlertLevel> {
    if value > critical {
        Some(AlertLevel::Critical)
    } else if value > warning {
        Some(AlertLevel::Warning)
    } else {
        None
    }
}

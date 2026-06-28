export interface DashboardSummary {
  today_total_tokens: number;
  last_hour_tokens: number;
  last_five_hours_tokens: number;
  active_session_count: number;
  input_tokens: number;
  output_tokens: number;
}

export type AlertLevel = "Warning" | "Critical";

export type AlertKind = "LargeToolOutput" | "HighSessionUsage" | "HighHourlyUsage";

export interface AlertItem {
  level: AlertLevel;
  kind: AlertKind;
  message: string;
  timestamp: string;
  session_id: string | null;
}

export interface SessionSummary {
  session_id: string;
  session_name: string;
  cwd: string;
  path: string;
  total_tokens: number;
  input_tokens: number;
  cached_input_tokens: number;
  output_tokens: number;
  reasoning_output_tokens: number;
  tool_calls: number;
  tool_output_bytes: number;
  last_seen_at: string;
}

export interface TurnDetail {
  timestamp: string;
  total_tokens: number;
  input_tokens: number;
  cached_input_tokens: number;
  output_tokens: number;
  reasoning_output_tokens: number;
}

export interface TimeBucket {
  bucket: string;
  total_tokens: number;
  input_tokens: number;
  output_tokens: number;
  session_count: number;
}

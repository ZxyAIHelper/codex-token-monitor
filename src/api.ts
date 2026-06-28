import { invoke } from "@tauri-apps/api/core";
import type {
  AlertItem,
  DashboardSummary,
  MessageDetail,
  ModelRequestDetail,
  SessionSummary,
  TimeBucket,
  TurnDetail,
} from "./types";

export function dashboardSummary(): Promise<DashboardSummary> {
  return invoke<DashboardSummary>("dashboard_summary");
}

export function listSessions(): Promise<SessionSummary[]> {
  return invoke<SessionSummary[]>("list_sessions");
}

export function getAlerts(): Promise<AlertItem[]> {
  return invoke<AlertItem[]>("list_alerts");
}

export function hourlyTotals(): Promise<TimeBucket[]> {
  return invoke<TimeBucket[]>("hourly_totals");
}

export function dailyTotals(): Promise<TimeBucket[]> {
  return invoke<TimeBucket[]>("daily_totals");
}

export function getSessionTurns(sessionId: string): Promise<TurnDetail[]> {
  return invoke<TurnDetail[]>("session_turns", { sessionId });
}

export function getSessionMessages(sessionId: string): Promise<MessageDetail[]> {
  return invoke<MessageDetail[]>("session_messages", { sessionId });
}

export function getSessionModelRequests(sessionId: string): Promise<ModelRequestDetail[]> {
  return invoke<ModelRequestDetail[]>("session_model_requests", { sessionId });
}

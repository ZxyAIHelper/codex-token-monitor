import { invoke } from "@tauri-apps/api/core";
import type { DashboardSummary, SessionSummary, TimeBucket, TurnDetail } from "./types";

export function dashboardSummary(): Promise<DashboardSummary> {
  return invoke<DashboardSummary>("dashboard_summary");
}

export function listSessions(): Promise<SessionSummary[]> {
  return invoke<SessionSummary[]>("list_sessions");
}

export function hourlyTotals(): Promise<TimeBucket[]> {
  return invoke<TimeBucket[]>("hourly_totals");
}

export function getSessionTurns(sessionId: string): Promise<TurnDetail[]> {
  return invoke<TurnDetail[]>("session_turns", { sessionId });
}

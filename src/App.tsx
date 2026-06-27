import { useCallback, useEffect, useState } from "react";
import { dashboardSummary, hourlyTotals, listSessions } from "./api";
import { AlertList } from "./components/AlertList";
import { HourlyTrend } from "./components/HourlyTrend";
import { SessionTable } from "./components/SessionTable";
import { SummaryCards } from "./components/SummaryCards";
import type { DashboardSummary, SessionSummary, TimeBucket } from "./types";

const REFRESH_INTERVAL_MS = 5_000;

function App() {
  const [summary, setSummary] = useState<DashboardSummary | null>(null);
  const [sessions, setSessions] = useState<SessionSummary[]>([]);
  const [hourlyBuckets, setHourlyBuckets] = useState<TimeBucket[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [updatedAt, setUpdatedAt] = useState<Date | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  const loadDashboard = useCallback(async () => {
    try {
      const [nextSummary, nextSessions, nextHourlyBuckets] = await Promise.all([
        dashboardSummary(),
        listSessions(),
        hourlyTotals(),
      ]);

      setSummary(nextSummary);
      setSessions(nextSessions);
      setHourlyBuckets(nextHourlyBuckets);
      setUpdatedAt(new Date());
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    void loadDashboard();
    const refreshId = window.setInterval(() => {
      void loadDashboard();
    }, REFRESH_INTERVAL_MS);

    return () => window.clearInterval(refreshId);
  }, [loadDashboard]);

  return (
    <main className="app-shell">
      <header className="app-header">
        <div>
          <h1>Codex Token Monitor</h1>
          <p>Live session usage dashboard</p>
        </div>
        <div className="header-status">
          <span className={error ? "status-dot status-error" : "status-dot"} />
          <span>{error ? "Disconnected" : "Live"}</span>
          {updatedAt ? <time>{updatedAt.toLocaleTimeString()}</time> : null}
        </div>
      </header>

      {error ? <div className="banner">Unable to load dashboard data: {error}</div> : null}
      {isLoading ? <div className="banner banner-muted">Loading dashboard data...</div> : null}

      <SummaryCards summary={summary} />
      <HourlyTrend buckets={hourlyBuckets} />

      <section className="lower-grid">
        <SessionTable sessions={sessions} />
        <AlertList />
      </section>
    </main>
  );
}

export default App;

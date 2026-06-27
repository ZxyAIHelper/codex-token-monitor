import { useCallback, useEffect, useRef, useState } from "react";
import { dashboardSummary, getAlerts, getSessionTurns, hourlyTotals, listSessions } from "./api";
import { AlertList } from "./components/AlertList";
import { HourlyTrend } from "./components/HourlyTrend";
import { SessionDetail } from "./components/SessionDetail";
import { SessionTable } from "./components/SessionTable";
import { SummaryCards } from "./components/SummaryCards";
import type { AlertItem, DashboardSummary, SessionSummary, TimeBucket, TurnDetail } from "./types";

const REFRESH_INTERVAL_MS = 5_000;

function App() {
  const [summary, setSummary] = useState<DashboardSummary | null>(null);
  const [sessions, setSessions] = useState<SessionSummary[]>([]);
  const [hourlyBuckets, setHourlyBuckets] = useState<TimeBucket[]>([]);
  const [alerts, setAlerts] = useState<AlertItem[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [updatedAt, setUpdatedAt] = useState<Date | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [selectedSession, setSelectedSession] = useState<SessionSummary | null>(null);
  const [turns, setTurns] = useState<TurnDetail[]>([]);
  const [turnsError, setTurnsError] = useState<string | null>(null);
  const [areTurnsLoading, setAreTurnsLoading] = useState(false);
  const detailRequestId = useRef(0);

  useEffect(() => {
    let isMounted = true;
    let isRequestInFlight = false;
    let latestRequestId = 0;

    const loadDashboard = async () => {
      if (isRequestInFlight) {
        return;
      }

      isRequestInFlight = true;
      const requestId = latestRequestId + 1;
      latestRequestId = requestId;

      try {
        const [nextSummary, nextSessions, nextHourlyBuckets, nextAlerts] = await Promise.all([
          dashboardSummary(),
          listSessions(),
          hourlyTotals(),
          getAlerts(),
        ]);

        if (!isMounted || requestId !== latestRequestId) {
          return;
        }

        setSummary(nextSummary);
        setSessions(nextSessions);
        setHourlyBuckets(nextHourlyBuckets);
        setAlerts(nextAlerts);
        setUpdatedAt(new Date());
        setError(null);
      } catch (err) {
        if (!isMounted || requestId !== latestRequestId) {
          return;
        }

        setError(err instanceof Error ? err.message : String(err));
      } finally {
        if (isMounted && requestId === latestRequestId) {
          setIsLoading(false);
        }

        if (requestId === latestRequestId) {
          isRequestInFlight = false;
        }
      }
    };

    void loadDashboard();
    const refreshId = window.setInterval(() => {
      void loadDashboard();
    }, REFRESH_INTERVAL_MS);

    return () => {
      isMounted = false;
      latestRequestId += 1;
      window.clearInterval(refreshId);
    };
  }, []);

  const handleOpenSession = useCallback(async (session: SessionSummary) => {
    const requestId = detailRequestId.current + 1;
    detailRequestId.current = requestId;

    setSelectedSession(session);
    setTurns([]);
    setTurnsError(null);
    setAreTurnsLoading(true);

    try {
      const nextTurns = await getSessionTurns(session.session_id);
      if (detailRequestId.current !== requestId) {
        return;
      }
      setTurns(nextTurns);
      setTurnsError(null);
    } catch (err) {
      if (detailRequestId.current !== requestId) {
        return;
      }
      setTurnsError(err instanceof Error ? err.message : String(err));
    } finally {
      if (detailRequestId.current === requestId) {
        setAreTurnsLoading(false);
      }
    }
  }, []);

  const handleBackToDashboard = useCallback(() => {
    detailRequestId.current += 1;
    setSelectedSession(null);
    setTurns([]);
    setTurnsError(null);
    setAreTurnsLoading(false);
  }, []);

  if (selectedSession) {
    return (
      <main className="app-shell">
        <SessionDetail
          session={selectedSession}
          turns={turns}
          isLoading={areTurnsLoading}
          error={turnsError}
          onBack={handleBackToDashboard}
        />
      </main>
    );
  }

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
        <SessionTable sessions={sessions} onOpenSession={handleOpenSession} />
        <AlertList alerts={alerts} />
      </section>
    </main>
  );
}

export default App;

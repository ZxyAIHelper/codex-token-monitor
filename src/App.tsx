import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  dailyTotals,
  dashboardSummary,
  getAlerts,
  getSessionModelRequests,
  hourlyTotals,
  listSessions,
} from "./api";
import { AlertList } from "./components/AlertList";
import { HourlyTrend } from "./components/HourlyTrend";
import { SessionDetail } from "./components/SessionDetail";
import { SessionTable } from "./components/SessionTable";
import { SummaryCards } from "./components/SummaryCards";
import { createTranslator, resolveLanguage, type Language } from "./i18n";
import type { AlertItem, DashboardSummary, ModelRequestDetail, SessionSummary, TimeBucket } from "./types";

const REFRESH_INTERVAL_MS = 5_000;
const LANGUAGE_STORAGE_KEY = "codex-token-monitor-language";

function App() {
  const [language, setLanguage] = useState<Language>(() =>
    resolveLanguage(window.localStorage.getItem(LANGUAGE_STORAGE_KEY) ?? window.navigator.language),
  );
  const t = useMemo(() => createTranslator(language), [language]);
  const [summary, setSummary] = useState<DashboardSummary | null>(null);
  const [sessions, setSessions] = useState<SessionSummary[]>([]);
  const [hourlyBuckets, setHourlyBuckets] = useState<TimeBucket[]>([]);
  const [dailyBuckets, setDailyBuckets] = useState<TimeBucket[]>([]);
  const [alerts, setAlerts] = useState<AlertItem[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [updatedAt, setUpdatedAt] = useState<Date | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [selectedSession, setSelectedSession] = useState<SessionSummary | null>(null);
  const [requests, setRequests] = useState<ModelRequestDetail[]>([]);
  const [turnsError, setTurnsError] = useState<string | null>(null);
  const [areTurnsLoading, setAreTurnsLoading] = useState(false);
  const detailRequestId = useRef(0);

  useEffect(() => {
    window.localStorage.setItem(LANGUAGE_STORAGE_KEY, language);
    document.documentElement.lang = language === "zh" ? "zh-CN" : "en";
    document.title = t("app.title");
  }, [language, t]);

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
        const [nextSummary, nextSessions, nextHourlyBuckets, nextDailyBuckets, nextAlerts] = await Promise.all([
          dashboardSummary(),
          listSessions(),
          hourlyTotals(),
          dailyTotals(),
          getAlerts(),
        ]);

        if (!isMounted || requestId !== latestRequestId) {
          return;
        }

        setSummary(nextSummary);
        setSessions(nextSessions);
        setHourlyBuckets(nextHourlyBuckets);
        setDailyBuckets(nextDailyBuckets);
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
    setRequests([]);
    setTurnsError(null);
    setAreTurnsLoading(true);

    try {
      const nextRequests = await getSessionModelRequests(session.session_id);
      if (detailRequestId.current !== requestId) {
        return;
      }
      setRequests(nextRequests);
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
    setRequests([]);
    setTurnsError(null);
    setAreTurnsLoading(false);
  }, []);

  if (selectedSession) {
    return (
      <main className="app-shell">
        <SessionDetail
          session={selectedSession}
          requests={requests}
          isLoading={areTurnsLoading}
          error={turnsError}
          onBack={handleBackToDashboard}
          t={t}
        />
      </main>
    );
  }

  return (
    <main className="app-shell">
      <header className="app-header">
        <div>
          <h1>{t("app.title")}</h1>
          <p>{t("dashboard.subtitle")}</p>
        </div>
        <div className="header-status">
          <span className={error ? "status-dot status-error" : "status-dot"} />
          <span>{error ? t("status.disconnected") : t("status.live")}</span>
          {updatedAt ? <time>{updatedAt.toLocaleTimeString()}</time> : null}
          <label className="language-picker">
            <span>{t("language.label")}</span>
            <select value={language} onChange={(event) => setLanguage(resolveLanguage(event.target.value))}>
              <option value="en">{t("language.en")}</option>
              <option value="zh">{t("language.zh")}</option>
            </select>
          </label>
        </div>
      </header>

      {error ? <div className="banner">{t("dashboard.loadError", { error })}</div> : null}
      {isLoading ? <div className="banner banner-muted">{t("dashboard.loading")}</div> : null}

      <SummaryCards summary={summary} t={t} />
      <HourlyTrend hourlyBuckets={hourlyBuckets} dailyBuckets={dailyBuckets} t={t} />

      <section className="lower-grid">
        <SessionTable sessions={sessions} onOpenSession={handleOpenSession} t={t} />
        <AlertList alerts={alerts} t={t} />
      </section>
    </main>
  );
}

export default App;

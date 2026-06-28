import { useEffect, useMemo, useState } from "react";
import type { Translator } from "../i18n";
import { compactSessionId, sessionDisplayTitle, sessionSubtitle } from "../sessionTitles";
import type { SessionSummary } from "../types";

interface SessionTableProps {
  sessions: SessionSummary[];
  onOpenSession: (session: SessionSummary) => void;
  t: Translator;
}

const PAGE_SIZE = 20;

const compactNumber = new Intl.NumberFormat("en", {
  notation: "compact",
  maximumFractionDigits: 1,
});

type SortDirection = "asc" | "desc";
type SortKey = "session" | "total" | "input" | "output" | "tools" | "toolBytes" | "lastSeen";

interface SortState {
  key: SortKey;
  direction: SortDirection;
}

const sortLabelKeys: Record<SortKey, Parameters<Translator>[0]> = {
  session: "columns.session",
  total: "columns.total",
  input: "columns.input",
  output: "columns.output",
  tools: "columns.tools",
  toolBytes: "columns.toolBytes",
  lastSeen: "columns.lastSeen",
};

function formatNumber(value: number): string {
  return compactNumber.format(value);
}

function formatKilobytes(bytes: number): string {
  return `${(bytes / 1024).toLocaleString(undefined, { maximumFractionDigits: 1 })} KB`;
}

function formatLastSeen(timestamp: string): string {
  const date = new Date(timestamp);
  if (Number.isNaN(date.getTime())) {
    return timestamp || "-";
  }
  return date.toLocaleString([], {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}

function sortValue(session: SessionSummary, key: SortKey): string | number {
  switch (key) {
    case "session":
      return sessionDisplayTitle(session);
    case "total":
      return session.total_tokens;
    case "input":
      return session.input_tokens;
    case "output":
      return session.output_tokens;
    case "tools":
      return session.tool_calls;
    case "toolBytes":
      return session.tool_output_bytes;
    case "lastSeen":
      return Date.parse(session.last_seen_at) || 0;
  }
}

function compareSessions(a: SessionSummary, b: SessionSummary, sort: SortState): number {
  const left = sortValue(a, sort.key);
  const right = sortValue(b, sort.key);
  const multiplier = sort.direction === "asc" ? 1 : -1;

  if (typeof left === "string" && typeof right === "string") {
    return left.localeCompare(right, undefined, { numeric: true, sensitivity: "base" }) * multiplier;
  }

  return ((left as number) - (right as number)) * multiplier;
}

export function SessionTable({ sessions, onOpenSession, t }: SessionTableProps) {
  const [sort, setSort] = useState<SortState>({ key: "total", direction: "desc" });
  const [page, setPage] = useState(1);
  const orderedSessions = useMemo(() => [...sessions].sort((a, b) => compareSessions(a, b, sort)), [sessions, sort]);
  const totalPages = Math.max(1, Math.ceil(orderedSessions.length / PAGE_SIZE));
  const safePage = Math.min(page, totalPages);
  const pageStart = (safePage - 1) * PAGE_SIZE;
  const pageSessions = orderedSessions.slice(pageStart, pageStart + PAGE_SIZE);

  useEffect(() => {
    if (page > totalPages) {
      setPage(totalPages);
    }
  }, [page, totalPages]);

  const setSortKey = (key: SortKey) => {
    setSort((current) => ({
      key,
      direction: current.key === key && current.direction === "desc" ? "asc" : "desc",
    }));
    setPage(1);
  };

  const sortSummary = t("sessions.sortedBy", {
    label: t(sortLabelKeys[sort.key]),
    direction: sort.direction === "desc" ? t("sessions.desc") : t("sessions.asc"),
  });
  const renderSortHeader = (key: SortKey) => {
    const isActive = sort.key === key;
    const directionLabel = sort.direction === "desc" ? t("sessions.desc") : t("sessions.asc");
    const ariaSort = isActive ? (sort.direction === "desc" ? "descending" : "ascending") : "none";

    return (
      <button
        type="button"
        className={`sort-button${isActive ? " sort-button-active" : ""}`}
        aria-label={t("sessions.sortBy", { label: t(sortLabelKeys[key]) })}
        aria-sort={ariaSort}
        onClick={() => setSortKey(key)}
      >
        <span>{t(sortLabelKeys[key])}</span>
        <span className="sort-indicator">{isActive ? (sort.direction === "desc" ? "v" : "^") : "-"}</span>
        {isActive ? <span className="sr-only">{directionLabel}</span> : null}
      </button>
    );
  };

  return (
    <section className="panel sessions-panel">
      <div className="panel-header">
        <div>
          <h2>{t("sessions.title")}</h2>
          <p>{sortSummary}</p>
        </div>
        <span className="panel-count">{orderedSessions.length}</span>
      </div>

      {orderedSessions.length === 0 ? (
        <div className="empty-state">{t("sessions.none")}</div>
      ) : (
        <>
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>{renderSortHeader("session")}</th>
                  <th>{renderSortHeader("total")}</th>
                  <th>{renderSortHeader("input")}</th>
                  <th>{renderSortHeader("output")}</th>
                  <th>{renderSortHeader("tools")}</th>
                  <th>{renderSortHeader("toolBytes")}</th>
                  <th>{renderSortHeader("lastSeen")}</th>
                </tr>
              </thead>
              <tbody>
                {pageSessions.map((session) => (
                  <tr key={session.session_id}>
                    <td>
                      <button
                        type="button"
                        className="session-action session-cell"
                        aria-label={t("sessions.open", { title: sessionDisplayTitle(session) })}
                        onClick={() => onOpenSession(session)}
                      >
                        <span title={session.session_name || session.session_id}>{sessionDisplayTitle(session)}</span>
                        <small title={sessionSubtitle(session)}>{sessionSubtitle(session)}</small>
                        <small title={session.session_id}>{compactSessionId(session.session_id)}</small>
                      </button>
                    </td>
                    <td>{formatNumber(session.total_tokens)}</td>
                    <td>{formatNumber(session.input_tokens)}</td>
                    <td>{formatNumber(session.output_tokens)}</td>
                    <td>{session.tool_calls.toLocaleString()}</td>
                    <td>{formatKilobytes(session.tool_output_bytes)}</td>
                    <td>{formatLastSeen(session.last_seen_at)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          <div className="pagination-bar">
            <span>
              {t("sessions.pageOf", {
                start: pageStart + 1,
                end: Math.min(pageStart + PAGE_SIZE, orderedSessions.length),
                total: orderedSessions.length,
              })}
            </span>
            <div>
              <button type="button" onClick={() => setPage((value) => Math.max(1, value - 1))} disabled={safePage === 1}>
                {t("sessions.prev")}
              </button>
              <span>
                {safePage} / {totalPages}
              </span>
              <button
                type="button"
                onClick={() => setPage((value) => Math.min(totalPages, value + 1))}
                disabled={safePage === totalPages}
              >
                {t("sessions.next")}
              </button>
            </div>
          </div>
        </>
      )}
    </section>
  );
}

import type { SessionSummary } from "../types";

interface SessionTableProps {
  sessions: SessionSummary[];
  onOpenSession: (session: SessionSummary) => void;
}

const compactNumber = new Intl.NumberFormat("en", {
  notation: "compact",
  maximumFractionDigits: 1,
});

function formatNumber(value: number): string {
  return compactNumber.format(value);
}

function formatKilobytes(bytes: number): string {
  return `${(bytes / 1024).toLocaleString(undefined, { maximumFractionDigits: 1 })} KB`;
}

function formatSessionId(sessionId: string): string {
  if (sessionId.length <= 12) {
    return sessionId;
  }
  return `${sessionId.slice(0, 8)}...${sessionId.slice(-4)}`;
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

export function SessionTable({ sessions, onOpenSession }: SessionTableProps) {
  const orderedSessions = [...sessions].sort((a, b) => b.total_tokens - a.total_tokens);

  return (
    <section className="panel sessions-panel">
      <div className="panel-header">
        <div>
          <h2>Sessions</h2>
          <p>Ordered by total tokens</p>
        </div>
        <span className="panel-count">{orderedSessions.length}</span>
      </div>

      {orderedSessions.length === 0 ? (
        <div className="empty-state">No sessions recorded.</div>
      ) : (
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>Session</th>
                <th>Total</th>
                <th>Input</th>
                <th>Output</th>
                <th>Tools</th>
                <th>Tool KB</th>
                <th>Last seen</th>
              </tr>
            </thead>
            <tbody>
              {orderedSessions.map((session) => (
                <tr key={session.session_id}>
                  <td>
                    <button
                      type="button"
                      className="session-action session-cell"
                      aria-label={`Open session ${session.session_id}`}
                      onClick={() => onOpenSession(session)}
                    >
                      <span title={session.session_id}>{formatSessionId(session.session_id)}</span>
                      <small title={session.path}>{session.path}</small>
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
      )}
    </section>
  );
}

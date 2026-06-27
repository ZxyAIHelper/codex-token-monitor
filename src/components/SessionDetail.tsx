import type { SessionSummary, TurnDetail } from "../types";

interface SessionDetailProps {
  session: SessionSummary;
  turns: TurnDetail[];
  isLoading: boolean;
  error: string | null;
  onBack: () => void;
}

const numberFormatter = new Intl.NumberFormat("en");

function formatNumber(value: number): string {
  return numberFormatter.format(value);
}

function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp);
  if (Number.isNaN(date.getTime())) {
    return timestamp || "-";
  }
  return date.toLocaleString();
}

export function SessionDetail({ session, turns, isLoading, error, onBack }: SessionDetailProps) {
  return (
    <section className="detail-view">
      <button className="back-button" type="button" onClick={onBack}>
        Back
      </button>

      <section className="panel detail-header-panel">
        <div className="detail-title">
          <h2 title={session.session_id}>{session.session_id}</h2>
          <p title={session.path}>{session.path}</p>
        </div>
        <div className="detail-metrics">
          <div>
            <span>Total</span>
            <strong>{formatNumber(session.total_tokens)}</strong>
          </div>
          <div>
            <span>Input</span>
            <strong>{formatNumber(session.input_tokens)}</strong>
          </div>
          <div>
            <span>Cached</span>
            <strong>{formatNumber(session.cached_input_tokens)}</strong>
          </div>
          <div>
            <span>Output</span>
            <strong>{formatNumber(session.output_tokens)}</strong>
          </div>
          <div>
            <span>Reasoning</span>
            <strong>{formatNumber(session.reasoning_output_tokens)}</strong>
          </div>
        </div>
      </section>

      {error ? <div className="banner">Unable to load session turns: {error}</div> : null}
      {isLoading ? <div className="banner banner-muted">Loading session turns...</div> : null}

      <section className="panel">
        <div className="panel-header">
          <div>
            <h2>Turns</h2>
            <p>Per-turn token usage</p>
          </div>
          <span className="panel-count">{turns.length}</span>
        </div>

        {!isLoading && turns.length === 0 ? (
          <div className="empty-state">No turns recorded for this session.</div>
        ) : (
          <div className="table-wrap">
            <table className="turns-table">
              <thead>
                <tr>
                  <th>Time</th>
                  <th>Total</th>
                  <th>Input</th>
                  <th>Cached</th>
                  <th>Output</th>
                  <th>Reasoning</th>
                </tr>
              </thead>
              <tbody>
                {turns.map((turn, index) => (
                  <tr key={`${turn.timestamp}-${index}`}>
                    <td title={turn.timestamp}>{formatTimestamp(turn.timestamp)}</td>
                    <td>{formatNumber(turn.total_tokens)}</td>
                    <td>{formatNumber(turn.input_tokens)}</td>
                    <td>{formatNumber(turn.cached_input_tokens)}</td>
                    <td>{formatNumber(turn.output_tokens)}</td>
                    <td>{formatNumber(turn.reasoning_output_tokens)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>
    </section>
  );
}

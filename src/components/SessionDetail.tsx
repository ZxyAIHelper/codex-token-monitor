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

function sessionTitle(session: SessionSummary): string {
  return session.session_name || session.session_id;
}

export function SessionDetail({ session, turns, isLoading, error, onBack }: SessionDetailProps) {
  return (
    <section className="detail-view">
      <button className="back-button" type="button" onClick={onBack}>
        返回
      </button>

      <section className="panel detail-header-panel">
        <div className="detail-title">
          <h2 title={sessionTitle(session)}>{sessionTitle(session)}</h2>
          <p title={session.cwd || "-"}>工作目录：{session.cwd || "-"}</p>
          <p title={session.session_id}>Session ID：{session.session_id}</p>
          <p title={session.path}>日志：{session.path}</p>
        </div>
        <div className="detail-metrics">
          <div>
            <span>总量</span>
            <strong>{formatNumber(session.total_tokens)}</strong>
          </div>
          <div>
            <span>输入</span>
            <strong>{formatNumber(session.input_tokens)}</strong>
          </div>
          <div>
            <span>缓存输入</span>
            <strong>{formatNumber(session.cached_input_tokens)}</strong>
          </div>
          <div>
            <span>输出</span>
            <strong>{formatNumber(session.output_tokens)}</strong>
          </div>
          <div>
            <span>推理输出</span>
            <strong>{formatNumber(session.reasoning_output_tokens)}</strong>
          </div>
        </div>
      </section>

      {error ? <div className="banner">无法加载会话明细：{error}</div> : null}
      {isLoading ? <div className="banner banner-muted">正在加载会话明细...</div> : null}

      <section className="panel">
        <div className="panel-header">
          <div>
            <h2>明细</h2>
            <p>每轮 token 消耗</p>
          </div>
          <span className="panel-count">{turns.length}</span>
        </div>

        {!isLoading && turns.length === 0 ? (
          <div className="empty-state">此会话暂无明细记录。</div>
        ) : (
          <div className="table-wrap">
            <table className="turns-table">
              <thead>
                <tr>
                  <th>时间</th>
                  <th>总量</th>
                  <th>输入</th>
                  <th>缓存</th>
                  <th>输出</th>
                  <th>推理</th>
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

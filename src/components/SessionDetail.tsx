import type { Translator } from "../i18n";
import { sessionDisplayTitle } from "../sessionTitles";
import type { MessageDetail, ModelRequestDetail, SessionSummary, TurnDetail } from "../types";

interface SessionDetailProps {
  session: SessionSummary;
  requests: ModelRequestDetail[];
  isLoading: boolean;
  error: string | null;
  onBack: () => void;
  t: Translator;
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

function roleClassName(role: string): string {
  const normalized = role.toLowerCase();
  if (["system", "developer", "user", "tool"].includes(normalized)) {
    return `role-badge role-${normalized}`;
  }
  return "role-badge";
}

function roleLabel(message: MessageDetail, t: Translator): string {
  return message.role || t("role.unknown");
}

function renderMetric(label: string, value: number) {
  return (
    <div>
      <span>{label}</span>
      <strong>{formatNumber(value)}</strong>
    </div>
  );
}

function RequestMetrics({ turn, t }: { turn: TurnDetail; t: Translator }) {
  return (
    <div className="request-metrics">
      <span>
        {t("columns.total")}: {formatNumber(turn.total_tokens)}
      </span>
      <span>
        {t("columns.input")}: {formatNumber(turn.input_tokens)}
      </span>
      <span>
        {t("columns.output")}: {formatNumber(turn.output_tokens)}
      </span>
    </div>
  );
}

function MessageList({ messages, t }: { messages: MessageDetail[]; t: Translator }) {
  if (messages.length === 0) {
    return <div className="empty-state empty-state-compact">{t("detail.noRequestMessages")}</div>;
  }

  return (
    <div className="message-list">
      {messages.map((message, index) => (
        <article className="message-item" key={`${message.timestamp}-${message.role}-${index}`}>
          <header>
            <span className={roleClassName(message.role)}>{roleLabel(message, t)}</span>
            <time title={message.timestamp}>{formatTimestamp(message.timestamp)}</time>
          </header>
          <pre>{message.content}</pre>
        </article>
      ))}
    </div>
  );
}

export function SessionDetail({ session, requests, isLoading, error, onBack, t }: SessionDetailProps) {
  return (
    <section className="detail-view">
      <button className="back-button" type="button" onClick={onBack}>
        {t("detail.back")}
      </button>

      <section className="panel detail-header-panel">
        <div className="detail-title">
          <h2 title={sessionDisplayTitle(session)}>{sessionDisplayTitle(session)}</h2>
          <p title={session.cwd || "-"}>{`${t("detail.cwd")}: ${session.cwd || "-"}`}</p>
          <p title={session.session_id}>{`${t("detail.sessionId")}: ${session.session_id}`}</p>
          <p title={session.path}>{`${t("detail.log")}: ${session.path}`}</p>
        </div>
        <div className="detail-metrics">
          {renderMetric(t("columns.total"), session.total_tokens)}
          {renderMetric(t("columns.input"), session.input_tokens)}
          {renderMetric(t("detail.cachedInput"), session.cached_input_tokens)}
          {renderMetric(t("columns.output"), session.output_tokens)}
          {renderMetric(t("detail.reasoning"), session.reasoning_output_tokens)}
        </div>
      </section>

      {error ? <div className="banner">{t("detail.loadError", { error })}</div> : null}
      {isLoading ? <div className="banner banner-muted">{t("detail.loading")}</div> : null}

      <section className="panel">
        <div className="panel-header">
          <div>
            <h2>{t("detail.modelRequests")}</h2>
            <p>{t("detail.modelRequestsSubtitle")}</p>
          </div>
          <span className="panel-count">{requests.length}</span>
        </div>

        {!isLoading && requests.length === 0 ? (
          <div className="empty-state">{t("detail.noModelRequests")}</div>
        ) : (
          <div className="request-list">
            {requests.map((request) => (
              <details className="request-item" key={request.request_index} open={request.request_index === 1}>
                <summary>
                  <div className="request-title">
                    <strong>{t("detail.requestLabel", { index: request.request_index })}</strong>
                    <time title={request.turn.timestamp}>{formatTimestamp(request.turn.timestamp)}</time>
                  </div>
                  <RequestMetrics turn={request.turn} t={t} />
                </summary>
                <MessageList messages={request.messages} t={t} />
              </details>
            ))}
          </div>
        )}
      </section>
    </section>
  );
}

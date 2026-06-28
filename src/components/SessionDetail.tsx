import { useMemo, useState } from "react";
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

type RequestSortDirection = "asc" | "desc";
type RequestSortKey = "time" | "total" | "input" | "output" | "cache" | "developer" | "user" | "tool";

interface RequestSortState {
  key: RequestSortKey;
  direction: RequestSortDirection;
}

const requestSortLabels: Record<RequestSortKey, string> = {
  time: "Time",
  total: "Total",
  input: "Input",
  output: "Output",
  cache: "Cache",
  developer: "Developer",
  user: "User",
  tool: "Tool",
};

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

function countRole(messages: MessageDetail[], role: string): number {
  return messages.filter((message) => message.role.toLowerCase() === role).length;
}

function requestSortValue(request: ModelRequestDetail, key: RequestSortKey): number {
  switch (key) {
    case "time":
      return Date.parse(request.turn.timestamp) || 0;
    case "total":
      return request.turn.total_tokens;
    case "input":
      return request.turn.input_tokens;
    case "output":
      return request.turn.output_tokens;
    case "cache":
      return request.turn.cached_input_tokens;
    case "developer":
      return countRole(request.messages, "developer");
    case "user":
      return countRole(request.messages, "user");
    case "tool":
      return countRole(request.messages, "tool");
  }
}

function compareRequests(a: ModelRequestDetail, b: ModelRequestDetail, sort: RequestSortState): number {
  const multiplier = sort.direction === "asc" ? 1 : -1;
  const byMetric = (requestSortValue(a, sort.key) - requestSortValue(b, sort.key)) * multiplier;
  if (byMetric !== 0) {
    return byMetric;
  }
  return a.request_index - b.request_index;
}

function RequestMetrics({ request, t }: { request: ModelRequestDetail; t: Translator }) {
  const { turn, messages } = request;
  const developerCount = countRole(messages, "developer");
  const userCount = countRole(messages, "user");
  const toolCount = countRole(messages, "tool");

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
      <span>
        {t("detail.cached")}: {formatNumber(turn.cached_input_tokens)}
      </span>
      <span>Developer: {formatNumber(developerCount)}</span>
      <span>User: {formatNumber(userCount)}</span>
      <span>Tool: {formatNumber(toolCount)}</span>
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
  const [requestSort, setRequestSort] = useState<RequestSortState>({ key: "time", direction: "asc" });
  const orderedRequests = useMemo(
    () => [...requests].sort((a, b) => compareRequests(a, b, requestSort)),
    [requests, requestSort],
  );
  const requestSortSummary = t("sessions.sortedBy", {
    label: requestSortLabels[requestSort.key],
    direction: requestSort.direction === "desc" ? t("sessions.desc") : t("sessions.asc"),
  });

  const setRequestSortKey = (key: RequestSortKey) => {
    setRequestSort((current) => ({
      key,
      direction: current.key === key && current.direction === "desc" ? "asc" : "desc",
    }));
  };

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
            <p>{requestSortSummary}</p>
          </div>
          <span className="panel-count">{requests.length}</span>
        </div>

        {!isLoading && requests.length === 0 ? (
          <div className="empty-state">{t("detail.noModelRequests")}</div>
        ) : (
          <>
            <div className="request-sort-bar" aria-label="Sort model requests">
              {(Object.keys(requestSortLabels) as RequestSortKey[]).map((key) => {
                const isActive = requestSort.key === key;
                return (
                  <button
                    type="button"
                    className={`request-sort-button${isActive ? " request-sort-button-active" : ""}`}
                    onClick={() => setRequestSortKey(key)}
                    key={key}
                  >
                    <span>{requestSortLabels[key]}</span>
                    <span>{isActive ? (requestSort.direction === "desc" ? "v" : "^") : "-"}</span>
                  </button>
                );
              })}
            </div>
            <div className="request-list">
              {orderedRequests.map((request) => (
                <details className="request-item" key={request.request_index} open={request.request_index === 1}>
                  <summary>
                    <div className="request-title">
                      <strong>{t("detail.requestLabel", { index: request.request_index })}</strong>
                      <time title={request.turn.timestamp}>{formatTimestamp(request.turn.timestamp)}</time>
                    </div>
                    <RequestMetrics request={request} t={t} />
                  </summary>
                  <MessageList messages={request.messages} t={t} />
                </details>
              ))}
            </div>
          </>
        )}
      </section>
    </section>
  );
}

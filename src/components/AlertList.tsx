import type { AlertItem, AlertKind, AlertLevel } from "../types";
import type { Translator } from "../i18n";

interface AlertListProps {
  alerts: AlertItem[];
  t: Translator;
}

const levelKeys: Record<AlertLevel, Parameters<Translator>[0]> = {
  Warning: "alerts.level.Warning",
  Critical: "alerts.level.Critical",
};

const kindKeys: Record<AlertKind, Parameters<Translator>[0]> = {
  LargeToolOutput: "alerts.kind.LargeToolOutput",
  HighSessionUsage: "alerts.kind.HighSessionUsage",
  HighHourlyUsage: "alerts.kind.HighHourlyUsage",
};

function formatTimestamp(timestamp: string) {
  const date = new Date(timestamp);
  if (Number.isNaN(date.getTime())) {
    return timestamp;
  }

  return date.toLocaleString();
}

function alertKey(alert: AlertItem) {
  return [alert.level, alert.kind, alert.timestamp, alert.session_id ?? "", alert.message].join("|");
}

export function AlertList({ alerts, t }: AlertListProps) {
  return (
    <section className="panel alerts-panel">
      <div className="panel-header">
        <div>
          <h2>{t("alerts.title")}</h2>
          <p>{t("alerts.subtitle")}</p>
        </div>
        <span className="panel-count">{alerts.length}</span>
      </div>

      {alerts.length === 0 ? (
        <div className="empty-state">{t("alerts.none")}</div>
      ) : (
        <ul className="alert-list">
          {alerts.map((alert) => (
            <li key={alertKey(alert)} className={`alert-item alert-${alert.level.toLowerCase()}`}>
              <div className="alert-item-header">
                <span className="alert-level">{t(levelKeys[alert.level])}</span>
                <span>{t(kindKeys[alert.kind])}</span>
              </div>
              <p>{alert.message}</p>
              <div className="alert-meta">
                <time>{formatTimestamp(alert.timestamp)}</time>
                {alert.session_id ? <span title={alert.session_id}>{alert.session_id}</span> : null}
              </div>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}

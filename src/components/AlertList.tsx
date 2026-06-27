import type { AlertItem, AlertKind, AlertLevel } from "../types";

interface AlertListProps {
  alerts: AlertItem[];
}

const levelLabels: Record<AlertLevel, string> = {
  Warning: "Warning",
  Critical: "Critical",
};

const kindLabels: Record<AlertKind, string> = {
  LargeToolOutput: "Large tool output",
  HighSessionUsage: "High session usage",
  HighHourlyUsage: "High hourly usage",
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

export function AlertList({ alerts }: AlertListProps) {
  return (
    <section className="panel alerts-panel">
      <div className="panel-header">
        <div>
          <h2>Alerts</h2>
          <p>Usage thresholds</p>
        </div>
        <span className="panel-count">{alerts.length}</span>
      </div>

      {alerts.length === 0 ? (
        <div className="empty-state">No active alerts.</div>
      ) : (
        <ul className="alert-list">
          {alerts.map((alert) => (
            <li key={alertKey(alert)} className={`alert-item alert-${alert.level.toLowerCase()}`}>
              <div className="alert-item-header">
                <span className="alert-level">{levelLabels[alert.level]}</span>
                <span>{kindLabels[alert.kind]}</span>
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

interface AlertListProps {
  alerts?: string[];
}

export function AlertList({ alerts = [] }: AlertListProps) {
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
            <li key={alert}>{alert}</li>
          ))}
        </ul>
      )}
    </section>
  );
}

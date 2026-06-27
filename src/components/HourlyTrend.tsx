import type { TimeBucket } from "../types";

interface HourlyTrendProps {
  buckets: TimeBucket[];
}

const tokenFormat = new Intl.NumberFormat("en", { notation: "compact", maximumFractionDigits: 1 });

function formatHour(bucket: string): string {
  const date = new Date(bucket);
  if (Number.isNaN(date.getTime())) {
    return bucket.slice(11, 16) || bucket;
  }
  return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
}

export function HourlyTrend({ buckets }: HourlyTrendProps) {
  const visibleBuckets = [...buckets]
    .sort((a, b) => a.bucket.localeCompare(b.bucket))
    .slice(-24);
  const maxTokens = Math.max(1, ...visibleBuckets.map((bucket) => bucket.total_tokens));

  return (
    <section className="panel trend-panel">
      <div className="panel-header">
        <div>
          <h2>Last 24 Hours</h2>
          <p>Hourly token volume</p>
        </div>
      </div>

      {visibleBuckets.length === 0 ? (
        <div className="empty-state">No hourly data yet.</div>
      ) : (
        <div className="trend-chart" aria-label="Hourly token totals">
          {visibleBuckets.map((bucket) => {
            const height = Math.max(4, Math.round((bucket.total_tokens / maxTokens) * 100));
            return (
              <div className="trend-column" key={bucket.bucket}>
                <div className="trend-value">{tokenFormat.format(bucket.total_tokens)}</div>
                <div
                  className="trend-bar"
                  style={{ height: `${height}%` }}
                  title={`${formatHour(bucket.bucket)}: ${bucket.total_tokens.toLocaleString()} tokens`}
                />
                <div className="trend-label">{formatHour(bucket.bucket)}</div>
              </div>
            );
          })}
        </div>
      )}
    </section>
  );
}

import { useState } from "react";
import type { Translator } from "../i18n";
import type { TimeBucket } from "../types";

interface HourlyTrendProps {
  hourlyBuckets: TimeBucket[];
  dailyBuckets: TimeBucket[];
  t: Translator;
}

type TrendRange = "hour" | "day" | "week";

interface TrendBucket {
  bucket: string;
  total_tokens: number;
  label: string;
  title: string;
}

const tokenFormat = new Intl.NumberFormat("en", { notation: "compact", maximumFractionDigits: 1 });

function rangeOptions(t: Translator): Array<{ value: TrendRange; label: string; title: string; subtitle: string }> {
  return [
    { value: "week", label: t("trend.week"), title: t("trend.lastWeeks"), subtitle: t("trend.weeklyVolume") },
    { value: "day", label: t("trend.day"), title: t("trend.lastDays"), subtitle: t("trend.dailyVolume") },
    { value: "hour", label: t("trend.hour"), title: t("trend.lastHours"), subtitle: t("trend.hourlyVolume") },
  ];
}

function formatHour(bucket: string): string {
  const date = new Date(bucket);
  if (Number.isNaN(date.getTime())) {
    return bucket.slice(11, 16) || bucket;
  }
  return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
}

function formatDay(bucket: string): string {
  const date = new Date(`${bucket}T00:00:00`);
  if (Number.isNaN(date.getTime())) {
    return bucket.slice(5) || bucket;
  }
  return date.toLocaleDateString([], { month: "short", day: "numeric" });
}

function formatDateKey(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

function weekKey(date: Date): string {
  const weekStart = new Date(date);
  weekStart.setHours(0, 0, 0, 0);
  weekStart.setDate(weekStart.getDate() - weekStart.getDay());
  return formatDateKey(weekStart);
}

function weeklyBuckets(dailyBuckets: TimeBucket[], t: Translator): TrendBucket[] {
  const totals = new Map<string, number>();

  for (const bucket of dailyBuckets) {
    const date = new Date(`${bucket.bucket}T00:00:00`);
    if (Number.isNaN(date.getTime())) {
      continue;
    }

    const key = weekKey(date);
    totals.set(key, (totals.get(key) ?? 0) + bucket.total_tokens);
  }

  return Array.from(totals.entries())
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([bucket, total_tokens]) => ({
      bucket,
      total_tokens,
      label: formatDay(bucket),
      title: t("trend.weekOf", { day: formatDay(bucket), tokens: total_tokens.toLocaleString() }),
    }));
}

function visibleTrendBuckets(
  range: TrendRange,
  hourlyBuckets: TimeBucket[],
  dailyBuckets: TimeBucket[],
  t: Translator,
): TrendBucket[] {
  if (range === "hour") {
    return [...hourlyBuckets]
      .sort((a, b) => a.bucket.localeCompare(b.bucket))
      .slice(-24)
      .map((bucket) => ({
        bucket: bucket.bucket,
        total_tokens: bucket.total_tokens,
        label: formatHour(bucket.bucket),
        title: `${formatHour(bucket.bucket)}: ${bucket.total_tokens.toLocaleString()} ${t("unit.tokens")}`,
      }));
  }

  if (range === "day") {
    return [...dailyBuckets]
      .sort((a, b) => a.bucket.localeCompare(b.bucket))
      .slice(-30)
      .map((bucket) => ({
        bucket: bucket.bucket,
        total_tokens: bucket.total_tokens,
        label: formatDay(bucket.bucket),
        title: `${formatDay(bucket.bucket)}: ${bucket.total_tokens.toLocaleString()} ${t("unit.tokens")}`,
      }));
  }

  return weeklyBuckets(dailyBuckets, t).slice(-12);
}

export function HourlyTrend({ hourlyBuckets, dailyBuckets, t }: HourlyTrendProps) {
  const [range, setRange] = useState<TrendRange>("hour");
  const options = rangeOptions(t);
  const selectedRange = options.find((option) => option.value === range) ?? options[2];
  const visibleBuckets = visibleTrendBuckets(range, hourlyBuckets, dailyBuckets, t);
  const maxTokens = Math.max(1, ...visibleBuckets.map((bucket) => bucket.total_tokens));

  return (
    <section className="panel trend-panel">
      <div className="panel-header">
        <div>
          <h2>{selectedRange.title}</h2>
          <p>{selectedRange.subtitle}</p>
        </div>
        <div className="trend-range-switch" aria-label={t("trend.rangeAria")}>
          {options.map((option) => (
            <button
              type="button"
              key={option.value}
              className={option.value === range ? "trend-range-active" : ""}
              onClick={() => setRange(option.value)}
            >
              {option.label}
            </button>
          ))}
        </div>
      </div>

      {visibleBuckets.length === 0 ? (
        <div className="empty-state">{t("session.emptyForRange", { range: selectedRange.label })}</div>
      ) : (
        <div
          className="trend-chart"
          style={{ gridTemplateColumns: `repeat(${visibleBuckets.length}, minmax(16px, 1fr))` }}
          aria-label={t("trend.tokenTotals", { range: selectedRange.label })}
        >
          {visibleBuckets.map((bucket) => {
            const height = Math.max(4, Math.round((bucket.total_tokens / maxTokens) * 100));
            return (
              <div className="trend-column" key={bucket.bucket}>
                <div className="trend-value">{tokenFormat.format(bucket.total_tokens)}</div>
                <div className="trend-bar" style={{ height: `${height}%` }} title={bucket.title} />
                <div className="trend-label">{bucket.label}</div>
              </div>
            );
          })}
        </div>
      )}
    </section>
  );
}

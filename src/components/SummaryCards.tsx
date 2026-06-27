import type { DashboardSummary } from "../types";

interface SummaryCardsProps {
  summary: DashboardSummary | null;
}

const emptySummary: DashboardSummary = {
  today_total_tokens: 0,
  last_hour_tokens: 0,
  last_five_hours_tokens: 0,
  active_session_count: 0,
  input_tokens: 0,
  output_tokens: 0,
};

const numberFormat = new Intl.NumberFormat("en", { notation: "compact", maximumFractionDigits: 1 });

function formatTokens(value: number): string {
  return numberFormat.format(value);
}

export function SummaryCards({ summary }: SummaryCardsProps) {
  const data = summary ?? emptySummary;
  const cards = [
    { label: "Today", value: formatTokens(data.today_total_tokens), meta: "tokens" },
    { label: "Last hour", value: formatTokens(data.last_hour_tokens), meta: "tokens" },
    { label: "Last 5h", value: formatTokens(data.last_five_hours_tokens), meta: "tokens" },
    { label: "Active", value: data.active_session_count.toString(), meta: "sessions" },
    { label: "Input", value: formatTokens(data.input_tokens), meta: "tokens" },
    { label: "Output", value: formatTokens(data.output_tokens), meta: "tokens" },
  ];

  return (
    <section className="summary-grid" aria-label="Token summary">
      {cards.map((card) => (
        <article className="metric-card" key={card.label}>
          <div className="metric-label">{card.label}</div>
          <div className="metric-value">{card.value}</div>
          <div className="metric-meta">{card.meta}</div>
        </article>
      ))}
    </section>
  );
}

import type { DashboardSummary } from "../types";
import type { Translator } from "../i18n";

interface SummaryCardsProps {
  summary: DashboardSummary | null;
  t: Translator;
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

export function SummaryCards({ summary, t }: SummaryCardsProps) {
  const data = summary ?? emptySummary;
  const cards = [
    { label: t("summary.today"), value: formatTokens(data.today_total_tokens), meta: t("unit.tokens") },
    { label: t("summary.lastHour"), value: formatTokens(data.last_hour_tokens), meta: t("unit.tokens") },
    { label: t("summary.lastFiveHours"), value: formatTokens(data.last_five_hours_tokens), meta: t("unit.tokens") },
    { label: t("summary.active"), value: data.active_session_count.toString(), meta: t("unit.sessions") },
    { label: t("summary.input"), value: formatTokens(data.input_tokens), meta: t("unit.tokens") },
    { label: t("summary.output"), value: formatTokens(data.output_tokens), meta: t("unit.tokens") },
  ];

  return (
    <section className="summary-grid" aria-label={t("summary.aria")}>
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

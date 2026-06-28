export type Language = "en" | "zh";

export type TranslationKey =
  | "app.title"
  | "dashboard.subtitle"
  | "status.live"
  | "status.disconnected"
  | "language.label"
  | "language.en"
  | "language.zh"
  | "dashboard.loadError"
  | "dashboard.loading"
  | "summary.aria"
  | "summary.today"
  | "summary.lastHour"
  | "summary.lastFiveHours"
  | "summary.active"
  | "summary.input"
  | "summary.output"
  | "unit.tokens"
  | "unit.sessions"
  | "trend.rangeAria"
  | "trend.week"
  | "trend.day"
  | "trend.hour"
  | "trend.lastWeeks"
  | "trend.lastDays"
  | "trend.lastHours"
  | "trend.weeklyVolume"
  | "trend.dailyVolume"
  | "trend.hourlyVolume"
  | "trend.weekOf"
  | "trend.tokenTotals"
  | "session.emptyForRange"
  | "sessions.title"
  | "sessions.none"
  | "sessions.sortedBy"
  | "sessions.desc"
  | "sessions.asc"
  | "sessions.sortBy"
  | "sessions.open"
  | "sessions.pageOf"
  | "sessions.prev"
  | "sessions.next"
  | "columns.session"
  | "columns.total"
  | "columns.input"
  | "columns.output"
  | "columns.tools"
  | "columns.toolBytes"
  | "columns.lastSeen"
  | "detail.back"
  | "detail.cwd"
  | "detail.sessionId"
  | "detail.log"
  | "detail.cachedInput"
  | "detail.reasoning"
  | "detail.loadError"
  | "detail.loading"
  | "detail.modelRequests"
  | "detail.modelRequestsSubtitle"
  | "detail.noModelRequests"
  | "detail.requestLabel"
  | "detail.messagesCount"
  | "detail.noRequestMessages"
  | "detail.inputs"
  | "detail.inputsSubtitle"
  | "detail.noInputs"
  | "detail.turns"
  | "detail.turnsSubtitle"
  | "detail.noTurns"
  | "detail.time"
  | "detail.cached"
  | "role.unknown"
  | "alerts.title"
  | "alerts.subtitle"
  | "alerts.thresholdsAria"
  | "alerts.threshold.session"
  | "alerts.threshold.sessionValue"
  | "alerts.threshold.hourly"
  | "alerts.threshold.hourlyValue"
  | "alerts.threshold.toolOutput"
  | "alerts.threshold.toolOutputValue"
  | "alerts.none"
  | "alerts.level.Warning"
  | "alerts.level.Critical"
  | "alerts.kind.LargeToolOutput"
  | "alerts.kind.HighSessionUsage"
  | "alerts.kind.HighHourlyUsage";

type Params = Record<string, string | number>;
export type Translator = (key: TranslationKey, params?: Params) => string;

const dictionaries: Record<Language, Record<TranslationKey, string>> = {
  en: {
    "app.title": "Codex Token Monitor",
    "dashboard.subtitle": "Live session usage dashboard",
    "status.live": "Live",
    "status.disconnected": "Disconnected",
    "language.label": "Language",
    "language.en": "English",
    "language.zh": "中文",
    "dashboard.loadError": "Unable to load dashboard data: {error}",
    "dashboard.loading": "Loading dashboard data...",
    "summary.aria": "Token summary",
    "summary.today": "Today",
    "summary.lastHour": "Last hour",
    "summary.lastFiveHours": "Last 5h",
    "summary.active": "Active",
    "summary.input": "Input",
    "summary.output": "Output",
    "unit.tokens": "tokens",
    "unit.sessions": "sessions",
    "trend.rangeAria": "Token volume range",
    "trend.week": "Week",
    "trend.day": "Day",
    "trend.hour": "Hour",
    "trend.lastWeeks": "Last 12 Weeks",
    "trend.lastDays": "Last 30 Days",
    "trend.lastHours": "Last 24 Hours",
    "trend.weeklyVolume": "Weekly token volume",
    "trend.dailyVolume": "Daily token volume",
    "trend.hourlyVolume": "Hourly token volume",
    "trend.weekOf": "Week of {day}: {tokens} tokens",
    "trend.tokenTotals": "{range} token totals",
    "session.emptyForRange": "No {range} data yet.",
    "sessions.title": "Sessions",
    "sessions.none": "No sessions yet.",
    "sessions.sortedBy": "Sorted by {label} {direction}",
    "sessions.desc": "descending",
    "sessions.asc": "ascending",
    "sessions.sortBy": "Sort by {label}",
    "sessions.open": "Open session {title}",
    "sessions.pageOf": "{start}-{end} of {total}",
    "sessions.prev": "Prev",
    "sessions.next": "Next",
    "columns.session": "Session",
    "columns.total": "Total",
    "columns.input": "Input",
    "columns.output": "Output",
    "columns.tools": "Tools",
    "columns.toolBytes": "Tool KB",
    "columns.lastSeen": "Last Seen",
    "detail.back": "Back",
    "detail.cwd": "CWD",
    "detail.sessionId": "Session ID",
    "detail.log": "Log",
    "detail.cachedInput": "Cached Input",
    "detail.reasoning": "Reasoning",
    "detail.loadError": "Unable to load session detail: {error}",
    "detail.loading": "Loading session detail...",
    "detail.modelRequests": "Model Requests",
    "detail.modelRequestsSubtitle": "Requests grouped by recorded model calls",
    "detail.noModelRequests": "No model request detail found for this session.",
    "detail.requestLabel": "Request #{index}",
    "detail.messagesCount": "{count} messages",
    "detail.noRequestMessages": "No captured messages for this request.",
    "detail.inputs": "Inputs",
    "detail.inputsSubtitle": "Messages grouped by recorded role",
    "detail.noInputs": "No message inputs found for this session.",
    "detail.turns": "Token Turns",
    "detail.turnsSubtitle": "Per-turn usage with compact units",
    "detail.noTurns": "No token detail found for this session.",
    "detail.time": "Time",
    "detail.cached": "Cached",
    "role.unknown": "unknown",
    "alerts.title": "Alerts",
    "alerts.subtitle": "Usage thresholds",
    "alerts.thresholdsAria": "Alert usage thresholds",
    "alerts.threshold.session": "Session usage",
    "alerts.threshold.sessionValue": "3M / 10M tokens",
    "alerts.threshold.hourly": "Hourly usage",
    "alerts.threshold.hourlyValue": "1M / 3M tokens",
    "alerts.threshold.toolOutput": "Tool output",
    "alerts.threshold.toolOutputValue": "50 KB / 200 KB",
    "alerts.none": "No active alerts.",
    "alerts.level.Warning": "Warning",
    "alerts.level.Critical": "Critical",
    "alerts.kind.LargeToolOutput": "Large tool output",
    "alerts.kind.HighSessionUsage": "High session usage",
    "alerts.kind.HighHourlyUsage": "High hourly usage",
  },
  zh: {
    "app.title": "Codex Token 监控",
    "dashboard.subtitle": "实时会话用量看板",
    "status.live": "实时",
    "status.disconnected": "已断开",
    "language.label": "语言",
    "language.en": "English",
    "language.zh": "中文",
    "dashboard.loadError": "无法加载看板数据：{error}",
    "dashboard.loading": "正在加载看板数据...",
    "summary.aria": "Token 汇总",
    "summary.today": "今日",
    "summary.lastHour": "最近 1 小时",
    "summary.lastFiveHours": "最近 5 小时",
    "summary.active": "活跃",
    "summary.input": "输入",
    "summary.output": "输出",
    "unit.tokens": "tokens",
    "unit.sessions": "会话",
    "trend.rangeAria": "Token 用量范围",
    "trend.week": "周",
    "trend.day": "日",
    "trend.hour": "小时",
    "trend.lastWeeks": "最近 12 周",
    "trend.lastDays": "最近 30 天",
    "trend.lastHours": "最近 24 小时",
    "trend.weeklyVolume": "每周 Token 用量",
    "trend.dailyVolume": "每日 Token 用量",
    "trend.hourlyVolume": "每小时 Token 用量",
    "trend.weekOf": "{day} 所在周：{tokens} tokens",
    "trend.tokenTotals": "{range} Token 总量",
    "session.emptyForRange": "暂无 {range} 数据。",
    "sessions.title": "会话",
    "sessions.none": "暂无会话。",
    "sessions.sortedBy": "按{label}{direction}排序",
    "sessions.desc": "降序",
    "sessions.asc": "升序",
    "sessions.sortBy": "按{label}排序",
    "sessions.open": "打开会话 {title}",
    "sessions.pageOf": "{start}-{end}，共 {total}",
    "sessions.prev": "上一页",
    "sessions.next": "下一页",
    "columns.session": "会话",
    "columns.total": "总量",
    "columns.input": "输入",
    "columns.output": "输出",
    "columns.tools": "工具",
    "columns.toolBytes": "工具 KB",
    "columns.lastSeen": "最近出现",
    "detail.back": "返回",
    "detail.cwd": "工作目录",
    "detail.sessionId": "会话 ID",
    "detail.log": "日志",
    "detail.cachedInput": "缓存输入",
    "detail.reasoning": "推理",
    "detail.loadError": "无法加载会话详情：{error}",
    "detail.loading": "正在加载会话详情...",
    "detail.modelRequests": "大模型请求",
    "detail.modelRequestsSubtitle": "按记录的大模型调用分组",
    "detail.noModelRequests": "此会话暂无大模型请求明细。",
    "detail.requestLabel": "请求 #{index}",
    "detail.messagesCount": "{count} 条消息",
    "detail.noRequestMessages": "此请求暂无捕获消息。",
    "detail.inputs": "输入内容",
    "detail.inputsSubtitle": "按记录的角色分组显示消息",
    "detail.noInputs": "此会话暂无消息输入。",
    "detail.turns": "Token 轮次",
    "detail.turnsSubtitle": "每轮用量，使用紧凑单位",
    "detail.noTurns": "此会话暂无 Token 明细。",
    "detail.time": "时间",
    "detail.cached": "缓存",
    "role.unknown": "未知",
    "alerts.title": "告警",
    "alerts.subtitle": "用量阈值",
    "alerts.thresholdsAria": "告警用量阈值",
    "alerts.threshold.session": "会话用量",
    "alerts.threshold.sessionValue": "3M / 10M tokens",
    "alerts.threshold.hourly": "小时用量",
    "alerts.threshold.hourlyValue": "1M / 3M tokens",
    "alerts.threshold.toolOutput": "工具输出",
    "alerts.threshold.toolOutputValue": "50 KB / 200 KB",
    "alerts.none": "暂无活跃告警。",
    "alerts.level.Warning": "警告",
    "alerts.level.Critical": "严重",
    "alerts.kind.LargeToolOutput": "工具输出过大",
    "alerts.kind.HighSessionUsage": "会话用量过高",
    "alerts.kind.HighHourlyUsage": "小时用量过高",
  },
};

export function resolveLanguage(languageTag: string | null | undefined): Language {
  const normalized = (languageTag ?? "").toLowerCase();
  if (normalized.startsWith("zh")) {
    return "zh";
  }
  if (normalized.startsWith("en")) {
    return "en";
  }
  return "en";
}

export function createTranslator(language: Language): Translator {
  const dictionary = dictionaries[language];

  return (key, params = {}) => {
    const template = dictionary[key] ?? dictionaries.en[key] ?? key;
    return template.replace(/\{(\w+)\}/g, (match, name) => String(params[name] ?? match));
  };
}

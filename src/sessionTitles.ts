import type { MessageDetail, SessionSummary } from "./types";

const TITLE_MAX_LENGTH = 64;

export function compactSessionId(sessionId: string): string {
  if (sessionId.length <= 12) {
    return sessionId;
  }
  return `${sessionId.slice(0, 8)}...${sessionId.slice(-4)}`;
}

export function sessionDisplayTitle(session: SessionSummary, messages: MessageDetail[] = []): string {
  const storedName = normalizeTitle(session.session_name);
  if (storedName) {
    return storedName;
  }

  const contentTitle = normalizeTitle(messages.find((message) => message.role.toLowerCase() === "user")?.content);
  if (contentTitle) {
    return trimTitle(contentTitle);
  }

  return compactSessionId(session.session_id);
}

export function sessionSubtitle(session: SessionSummary): string {
  return session.cwd || session.path || "-";
}

function normalizeTitle(value: string | undefined): string {
  return (value ?? "").replace(/\s+/g, " ").trim();
}

function trimTitle(value: string): string {
  if (value.length <= TITLE_MAX_LENGTH) {
    return value;
  }
  return `${value.slice(0, TITLE_MAX_LENGTH - 1)}...`;
}

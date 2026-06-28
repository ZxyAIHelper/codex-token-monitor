import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { sessionDisplayTitle } from "../src/sessionTitles.js";
import type { MessageDetail, SessionSummary } from "../src/types.js";

const baseSession: SessionSummary = {
  session_id: "019f08f1-c13e-7ea1-b57d-8ba3bc9d4186",
  session_name: "",
  cwd: "",
  path: "C:/tmp/rollout.jsonl",
  total_tokens: 0,
  input_tokens: 0,
  cached_input_tokens: 0,
  output_tokens: 0,
  reasoning_output_tokens: 0,
  tool_calls: 0,
  tool_output_bytes: 0,
  last_seen_at: "",
};

describe("session titles", () => {
  it("prefers stored session names", () => {
    assert.equal(sessionDisplayTitle({ ...baseSession, session_name: "Token 监控" }, []), "Token 监控");
  });

  it("derives a compact title from the first user message", () => {
    const messages: MessageDetail[] = [
      { timestamp: "", role: "developer", content: "Use concise output." },
      { timestamp: "", role: "user", content: "增加多语言系统，所有 app 的操作界面支持多语言" },
    ];

    assert.equal(sessionDisplayTitle(baseSession, messages), "增加多语言系统，所有 app 的操作界面支持多语言");
  });

  it("falls back to a compact session id", () => {
    assert.equal(sessionDisplayTitle(baseSession, []), "019f08f1...4186");
  });
});

import assert from "node:assert/strict";
import { describe, it } from "node:test";

import { createTranslator, resolveLanguage } from "../src/i18n.js";

describe("i18n", () => {
  it("resolves supported language tags with fallback", () => {
    assert.equal(resolveLanguage("zh-CN"), "zh");
    assert.equal(resolveLanguage("en-US"), "en");
    assert.equal(resolveLanguage("fr-FR"), "en");
  });

  it("translates labels and interpolates variables", () => {
    const t = createTranslator("zh");

    assert.equal(t("dashboard.subtitle"), "实时会话用量看板");
    assert.equal(t("session.emptyForRange", { range: "hour" }), "暂无 hour 数据。");
  });
});

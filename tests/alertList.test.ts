import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";

import { AlertList } from "../src/components/AlertList.js";
import { createTranslator } from "../src/i18n.js";

describe("AlertList", () => {
  it("renders usage thresholds as a list", () => {
    const html = renderToStaticMarkup(
      createElement(AlertList, { alerts: [], t: createTranslator("en") }),
    );

    assert.match(html, /<ul class="threshold-list"/);
    assert.match(html, /Session usage/);
    assert.match(html, /3M \/ 10M tokens/);
    assert.match(html, /Hourly usage/);
    assert.match(html, /1M \/ 3M tokens/);
    assert.match(html, /Tool output/);
    assert.match(html, /50 KB \/ 200 KB/);
  });
});

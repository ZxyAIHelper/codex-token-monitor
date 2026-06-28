import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { calculateDockPlacement } from "../src/statusIslandDock.js";

const workArea = { x: 0, y: 0, width: 1920, height: 1080 };
const horizontalSize = { width: 240, height: 22 };
const sideSize = { width: 40, height: 112 };

describe("status island docking", () => {
  it("snaps near the top edge at the dragged horizontal position", () => {
    const placement = calculateDockPlacement({
      position: { x: 900, y: 12 },
      currentSize: horizontalSize,
      workArea,
      horizontalSize,
      sideSize,
    });

    assert.deepEqual(placement, {
      edge: "top",
      position: { x: 900, y: 0 },
      size: horizontalSize,
    });
  });

  it("prioritizes the top notch when dragged into a top corner without overflowing", () => {
    const placement = calculateDockPlacement({
      position: { x: 0, y: 0 },
      currentSize: horizontalSize,
      workArea,
      horizontalSize,
      sideSize,
    });

    assert.deepEqual(placement, {
      edge: "top",
      position: { x: 8, y: 0 },
      size: horizontalSize,
    });
  });

  it("snaps near the left edge as a vertical side island", () => {
    const placement = calculateDockPlacement({
      position: { x: 20, y: 380 },
      currentSize: horizontalSize,
      workArea,
      horizontalSize,
      sideSize,
    });

    assert.deepEqual(placement, {
      edge: "left",
      position: { x: 8, y: 380 },
      size: sideSize,
    });
  });

  it("snaps near the right edge without overflowing the screen", () => {
    const placement = calculateDockPlacement({
      position: { x: 1680, y: 980 },
      currentSize: horizontalSize,
      workArea,
      horizontalSize,
      sideSize,
    });

    assert.deepEqual(placement, {
      edge: "right",
      position: { x: 1872, y: 960 },
      size: sideSize,
    });
  });
});

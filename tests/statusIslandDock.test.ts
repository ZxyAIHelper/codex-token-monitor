import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { calculateDockPlacement } from "../src/statusIslandDock.js";

const workArea = { x: 0, y: 0, width: 1920, height: 1080 };
const horizontalSize = { width: 360, height: 42 };
const sideSize = { width: 52, height: 150 };

describe("status island docking", () => {
  it("snaps near the top edge and centers inside the work area", () => {
    const placement = calculateDockPlacement({
      position: { x: 900, y: 12 },
      currentSize: horizontalSize,
      workArea,
      horizontalSize,
      sideSize,
    });

    assert.deepEqual(placement, {
      edge: "top",
      position: { x: 780, y: 8 },
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
      position: { x: 1500, y: 980 },
      currentSize: horizontalSize,
      workArea,
      horizontalSize,
      sideSize,
    });

    assert.deepEqual(placement, {
      edge: "right",
      position: { x: 1860, y: 922 },
      size: sideSize,
    });
  });
});

import { describe, expect, it } from "vitest";

import { findNextFocus, type FocusTarget } from "./index";

const targets: FocusTarget[] = [
  { id: "top-left", rect: { x: 0, y: 0, width: 100, height: 50 } },
  { id: "top-right", rect: { x: 140, y: 0, width: 100, height: 50 } },
  { id: "bottom-left", rect: { x: 0, y: 100, width: 100, height: 50 } },
  { id: "bottom-right", rect: { x: 140, y: 100, width: 100, height: 50 } },
];

describe("findNextFocus", () => {
  it("chooses the nearest aligned target", () => {
    expect(findNextFocus("top-left", targets, "right")).toBe("top-right");
    expect(findNextFocus("top-left", targets, "down")).toBe("bottom-left");
  });

  it("does not move outside the available targets", () => {
    expect(findNextFocus("top-left", targets, "left")).toBeNull();
    expect(findNextFocus("top-left", targets, "up")).toBeNull();
  });

  it("skips disabled targets", () => {
    const withDisabled = targets.map((target) =>
      target.id === "top-right" ? { ...target, disabled: true } : target,
    );

    expect(findNextFocus("top-left", withDisabled, "right")).toBe(
      "bottom-right",
    );
  });

  it("recovers from an unknown current id", () => {
    expect(findNextFocus("missing", targets, "right")).toBe("top-left");
  });
});

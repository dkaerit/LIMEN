import { describe, expect, it } from "vitest";

import { calculateVirtualGrid, columnsForWidth } from "./virtualization";

describe("virtualized library grid", () => {
  it("selects a readable column count for each target width", () => {
    expect(columnsForWidth(1600)).toBe(5);
    expect(columnsForWidth(1000)).toBe(4);
    expect(columnsForWidth(650)).toBe(3);
    expect(columnsForWidth(390)).toBe(2);
  });

  it("renders only visible and overscan rows for the 160-game library", () => {
    const layout = calculateVirtualGrid({
      itemCount: 160,
      width: 1000,
      scrollOffset: 0,
      viewportHeight: 720,
    });

    expect(layout.columns).toBe(4);
    expect(layout.rowCount).toBe(40);
    expect(layout.visibleRows.length).toBeLessThan(12);
    expect(layout.totalHeight).toBeGreaterThan(6000);
  });

  it("keeps a focused off-screen item mounted so focus can be restored", () => {
    const layout = calculateVirtualGrid({
      itemCount: 160,
      width: 1000,
      scrollOffset: 0,
      viewportHeight: 720,
      pinnedIndex: 159,
    });

    expect(layout.visibleRows).toContain(39);
  });
});

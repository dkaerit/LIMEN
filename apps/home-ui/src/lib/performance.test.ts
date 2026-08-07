import { describe, expect, it } from "vitest";

import { resolveGraphicsProfile, summarizeFrameTimes } from "./performance";

describe("M1 graphics quality and benchmark math", () => {
  it("degrades the automatic profile as rendered pixel cost grows", () => {
    expect(
      resolveGraphicsProfile("auto", {
        width: 1920,
        height: 1080,
        devicePixelRatio: 1,
      }),
    ).toBe("quality");
    expect(
      resolveGraphicsProfile("auto", {
        width: 2560,
        height: 1440,
        devicePixelRatio: 1,
      }),
    ).toBe("balanced");
    expect(
      resolveGraphicsProfile("auto", {
        width: 3840,
        height: 2160,
        devicePixelRatio: 1,
      }),
    ).toBe("performance");
  });

  it("reports frame-time percentiles without hiding slow frames", () => {
    const summary = summarizeFrameTimes([
      ...Array.from({ length: 95 }, () => 16),
      ...Array.from({ length: 5 }, () => 32),
    ]);

    expect(summary.frames).toBe(100);
    expect(summary.averageFps).toBeCloseTo(59.52, 1);
    expect(summary.p95FrameMs).toBe(16);
    expect(summary.slowFramePercent).toBe(5);
  });
});

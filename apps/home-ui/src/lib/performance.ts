export type GraphicsPreference =
  "auto" | "quality" | "balanced" | "performance" | "2d";

export type GraphicsProfile = "quality" | "balanced" | "performance";

export interface ViewportMetrics {
  width: number;
  height: number;
  devicePixelRatio: number;
}

export interface FrameSummary {
  frames: number;
  averageFrameMs: number;
  p95FrameMs: number;
  averageFps: number;
  slowFramePercent: number;
}

export interface HomeBenchmarkResult extends FrameSummary, ViewportMetrics {
  durationMs: number;
  profile: GraphicsProfile | "2d";
  usedHeapMb: number | null;
}

export function resolveGraphicsProfile(
  preference: GraphicsPreference,
  viewport: ViewportMetrics,
): GraphicsProfile | "2d" {
  if (preference !== "auto") return preference;

  const renderedPixels =
    viewport.width *
    viewport.height *
    Math.max(viewport.devicePixelRatio, 1) ** 2;
  if (renderedPixels >= 7_000_000) return "performance";
  if (renderedPixels >= 3_200_000) return "balanced";
  return "quality";
}

export function summarizeFrameTimes(frameTimes: number[]): FrameSummary {
  const valid = frameTimes.filter(
    (frameTime) => Number.isFinite(frameTime) && frameTime > 0,
  );
  if (valid.length === 0) {
    return {
      frames: 0,
      averageFrameMs: 0,
      p95FrameMs: 0,
      averageFps: 0,
      slowFramePercent: 0,
    };
  }

  const sorted = [...valid].sort((left, right) => left - right);
  const averageFrameMs =
    valid.reduce((total, frameTime) => total + frameTime, 0) / valid.length;
  const p95Index = Math.min(
    sorted.length - 1,
    Math.ceil(sorted.length * 0.95) - 1,
  );
  const slowFrames = valid.filter((frameTime) => frameTime > 20).length;

  return {
    frames: valid.length,
    averageFrameMs,
    p95FrameMs: sorted[p95Index] ?? 0,
    averageFps: 1000 / averageFrameMs,
    slowFramePercent: (slowFrames / valid.length) * 100,
  };
}

function readUsedHeapMb(): number | null {
  const memory = (
    performance as Performance & {
      memory?: { usedJSHeapSize?: number };
    }
  ).memory;
  return typeof memory?.usedJSHeapSize === "number"
    ? memory.usedJSHeapSize / 1024 / 1024
    : null;
}

export function runHomeBenchmark(
  durationMs: number,
  profile: GraphicsProfile | "2d",
): Promise<HomeBenchmarkResult> {
  return new Promise((resolve) => {
    const frameTimes: number[] = [];
    const startedAt = performance.now();
    let previousFrame = startedAt;

    const sample = (timestamp: number) => {
      frameTimes.push(timestamp - previousFrame);
      previousFrame = timestamp;

      if (timestamp - startedAt < durationMs) {
        window.requestAnimationFrame(sample);
        return;
      }

      resolve({
        ...summarizeFrameTimes(frameTimes.slice(1)),
        durationMs: timestamp - startedAt,
        profile,
        width: window.innerWidth,
        height: window.innerHeight,
        devicePixelRatio: window.devicePixelRatio,
        usedHeapMb: readUsedHeapMb(),
      });
    };

    window.requestAnimationFrame(sample);
  });
}

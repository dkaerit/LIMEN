export type FocusDirection = "up" | "down" | "left" | "right";

export interface FocusRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface FocusTarget {
  id: string;
  rect: FocusRect;
  disabled?: boolean;
}

interface Point {
  x: number;
  y: number;
}

function center(rect: FocusRect): Point {
  return {
    x: rect.x + rect.width / 2,
    y: rect.y + rect.height / 2,
  };
}

function isInDirection(
  origin: Point,
  candidate: Point,
  direction: FocusDirection,
): boolean {
  switch (direction) {
    case "up":
      return candidate.y < origin.y - 1;
    case "down":
      return candidate.y > origin.y + 1;
    case "left":
      return candidate.x < origin.x - 1;
    case "right":
      return candidate.x > origin.x + 1;
  }
}

function score(
  origin: Point,
  candidate: Point,
  direction: FocusDirection,
): number {
  const deltaX = Math.abs(candidate.x - origin.x);
  const deltaY = Math.abs(candidate.y - origin.y);
  const vertical = direction === "up" || direction === "down";
  const primary = vertical ? deltaY : deltaX;
  const secondary = vertical ? deltaX : deltaY;
  const angularPenalty = (secondary / Math.max(primary, 1)) * 80;

  return primary + secondary * 0.35 + angularPenalty;
}

export function findNextFocus(
  currentId: string,
  targets: readonly FocusTarget[],
  direction: FocusDirection,
): string | null {
  const current = targets.find(
    (target) => target.id === currentId && !target.disabled,
  );

  if (!current) {
    return targets.find((target) => !target.disabled)?.id ?? null;
  }

  const origin = center(current.rect);
  let best: { id: string; score: number } | null = null;

  for (const target of targets) {
    if (target.id === currentId || target.disabled) {
      continue;
    }

    const candidate = center(target.rect);
    if (!isInDirection(origin, candidate, direction)) {
      continue;
    }

    const candidateScore = score(origin, candidate, direction);
    if (!best || candidateScore < best.score) {
      best = { id: target.id, score: candidateScore };
    }
  }

  return best?.id ?? null;
}

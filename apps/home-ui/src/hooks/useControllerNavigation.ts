import { useEffect, useRef, useState } from "react";

import type { FocusDirection } from "@limen/focus-engine";

export type ControllerAction = FocusDirection | "accept" | "back";

const INITIAL_REPEAT_DELAY = 320;
const REPEAT_INTERVAL = 105;
const AXIS_THRESHOLD = 0.58;

function pressed(gamepad: Gamepad, index: number): boolean {
  return gamepad.buttons[index]?.pressed ?? false;
}

function directionFor(gamepad: Gamepad): FocusDirection | null {
  const horizontal = gamepad.axes[0] ?? 0;
  const vertical = gamepad.axes[1] ?? 0;

  if (pressed(gamepad, 12) || vertical < -AXIS_THRESHOLD) return "up";
  if (pressed(gamepad, 13) || vertical > AXIS_THRESHOLD) return "down";
  if (pressed(gamepad, 14) || horizontal < -AXIS_THRESHOLD) return "left";
  if (pressed(gamepad, 15) || horizontal > AXIS_THRESHOLD) return "right";
  return null;
}

export function useControllerNavigation(
  onAction: (action: ControllerAction) => void,
): boolean {
  const callback = useRef(onAction);
  const [connected, setConnected] = useState(false);

  useEffect(() => {
    callback.current = onAction;
  }, [onAction]);

  useEffect(() => {
    let animationFrame = 0;
    let activeDirection: FocusDirection | null = null;
    let nextRepeatAt = 0;
    let acceptWasPressed = false;
    let backWasPressed = false;
    let connectionState = false;

    const updateConnection = (next: boolean) => {
      if (next !== connectionState) {
        connectionState = next;
        setConnected(next);
      }
    };

    const poll = (timestamp: number) => {
      const gamepad =
        [...navigator.getGamepads()].find(
          (candidate) => candidate?.connected,
        ) ?? null;
      updateConnection(Boolean(gamepad));

      if (gamepad) {
        const direction = directionFor(gamepad);
        if (direction && direction !== activeDirection) {
          activeDirection = direction;
          nextRepeatAt = timestamp + INITIAL_REPEAT_DELAY;
          callback.current(direction);
        } else if (direction && timestamp >= nextRepeatAt) {
          nextRepeatAt = timestamp + REPEAT_INTERVAL;
          callback.current(direction);
        } else if (!direction) {
          activeDirection = null;
        }

        const acceptIsPressed = pressed(gamepad, 0);
        const backIsPressed = pressed(gamepad, 1);
        if (acceptIsPressed && !acceptWasPressed) callback.current("accept");
        if (backIsPressed && !backWasPressed) callback.current("back");
        acceptWasPressed = acceptIsPressed;
        backWasPressed = backIsPressed;
      } else {
        activeDirection = null;
        acceptWasPressed = false;
        backWasPressed = false;
      }

      animationFrame = window.requestAnimationFrame(poll);
    };

    animationFrame = window.requestAnimationFrame(poll);
    return () => window.cancelAnimationFrame(animationFrame);
  }, []);

  return connected;
}

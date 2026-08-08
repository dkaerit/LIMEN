import { useEffect, useRef, useState } from "react";

import type { FocusDirection } from "@limen/focus-engine";

export type ControllerAction = FocusDirection | "accept" | "back";

const INITIAL_REPEAT_DELAY = 320;
const REPEAT_INTERVAL = 105;
const AXIS_THRESHOLD = 0.58;

type ControllerSample = Pick<Gamepad, "axes" | "buttons">;

export interface ControllerFrameState {
  activeDirection: FocusDirection | null;
  nextRepeatAt: number;
  acceptWasPressed: boolean;
  backWasPressed: boolean;
}

export const initialControllerFrameState: ControllerFrameState = {
  activeDirection: null,
  nextRepeatAt: 0,
  acceptWasPressed: false,
  backWasPressed: false,
};

function pressed(gamepad: ControllerSample, index: number): boolean {
  return gamepad.buttons[index]?.pressed ?? false;
}

function directionFor(gamepad: ControllerSample): FocusDirection | null {
  const horizontal = gamepad.axes[0] ?? 0;
  const vertical = gamepad.axes[1] ?? 0;

  if (pressed(gamepad, 12) || vertical < -AXIS_THRESHOLD) return "up";
  if (pressed(gamepad, 13) || vertical > AXIS_THRESHOLD) return "down";
  if (pressed(gamepad, 14) || horizontal < -AXIS_THRESHOLD) return "left";
  if (pressed(gamepad, 15) || horizontal > AXIS_THRESHOLD) return "right";
  return null;
}

export function controllerActionsForFrame(
  gamepad: ControllerSample | null,
  timestamp: number,
  state: ControllerFrameState,
): { actions: ControllerAction[]; state: ControllerFrameState } {
  if (!gamepad) {
    return {
      actions: [],
      state: { ...initialControllerFrameState },
    };
  }

  const actions: ControllerAction[] = [];
  const direction = directionFor(gamepad);
  let activeDirection = state.activeDirection;
  let nextRepeatAt = state.nextRepeatAt;

  if (direction && direction !== activeDirection) {
    activeDirection = direction;
    nextRepeatAt = timestamp + INITIAL_REPEAT_DELAY;
    actions.push(direction);
  } else if (direction && timestamp >= nextRepeatAt) {
    nextRepeatAt = timestamp + REPEAT_INTERVAL;
    actions.push(direction);
  } else if (!direction) {
    activeDirection = null;
  }

  const acceptIsPressed = pressed(gamepad, 0);
  const backIsPressed = pressed(gamepad, 1);
  if (acceptIsPressed && !state.acceptWasPressed) actions.push("accept");
  if (backIsPressed && !state.backWasPressed) actions.push("back");

  return {
    actions,
    state: {
      activeDirection,
      nextRepeatAt,
      acceptWasPressed: acceptIsPressed,
      backWasPressed: backIsPressed,
    },
  };
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
    let inputState = { ...initialControllerFrameState };
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
      const frame = controllerActionsForFrame(gamepad, timestamp, inputState);
      inputState = frame.state;
      frame.actions.forEach((action) => callback.current(action));

      animationFrame = window.requestAnimationFrame(poll);
    };

    animationFrame = window.requestAnimationFrame(poll);
    return () => window.cancelAnimationFrame(animationFrame);
  }, []);

  return connected;
}

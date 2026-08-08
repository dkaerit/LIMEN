import { describe, expect, it } from "vitest";

import {
  controllerActionsForFrame,
  initialControllerFrameState,
} from "./useControllerNavigation";

function sample({
  horizontal = 0,
  vertical = 0,
  accept = false,
  back = false,
}: {
  horizontal?: number;
  vertical?: number;
  accept?: boolean;
  back?: boolean;
} = {}) {
  const buttons = Array.from({ length: 16 }, () => ({
    pressed: false,
    touched: false,
    value: 0,
  }));
  buttons[0] = { pressed: accept, touched: accept, value: accept ? 1 : 0 };
  buttons[1] = { pressed: back, touched: back, value: back ? 1 : 0 };
  return { axes: [horizontal, vertical], buttons };
}

describe("temporary controller input backend", () => {
  it("applies a delay and stable cadence to a held direction", () => {
    const first = controllerActionsForFrame(
      sample({ horizontal: 1 }),
      0,
      initialControllerFrameState,
    );
    const beforeDelay = controllerActionsForFrame(
      sample({ horizontal: 1 }),
      250,
      first.state,
    );
    const repeated = controllerActionsForFrame(
      sample({ horizontal: 1 }),
      320,
      beforeDelay.state,
    );

    expect(first.actions).toEqual(["right"]);
    expect(beforeDelay.actions).toEqual([]);
    expect(repeated.actions).toEqual(["right"]);
  });

  it("emits accept and back only on their pressed edge", () => {
    const first = controllerActionsForFrame(
      sample({ accept: true, back: true }),
      0,
      initialControllerFrameState,
    );
    const held = controllerActionsForFrame(
      sample({ accept: true, back: true }),
      16,
      first.state,
    );

    expect(first.actions).toEqual(["accept", "back"]);
    expect(held.actions).toEqual([]);
  });

  it("resets held input after disconnection so reconnection is usable", () => {
    const pressed = controllerActionsForFrame(
      sample({ accept: true }),
      0,
      initialControllerFrameState,
    );
    const disconnected = controllerActionsForFrame(null, 16, pressed.state);
    const reconnected = controllerActionsForFrame(
      sample({ accept: true }),
      32,
      disconnected.state,
    );

    expect(reconnected.actions).toEqual(["accept"]);
  });

  it("stays deterministic across fifteen simulated minutes of held input and reconnects", () => {
    let state = { ...initialControllerFrameState };
    let actionCount = 0;
    let disconnectFrames = 0;

    for (let timestamp = 0; timestamp < 15 * 60 * 1000; timestamp += 16) {
      const isDisconnected = timestamp % 60_000 >= 59_000;
      const horizontal = Math.floor(timestamp / 1000) % 2 === 0 ? 1 : -1;
      const frame = controllerActionsForFrame(
        isDisconnected ? null : sample({ horizontal }),
        timestamp,
        state,
      );
      state = frame.state;
      actionCount += frame.actions.length;
      if (isDisconnected) disconnectFrames += 1;
    }

    expect(actionCount).toBeGreaterThan(6_500);
    expect(disconnectFrames).toBeGreaterThan(800);
    expect(state).toEqual(initialControllerFrameState);
  });
});

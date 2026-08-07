import { describe, expect, it } from "vitest";

import { games } from "./games";

describe("M1 simulated library", () => {
  it("contains the 160 safe placeholder entries required for the stress slice", () => {
    expect(games).toHaveLength(160);
    expect(new Set(games.map((game) => game.id)).size).toBe(160);
  });

  it("keeps the original PS2 vertical-slice title and artwork selected first", () => {
    expect(games[0]).toMatchObject({
      title: "Crystal Voyage",
      platform: "PS2",
      artwork: "/assets/crystal-threshold-v1.png",
    });
  });

  it("uses only the original LIMEN artwork set", () => {
    expect(new Set(games.map((game) => game.artwork))).toEqual(
      new Set([
        "/assets/crystal-threshold-v1.png",
        "/assets/golden-sanctuary-v1.png",
        "/assets/limen-spatial-bg-v1.png",
      ]),
    );
  });
});

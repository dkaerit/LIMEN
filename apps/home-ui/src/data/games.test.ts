import { describe, expect, it } from "vitest";

import { games } from "./games";

describe("M1 simulated library", () => {
  it("contains the 160 safe placeholder entries required for the stress slice", () => {
    expect(games).toHaveLength(160);
    expect(new Set(games.map((game) => game.id)).size).toBe(160);
  });

  it("keeps the PS2 vertical-slice title selected first without bundled artwork", () => {
    expect(games[0]).toMatchObject({
      title: "Final Fantasy X",
      platform: "PS2",
    });
  });
});

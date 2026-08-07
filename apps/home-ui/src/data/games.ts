export type GamePlatform = "PS2" | "PC" | "ARCADE" | "HANDHELD" | "16-BIT";

export interface GameSummary {
  id: string;
  title: string;
  subtitle: string;
  platform: GamePlatform;
  playtime: string;
  progress: number;
  accent: string;
  accentSecondary: string;
  compatibility: "Perfecta" | "Excelente" | "Verificada";
}

const titleStarts = [
  "Astra",
  "Crystal",
  "Neon",
  "Solar",
  "Echo",
  "Velvet",
  "Silent",
  "Azure",
  "Orbital",
  "Midnight",
  "Radiant",
  "Parallel",
  "Lunar",
  "Iron",
  "Golden",
  "Mirage",
];

const titleEnds = [
  "Voyage",
  "Frontier",
  "Signal",
  "Odyssey",
  "Circuit",
  "Horizon",
  "Archive",
  "Garden",
  "Drift",
  "Legacy",
];

const palettes = [
  ["#68c7ff", "#3d5cff"],
  ["#9c72ff", "#3d75ff"],
  ["#48e0c2", "#2587cc"],
  ["#ffb45c", "#9d4cff"],
  ["#ff6f9f", "#4c63ff"],
  ["#7ee46c", "#2d78c8"],
] as const;

const platforms: GamePlatform[] = ["PS2", "PC", "ARCADE", "HANDHELD", "16-BIT"];

export const games: GameSummary[] = Array.from({ length: 160 }, (_, index) => {
  const palette = palettes[index % palettes.length] ?? palettes[0];
  const platform = platforms[index % platforms.length] ?? "PC";
  const titleStart = titleStarts[index % titleStarts.length] ?? "LIMEN";
  const titleEnd =
    titleEnds[Math.floor(index / titleStarts.length) % titleEnds.length] ??
    "Archive";

  return {
    id: `game-${String(index + 1).padStart(3, "0")}`,
    title: index === 0 ? "Final Fantasy X" : `${titleStart} ${titleEnd}`,
    subtitle:
      index === 0
        ? "El umbral de Spira"
        : `Entrada de prueba ${String(index + 1).padStart(3, "0")}`,
    platform: index === 0 ? "PS2" : platform,
    playtime: `${2 + ((index * 7) % 83)} h ${String((index * 13) % 60).padStart(2, "0")} min`,
    progress: index === 0 ? 62 : 8 + ((index * 17) % 88),
    accent: palette[0],
    accentSecondary: palette[1],
    compatibility:
      index % 7 === 0
        ? "Verificada"
        : index % 3 === 0
          ? "Excelente"
          : "Perfecta",
  };
});

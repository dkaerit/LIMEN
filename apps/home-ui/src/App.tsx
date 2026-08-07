import {
  useCallback,
  useEffect,
  useMemo,
  useState,
  type CSSProperties,
} from "react";

import {
  findNextFocus,
  type FocusDirection,
  type FocusTarget,
} from "@limen/focus-engine";
import { AmbientScene } from "@limen/graphics";
import { ControllerHint, FocusButton, PlatformBadge } from "@limen/ui-kit";

import { games, type GameSummary } from "./data/games";
import {
  useControllerNavigation,
  type ControllerAction,
} from "./hooks/useControllerNavigation";

type ViewId = "home" | "library" | "add" | "settings";

interface NavItem {
  id: ViewId;
  label: string;
}

const navigation: NavItem[] = [
  { id: "home", label: "Inicio" },
  { id: "library", label: "Biblioteca" },
  { id: "add", label: "Añadir juegos" },
  { id: "settings", label: "Ajustes" },
];

function focusableElements(): HTMLElement[] {
  return [
    ...document.querySelectorAll<HTMLElement>(
      "[data-focus-id]:not([disabled])",
    ),
  ].filter((element) => element.offsetParent !== null);
}

function focusTargets(): FocusTarget[] {
  return focusableElements().map((element) => {
    const rect = element.getBoundingClientRect();
    return {
      id: element.dataset.focusId ?? "",
      rect: { x: rect.x, y: rect.y, width: rect.width, height: rect.height },
    };
  });
}

function elementForFocusId(focusId: string): HTMLElement | null {
  return (
    focusableElements().find(
      (element) => element.dataset.focusId === focusId,
    ) ?? null
  );
}

function gameColors(game: GameSummary): CSSProperties {
  return {
    "--game-accent": game.accent,
    "--game-accent-secondary": game.accentSecondary,
  } as CSSProperties;
}

interface GameCardProps {
  game: GameSummary;
  focusId: string;
  focusedId: string;
  compact?: boolean;
  onChoose: (game: GameSummary, focusId: string) => void;
}

function GameCard({
  game,
  focusId,
  focusedId,
  compact = false,
  onChoose,
}: GameCardProps) {
  return (
    <FocusButton
      focusId={focusId}
      focused={focusedId === focusId}
      className={`game-card ${compact ? "game-card--compact" : ""}`}
      data-game-id={game.id}
      style={gameColors(game)}
      onClick={() => onChoose(game, focusId)}
      aria-label={`${game.title}, ${game.platform}, progreso ${game.progress}%`}
    >
      <span className="game-card__art" aria-hidden="true">
        <span className="game-card__planet" />
        <span className="game-card__monolith" />
      </span>
      <span className="game-card__shade" />
      <span className="game-card__copy">
        <strong>{game.title}</strong>
        <span>{game.subtitle}</span>
      </span>
      <span className="game-card__footer">
        <PlatformBadge>{game.platform}</PlatformBadge>
        <span className="game-card__progress" aria-hidden="true">
          <span style={{ width: `${game.progress}%` }} />
        </span>
      </span>
    </FocusButton>
  );
}

interface HomeViewProps {
  featured: GameSummary;
  focusedId: string;
  onPlay: () => void;
  onChooseGame: (game: GameSummary, focusId: string) => void;
}

function HomeView({
  featured,
  focusedId,
  onPlay,
  onChooseGame,
}: HomeViewProps) {
  return (
    <div className="home-view">
      <section
        className="hero-panel"
        style={gameColors(featured)}
        aria-labelledby="featured-title"
      >
        <div className="hero-panel__copy">
          <span className="eyebrow">
            Juego destacado · Biblioteca de prueba
          </span>
          <h1 id="featured-title">{featured.title}</h1>
          <p className="hero-panel__subtitle">{featured.subtitle}</p>
          <div className="hero-panel__metadata">
            <PlatformBadge>{featured.platform}</PlatformBadge>
            <span>
              <i className="meta-icon meta-icon--user" />1 jugador
            </span>
            <span>
              <i className="meta-icon meta-icon--clock" />
              {featured.playtime}
            </span>
            <span className="compatibility">
              <i />
              {featured.compatibility}
            </span>
          </div>
          <FocusButton
            focusId="play-featured"
            focused={focusedId === "play-featured"}
            className="play-button"
            onClick={onPlay}
          >
            <span className="play-button__icon" aria-hidden="true">
              ▶
            </span>
            Jugar
          </FocusButton>
        </div>

        <div className="hero-panel__visual" aria-hidden="true">
          <div className="portal portal--outer" />
          <div className="portal portal--middle" />
          <div className="portal portal--inner" />
          <div className="horizon-glow" />
          <div className="crystal-city">
            <i />
            <i />
            <i />
            <i />
            <i />
          </div>
          <div className="water-plane" />
        </div>
        <div className="hero-panel__pagination" aria-label="Diapositiva 1 de 5">
          <i className="is-active" />
          <i />
          <i />
          <i />
          <i />
        </div>
      </section>

      <GameRail
        title="Continuar jugando"
        games={games.slice(0, 7)}
        prefix="continue"
        focusedId={focusedId}
        onChoose={onChooseGame}
      />
      <GameRail
        title="Recientes"
        games={games.slice(7, 14)}
        prefix="recent"
        focusedId={focusedId}
        onChoose={onChooseGame}
        compact
      />
    </div>
  );
}

interface GameRailProps {
  title: string;
  games: readonly GameSummary[];
  prefix: string;
  focusedId: string;
  compact?: boolean;
  onChoose: (game: GameSummary, focusId: string) => void;
}

function GameRail({
  title,
  games: railGames,
  prefix,
  focusedId,
  compact,
  onChoose,
}: GameRailProps) {
  return (
    <section className={`game-rail ${compact ? "game-rail--compact" : ""}`}>
      <div className="section-heading">
        <h2>{title}</h2>
        <span>{railGames.length} visibles</span>
      </div>
      <div className="game-rail__track">
        {railGames.map((game) => (
          <GameCard
            key={game.id}
            game={game}
            focusId={`${prefix}-${game.id}`}
            focusedId={focusedId}
            compact={compact}
            onChoose={onChoose}
          />
        ))}
      </div>
    </section>
  );
}

interface LibraryViewProps {
  focusedId: string;
  onChooseGame: (game: GameSummary, focusId: string) => void;
}

function LibraryView({ focusedId, onChooseGame }: LibraryViewProps) {
  return (
    <section className="library-view" aria-labelledby="library-title">
      <div className="library-view__heading">
        <div>
          <span className="eyebrow">Prueba de estrés visual</span>
          <h1 id="library-title">Tu biblioteca</h1>
        </div>
        <span className="library-count">{games.length} juegos simulados</span>
      </div>
      <div className="library-grid">
        {games.map((game) => (
          <GameCard
            key={game.id}
            game={game}
            focusId={`library-${game.id}`}
            focusedId={focusedId}
            onChoose={onChooseGame}
          />
        ))}
      </div>
    </section>
  );
}

interface PlaceholderViewProps {
  focusedId: string;
  kind: "add" | "settings";
  onAction: () => void;
}

function PlaceholderView({ focusedId, kind, onAction }: PlaceholderViewProps) {
  const isAdd = kind === "add";
  return (
    <section className="placeholder-view">
      <span className="eyebrow">M1 · Prototipo visual</span>
      <h1>{isAdd ? "Añadir juegos" : "Ajustes de Home"}</h1>
      <p>
        {isAdd
          ? "La importación real llegará después del Core. Esta pantalla validará el recorrido con mando sin tocar archivos del usuario."
          : "Los perfiles de calidad, movimiento reducido, escala y glifos se conectarán aquí durante M1."}
      </p>
      <FocusButton
        focusId="placeholder-action"
        focused={focusedId === "placeholder-action"}
        className="placeholder-action"
        onClick={onAction}
      >
        {isAdd ? "Ver alcance previsto" : "Probar confirmación"}
      </FocusButton>
    </section>
  );
}

export function App() {
  const [view, setView] = useState<ViewId>("home");
  const [focusedId, setFocusedId] = useState("play-featured");
  const [featuredId, setFeaturedId] = useState(games[0]?.id ?? "game-001");
  const [notice, setNotice] = useState<string | null>(null);

  const featured = useMemo(
    () => games.find((game) => game.id === featuredId) ?? games[0]!,
    [featuredId],
  );

  const moveFocus = useCallback(
    (direction: FocusDirection) => {
      const next = findNextFocus(focusedId, focusTargets(), direction);
      if (next) setFocusedId(next);
    },
    [focusedId],
  );

  const activateFocused = useCallback(() => {
    elementForFocusId(focusedId)?.click();
  }, [focusedId]);

  const goBack = useCallback(() => {
    if (notice) {
      setNotice(null);
      return;
    }
    if (view !== "home") {
      setView("home");
      setFocusedId("nav-home");
    }
  }, [notice, view]);

  const handleAction = useCallback(
    (action: ControllerAction) => {
      if (action === "accept") activateFocused();
      else if (action === "back") goBack();
      else moveFocus(action);
    },
    [activateFocused, goBack, moveFocus],
  );

  const controllerConnected = useControllerNavigation(handleAction);

  useEffect(() => {
    const keyToAction: Partial<Record<string, ControllerAction>> = {
      ArrowUp: "up",
      w: "up",
      W: "up",
      ArrowDown: "down",
      s: "down",
      S: "down",
      ArrowLeft: "left",
      a: "left",
      A: "left",
      ArrowRight: "right",
      d: "right",
      D: "right",
      Enter: "accept",
      " ": "accept",
      Escape: "back",
      Backspace: "back",
    };

    const onKeyDown = (event: KeyboardEvent) => {
      const action = keyToAction[event.key];
      if (!action || event.repeat) return;
      event.preventDefault();
      handleAction(action);
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [handleAction]);

  useEffect(() => {
    const element = elementForFocusId(focusedId);
    if (!element) return;
    element.focus({ preventScroll: true });
    element.scrollIntoView({
      behavior: "smooth",
      block: "nearest",
      inline: "nearest",
    });

    const gameId = element.dataset.gameId;
    if (gameId) setFeaturedId(gameId);
  }, [focusedId, view]);

  useEffect(() => {
    if (!notice) return;
    const timeout = window.setTimeout(() => setNotice(null), 3800);
    return () => window.clearTimeout(timeout);
  }, [notice]);

  const chooseView = (nextView: ViewId) => {
    setView(nextView);
    setFocusedId(`nav-${nextView}`);
    setNotice(null);
  };

  const chooseGame = (game: GameSummary, focusId: string) => {
    setFeaturedId(game.id);
    setFocusedId(focusId);
    if (view === "library") {
      setNotice(
        `${game.title} seleccionado · el lanzamiento real se conecta en M4.`,
      );
    }
  };

  return (
    <div
      className="app-shell"
      onPointerDownCapture={(event) => {
        const element = (event.target as HTMLElement).closest<HTMLElement>(
          "[data-focus-id]",
        );
        if (element?.dataset.focusId) setFocusedId(element.dataset.focusId);
      }}
    >
      <AmbientScene />

      <header className="top-bar">
        <div className="brand" aria-label="LIMEN">
          <span className="brand__halo" aria-hidden="true">
            <i />
          </span>
          <span className="brand__wordmark">LIMEN</span>
        </div>

        <nav className="main-nav" aria-label="Secciones principales">
          {navigation.map((item) => (
            <FocusButton
              key={item.id}
              focusId={`nav-${item.id}`}
              focused={focusedId === `nav-${item.id}`}
              className={`nav-item ${view === item.id ? "is-active" : ""}`}
              onClick={() => chooseView(item.id)}
              aria-current={view === item.id ? "page" : undefined}
            >
              {item.label}
            </FocusButton>
          ))}
        </nav>

        <div className="system-status">
          <span className="profile">
            <i>DV</i>
            <strong>Diego</strong>
          </span>
          <span
            className={`controller-state ${controllerConnected ? "is-connected" : ""}`}
          >
            <i />
            {controllerConnected ? "Mando" : "Teclado"}
          </span>
          <span className="battery" aria-label="Batería al 100%">
            <i />
            <strong>100%</strong>
          </span>
        </div>
      </header>

      <main className="main-content">
        {view === "home" && (
          <HomeView
            featured={featured}
            focusedId={focusedId}
            onPlay={() =>
              setNotice(
                "Demo visual M1: el flujo de lanzamiento se conectará al Core en M2–M4.",
              )
            }
            onChooseGame={chooseGame}
          />
        )}
        {view === "library" && (
          <LibraryView focusedId={focusedId} onChooseGame={chooseGame} />
        )}
        {(view === "add" || view === "settings") && (
          <PlaceholderView
            kind={view}
            focusedId={focusedId}
            onAction={() =>
              setNotice(
                "Recorrido confirmado. Esta función se completará en su hito.",
              )
            }
          />
        )}
      </main>

      <footer className="control-footer">
        <div>
          <ControllerHint glyph="A">Seleccionar</ControllerHint>
          <ControllerHint glyph="B">Atrás</ControllerHint>
          <ControllerHint glyph="☰">Menú</ControllerHint>
        </div>
        <span className="build-label">M1 · HOME VISUAL SLICE</span>
      </footer>

      {notice && (
        <div className="notice" role="status" aria-live="polite">
          {notice}
        </div>
      )}
    </div>
  );
}

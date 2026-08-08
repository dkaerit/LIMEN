import {
  Component,
  lazy,
  Suspense,
  useCallback,
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
  type CSSProperties,
  type ReactNode,
} from "react";

import {
  findNextFocus,
  type FocusDirection,
  type FocusTarget,
} from "@limen/focus-engine";
import { ControllerHint, FocusButton, PlatformBadge } from "@limen/ui-kit";

import { Icon, type IconName } from "./components/Icon";
import { games, type GamePlatform, type GameSummary } from "./data/games";
import {
  useControllerNavigation,
  type ControllerAction,
} from "./hooks/useControllerNavigation";
import {
  resolveGraphicsProfile,
  runHomeBenchmark,
  type GraphicsPreference,
  type HomeBenchmarkResult,
  type ViewportMetrics,
} from "./lib/performance";
import { calculateVirtualGrid } from "./lib/virtualization";

const AmbientScene = lazy(async () => {
  const graphics = await import("@limen/graphics");
  return { default: graphics.AmbientScene };
});

function SceneFallback() {
  return (
    <div className="ambient-scene ambient-scene--2d" aria-hidden="true">
      <div className="ambient-scene__fallback" />
      <div className="ambient-scene__vignette" />
    </div>
  );
}

class SceneErrorBoundary extends Component<
  { children: ReactNode },
  { failed: boolean }
> {
  state = { failed: false };

  static getDerivedStateFromError() {
    return { failed: true };
  }

  render() {
    return this.state.failed ? <SceneFallback /> : this.props.children;
  }
}

type ViewId =
  | "home"
  | "library"
  | "discover"
  | "community"
  | "apps"
  | "add"
  | "settings"
  | "detail";

type InputMethod = "gamepad" | "keyboard" | "pointer";

interface NavItem {
  id: Exclude<ViewId, "detail">;
  label: string;
  icon: IconName;
}

const navigation: NavItem[] = [
  { id: "home", label: "Inicio", icon: "home" },
  { id: "library", label: "Mi biblioteca", icon: "library" },
  { id: "discover", label: "Descubrir", icon: "discover" },
  { id: "community", label: "Comunidad", icon: "community" },
  { id: "apps", label: "Aplicaciones", icon: "apps" },
  { id: "add", label: "Añadir contenido", icon: "add" },
  { id: "settings", label: "Configuración", icon: "settings" },
];

const viewTitles: Record<ViewId, string> = {
  home: "Tu espacio de juego",
  library: "Mi biblioteca",
  discover: "Descubrir",
  community: "Comunidad",
  apps: "Aplicaciones",
  add: "Añadir contenido",
  settings: "Configuración",
  detail: "Ficha del juego",
};

const libraryFilters: Array<"Todos" | GamePlatform> = [
  "Todos",
  "PC",
  "PS2",
  "ARCADE",
  "HANDHELD",
  "16-BIT",
];

const graphicsOptions: Array<{
  id: GraphicsPreference;
  label: string;
}> = [
  { id: "auto", label: "Auto" },
  { id: "quality", label: "Calidad" },
  { id: "balanced", label: "Equilibrado" },
  { id: "performance", label: "Rendimiento" },
  { id: "2d", label: "Solo 2D" },
];

const communityProjects = [
  {
    id: "translation",
    title: "Traducción: Crystal Voyage",
    kind: "Traducción",
    rating: "4,8",
    installs: "5.842",
    accent: "#65c8ff",
  },
  {
    id: "coop",
    title: "Cooperativo sin fisuras",
    kind: "Mod",
    rating: "4,7",
    installs: "2.410",
    accent: "#6cc4ff",
  },
  {
    id: "remake",
    title: "Proyecto Nightfall",
    kind: "Juego comunitario",
    rating: "4,6",
    installs: "862",
    accent: "#b879ff",
  },
];

const sourceOptions: Array<{
  id: string;
  label: string;
  description: string;
  icon: IconName;
  milestone: string;
}> = [
  {
    id: "folder",
    label: "Carpeta local",
    description: "Detecta accesos y metadatos sin mover tus archivos.",
    icon: "folder",
    milestone: "M2",
  },
  {
    id: "disc",
    label: "Disco",
    description: "Lee un medio físico compatible.",
    icon: "disc",
    milestone: "M3",
  },
  {
    id: "cloud",
    label: "Nube personal",
    description: "Conecta una fuente que tú controlas.",
    icon: "cloud",
    milestone: "M3",
  },
  {
    id: "community",
    label: "Comunidad",
    description: "Instala proyectos mediante recetas verificadas.",
    icon: "community",
    milestone: "M3",
  },
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
    [
      ...document.querySelectorAll<HTMLElement>(
        "[data-focus-id]:not([disabled])",
      ),
    ].find((element) => element.dataset.focusId === focusId) ?? null
  );
}

function gameColors(game: GameSummary): CSSProperties {
  return {
    "--game-accent": game.accent,
    "--game-accent-secondary": game.accentSecondary,
  } as CSSProperties;
}

function useViewportMetrics(): ViewportMetrics {
  const [viewport, setViewport] = useState<ViewportMetrics>(() => ({
    width: window.innerWidth,
    height: window.innerHeight,
    devicePixelRatio: window.devicePixelRatio,
  }));

  useEffect(() => {
    const update = () =>
      setViewport({
        width: window.innerWidth,
        height: window.innerHeight,
        devicePixelRatio: window.devicePixelRatio,
      });
    window.addEventListener("resize", update);
    return () => window.removeEventListener("resize", update);
  }, []);

  return viewport;
}

function usePrefersReducedMotion(): boolean {
  const [reducedMotion, setReducedMotion] = useState(
    () => window.matchMedia("(prefers-reduced-motion: reduce)").matches,
  );

  useEffect(() => {
    const mediaQuery = window.matchMedia("(prefers-reduced-motion: reduce)");
    const update = () => setReducedMotion(mediaQuery.matches);
    mediaQuery.addEventListener("change", update);
    update();
    return () => mediaQuery.removeEventListener("change", update);
  }, []);

  return reducedMotion;
}

function Panel({
  children,
  className = "",
}: {
  children: ReactNode;
  className?: string;
}) {
  return <section className={`glass-panel ${className}`}>{children}</section>;
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
      aria-label={`Abrir ${game.title}, ${game.platform}, progreso ${game.progress}%`}
    >
      <img
        className="game-card__art"
        src={game.artwork}
        alt=""
        loading="lazy"
        style={{ objectPosition: game.artworkPosition }}
      />
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

function VirtualizedLibraryGrid({
  libraryGames,
  focusedId,
  onChooseGame,
}: {
  libraryGames: readonly GameSummary[];
  focusedId: string;
  onChooseGame: (game: GameSummary, focusId: string) => void;
}) {
  const gridRef = useRef<HTMLDivElement>(null);
  const [viewport, setViewport] = useState({
    width: 1000,
    height: 720,
    scrollOffset: 0,
  });
  const focusedGameId = focusedId.startsWith("library-")
    ? focusedId.slice("library-".length)
    : "";
  const pinnedIndex = libraryGames.findIndex(
    (game) => game.id === focusedGameId,
  );
  const layout = useMemo(
    () =>
      calculateVirtualGrid({
        itemCount: libraryGames.length,
        width: viewport.width,
        scrollOffset: viewport.scrollOffset,
        viewportHeight: viewport.height,
        pinnedIndex,
      }),
    [libraryGames.length, pinnedIndex, viewport],
  );

  useLayoutEffect(() => {
    const grid = gridRef.current;
    const scrollRoot = grid?.closest<HTMLElement>(".view-scroll");
    if (!grid || !scrollRoot) return;

    const updateViewport = () => {
      const nextViewport = {
        width: grid.clientWidth,
        height: scrollRoot.clientHeight,
        scrollOffset: Math.max(0, scrollRoot.scrollTop - grid.offsetTop),
      };
      setViewport((current) =>
        current.width === nextViewport.width &&
        current.height === nextViewport.height &&
        current.scrollOffset === nextViewport.scrollOffset
          ? current
          : nextViewport,
      );
    };

    const resizeObserver = new ResizeObserver(updateViewport);
    resizeObserver.observe(grid);
    resizeObserver.observe(scrollRoot);
    scrollRoot.addEventListener("scroll", updateViewport, { passive: true });
    updateViewport();

    return () => {
      resizeObserver.disconnect();
      scrollRoot.removeEventListener("scroll", updateViewport);
    };
  }, [libraryGames.length]);

  return (
    <div
      ref={gridRef}
      className="library-virtualizer"
      style={{ height: `${layout.totalHeight}px` }}
      role="list"
      aria-label={`${libraryGames.length} juegos virtualizados`}
      data-total-items={libraryGames.length}
      data-rendered-rows={layout.visibleRows.length}
      data-columns={layout.columns}
    >
      {layout.visibleRows.map((rowIndex) => {
        const rowGames = libraryGames.slice(
          rowIndex * layout.columns,
          Math.min((rowIndex + 1) * layout.columns, libraryGames.length),
        );

        return (
          <div
            key={rowIndex}
            className="library-virtual-row"
            style={{
              height: `${layout.rowHeight - layout.gap}px`,
              gridTemplateColumns: `repeat(${layout.columns}, minmax(0, 1fr))`,
              gap: `${layout.gap}px`,
              transform: `translateY(${rowIndex * layout.rowHeight}px)`,
            }}
          >
            {rowGames.map((game) => (
              <div key={game.id} className="virtual-game-cell" role="listitem">
                <GameCard
                  game={game}
                  focusId={`library-${game.id}`}
                  focusedId={focusedId}
                  onChoose={onChooseGame}
                />
              </div>
            ))}
          </div>
        );
      })}
    </div>
  );
}

interface HeroPanelProps {
  featured: GameSummary;
  featuredIndex: number;
  focusedId: string;
  onPlay: () => void;
  onDetails: () => void;
  onSlide: (index: number) => void;
}

function HeroPanel({
  featured,
  featuredIndex,
  focusedId,
  onPlay,
  onDetails,
  onSlide,
}: HeroPanelProps) {
  return (
    <section
      className="hero-panel"
      style={gameColors(featured)}
      aria-labelledby="featured-title"
    >
      <img
        className="hero-panel__art"
        src={featured.artwork}
        alt=""
        style={{ objectPosition: featured.artworkPosition }}
      />
      <div className="hero-panel__scrim" />
      <div className="hero-panel__copy">
        <span className="eyebrow">Selección LIMEN · Verificada</span>
        <h1 id="featured-title">{featured.title}</h1>
        <p className="hero-panel__subtitle">{featured.subtitle}</p>
        <div className="hero-panel__metadata">
          <PlatformBadge>{featured.platform}</PlatformBadge>
          <span>
            <Icon name="profile" />1 jugador
          </span>
          <span>
            <span className="clock-icon" />
            {featured.playtime}
          </span>
          <span className="compatibility">
            <Icon name="check" />
            {featured.compatibility}
          </span>
        </div>
        <div className="hero-panel__actions">
          <FocusButton
            focusId="play-featured"
            focused={focusedId === "play-featured"}
            className="primary-action"
            onClick={onPlay}
          >
            <span className="play-triangle" aria-hidden="true" />
            Jugar
          </FocusButton>
          <FocusButton
            focusId="details-featured"
            focused={focusedId === "details-featured"}
            className="secondary-action"
            onClick={onDetails}
          >
            Ver ficha
          </FocusButton>
        </div>
      </div>
      <div className="hero-panel__pagination" aria-label="Juegos destacados">
        {games.slice(0, 5).map((game, index) => (
          <FocusButton
            key={game.id}
            focusId={`hero-slide-${index}`}
            focused={focusedId === `hero-slide-${index}`}
            className={index === featuredIndex ? "is-active" : ""}
            onClick={() => onSlide(index)}
            aria-label={`Mostrar ${game.title}`}
          />
        ))}
      </div>
    </section>
  );
}

function GameRail({
  title,
  games: railGames,
  prefix,
  focusedId,
  onChoose,
}: {
  title: string;
  games: readonly GameSummary[];
  prefix: string;
  focusedId: string;
  onChoose: (game: GameSummary, focusId: string) => void;
}) {
  return (
    <section className="game-rail">
      <div className="section-heading">
        <h2>{title}</h2>
        <span>{railGames.length} elementos</span>
      </div>
      <div className="game-rail__track">
        {railGames.map((game) => (
          <GameCard
            key={game.id}
            game={game}
            focusId={`${prefix}-${game.id}`}
            focusedId={focusedId}
            onChoose={onChoose}
          />
        ))}
      </div>
    </section>
  );
}

function HomeView({
  featured,
  featuredIndex,
  focusedId,
  onPlay,
  onDetails,
  onSlide,
  onChooseGame,
}: HeroPanelProps & {
  onChooseGame: (game: GameSummary, focusId: string) => void;
}) {
  return (
    <div className="home-view view-scroll">
      <HeroPanel
        featured={featured}
        featuredIndex={featuredIndex}
        focusedId={focusedId}
        onPlay={onPlay}
        onDetails={onDetails}
        onSlide={onSlide}
      />
      <GameRail
        title="Continuar jugando"
        games={games.slice(0, 5)}
        prefix="continue"
        focusedId={focusedId}
        onChoose={onChooseGame}
      />
      <GameRail
        title="Recientes"
        games={games.slice(7, 12)}
        prefix="recent"
        focusedId={focusedId}
        onChoose={onChooseGame}
      />
    </div>
  );
}

function LibraryView({
  focusedId,
  filter,
  onFilter,
  onChooseGame,
  onSearch,
}: {
  focusedId: string;
  filter: (typeof libraryFilters)[number];
  onFilter: (filter: (typeof libraryFilters)[number]) => void;
  onChooseGame: (game: GameSummary, focusId: string) => void;
  onSearch: () => void;
}) {
  const filteredGames = useMemo(
    () =>
      filter === "Todos"
        ? games
        : games.filter((game) => game.platform === filter),
    [filter],
  );

  return (
    <div className="library-view view-scroll">
      <div className="view-heading">
        <div>
          <span className="eyebrow">Una biblioteca · cualquier juego</span>
          <h1>Mi biblioteca</h1>
        </div>
        <FocusButton
          focusId="library-search"
          focused={focusedId === "library-search"}
          className="search-action"
          onClick={onSearch}
        >
          <Icon name="search" /> Buscar
        </FocusButton>
      </div>
      <div className="filter-row" aria-label="Filtrar biblioteca">
        {libraryFilters.map((item) => (
          <FocusButton
            key={item}
            focusId={`filter-${item}`}
            focused={focusedId === `filter-${item}`}
            className={filter === item ? "is-active" : ""}
            onClick={() => onFilter(item)}
          >
            {item}
          </FocusButton>
        ))}
        <span className="library-count">{filteredGames.length} juegos</span>
      </div>
      <VirtualizedLibraryGrid
        libraryGames={filteredGames}
        focusedId={focusedId}
        onChooseGame={onChooseGame}
      />
    </div>
  );
}

function DiscoverView({
  focusedId,
  onChooseGame,
  onNotice,
}: {
  focusedId: string;
  onChooseGame: (game: GameSummary, focusId: string) => void;
  onNotice: (message: string) => void;
}) {
  return (
    <div className="catalog-view view-scroll">
      <div className="view-heading">
        <div>
          <span className="eyebrow">Colecciones locales y conectadas</span>
          <h1>Descubrir</h1>
        </div>
      </div>
      <div className="feature-grid">
        <Panel className="feature-card feature-card--wide">
          <span className="status-pill status-pill--blue">
            Colección destacada
          </span>
          <h2>Mundos más allá del umbral</h2>
          <p>Una selección editorial construida con tu propia biblioteca.</p>
          <FocusButton
            focusId="discover-collection"
            focused={focusedId === "discover-collection"}
            className="primary-action"
            onClick={() =>
              onNotice(
                "Colección abierta: se muestran 12 juegos de tu biblioteca.",
              )
            }
          >
            Explorar colección
          </FocusButton>
        </Panel>
        <Panel className="feature-card">
          <Icon name="spark" />
          <h2>Recomendaciones locales</h2>
          <p>Sin perfiles publicitarios ni seguimiento externo.</p>
          <FocusButton
            focusId="discover-local"
            focused={focusedId === "discover-local"}
            className="secondary-action"
            onClick={() =>
              onNotice("La recomendación local se conectará al catálogo en M2.")
            }
          >
            Ver cómo funciona
          </FocusButton>
        </Panel>
      </div>
      <GameRail
        title="Para volver esta noche"
        games={games.slice(18, 23)}
        prefix="discover"
        focusedId={focusedId}
        onChoose={onChooseGame}
      />
    </div>
  );
}

function CommunityView({
  focusedId,
  onNotice,
}: {
  focusedId: string;
  onNotice: (message: string) => void;
}) {
  const [selectedProject, setSelectedProject] = useState(communityProjects[0]!);

  return (
    <div className="community-view view-scroll">
      <div className="view-heading">
        <div>
          <span className="eyebrow">La comunidad crea · tú decides</span>
          <h1>Comunidad</h1>
        </div>
        <FocusButton
          focusId="community-search"
          focused={focusedId === "community-search"}
          className="search-action"
          onClick={() =>
            onNotice(
              "La búsqueda comunitaria se conectará al catálogo verificado en M3.",
            )
          }
        >
          <Icon name="search" /> Buscar proyectos
        </FocusButton>
      </div>
      <div className="community-layout">
        <Panel className="community-browser">
          <div className="panel-heading">
            <div>
              <small>Explorar proyectos</small>
              <h2>Destacados</h2>
            </div>
            <span className="status-pill status-pill--info">
              Vista previa M1
            </span>
          </div>
          <div className="project-list">
            {communityProjects.map((project) => (
              <FocusButton
                key={project.id}
                focusId={`project-${project.id}`}
                focused={focusedId === `project-${project.id}`}
                className={
                  selectedProject.id === project.id ? "is-selected" : ""
                }
                style={{ "--project-accent": project.accent } as CSSProperties}
                onClick={() => setSelectedProject(project)}
              >
                <span className="project-art">
                  <Icon name="community" />
                </span>
                <strong>{project.title}</strong>
                <small>{project.kind}</small>
                <span>
                  ★ {project.rating} · {project.installs}
                </span>
              </FocusButton>
            ))}
          </div>
        </Panel>
        <Panel className="project-detail">
          <div className="panel-heading">
            <div>
              <small>{selectedProject.kind}</small>
              <h2>{selectedProject.title}</h2>
            </div>
            <Icon name="shield" />
          </div>
          <div className="project-detail__body">
            <div className="recipe-preview">
              <span className="recipe-preview__art">
                <Icon name="spark" />
              </span>
              <div>
                <span>Compatible con versiones 1.0–1.8</span>
                <strong>24,7 MB · Reversible</strong>
              </div>
            </div>
            <ul className="check-list">
              <li>
                <Icon name="check" /> Verifica la versión del juego
              </li>
              <li>
                <Icon name="check" /> Realiza una copia de seguridad
              </li>
              <li>
                <Icon name="check" /> Solo modifica el juego seleccionado
              </li>
              <li>
                <Icon name="check" /> Puede desinstalarse
              </li>
            </ul>
            <div className="verified-box">
              <Icon name="shield" />
              <div>
                <strong>Receta verificada</strong>
                <span>
                  La seguridad y permisos serán visibles antes de instalar.
                </span>
              </div>
            </div>
          </div>
          <div className="panel-actions">
            <FocusButton
              focusId="community-install"
              focused={focusedId === "community-install"}
              className="primary-action"
              onClick={() =>
                onNotice(
                  "Instalación comunitaria: prevista para M3; esta vista no modifica archivos.",
                )
              }
            >
              Instalar (M3)
            </FocusButton>
            <FocusButton
              focusId="community-recipe"
              focused={focusedId === "community-recipe"}
              className="secondary-action"
              onClick={() =>
                onNotice(
                  "Receta: verificación, copia, aplicación y rollback controlado.",
                )
              }
            >
              Ver receta
            </FocusButton>
          </div>
        </Panel>
      </div>
    </div>
  );
}

function AppsView({
  focusedId,
  onNotice,
}: {
  focusedId: string;
  onNotice: (message: string) => void;
}) {
  return (
    <div className="apps-view view-scroll">
      <div className="view-heading">
        <div>
          <span className="eyebrow">Tú eliges qué puede abrir LIMEN</span>
          <h1>Aplicaciones</h1>
        </div>
      </div>
      <div className="apps-layout">
        <Panel className="apps-library">
          <div className="panel-heading">
            <div>
              <small>Bibliotecas externas</small>
              <h2>Centro de aplicaciones</h2>
            </div>
            <Icon name="apps" />
          </div>
          <div className="empty-state">
            <span>
              <Icon name="apps" />
            </span>
            <h3>No hay aplicaciones instaladas</h3>
            <p>
              Añade un ejecutor o herramienta compatible cuando M3 esté
              disponible.
            </p>
          </div>
          <FocusButton
            focusId="apps-add"
            focused={focusedId === "apps-add"}
            className="primary-action"
            onClick={() =>
              onNotice("El instalador aislado de aplicaciones llegará en M3.")
            }
          >
            <Icon name="add" /> Añadir aplicación
          </FocusButton>
        </Panel>
        <Panel className="permission-card">
          <div className="panel-heading">
            <div>
              <small>Control explícito</small>
              <h2>Permisos del ejecutor</h2>
            </div>
            <Icon name="shield" />
          </div>
          <div className="permission-row">
            <Icon name="gamepad" />
            <span>Mando</span>
            <strong>Permitido</strong>
          </div>
          <div className="permission-row">
            <Icon name="folder" />
            <span>Carpeta seleccionada</span>
            <strong>Permitido</strong>
          </div>
          <div className="permission-row">
            <Icon name="wifi" />
            <span>Red</span>
            <strong>Preguntar</strong>
          </div>
          <div className="permission-row">
            <Icon name="cloud" />
            <span>Cuentas</span>
            <strong>Bloqueado</strong>
          </div>
          <div className="security-note">
            <Icon name="shield" /> Nunca recibe acceso completo al sistema.
          </div>
          <FocusButton
            focusId="apps-permissions"
            focused={focusedId === "apps-permissions"}
            className="secondary-action"
            onClick={() =>
              onNotice(
                "Los permisos se concederán aplicación por aplicación, nunca globalmente.",
              )
            }
          >
            Revisar modelo de seguridad
          </FocusButton>
        </Panel>
      </div>
    </div>
  );
}

function AddView({
  focusedId,
  selectedSource,
  onSelectSource,
  onNotice,
}: {
  focusedId: string;
  selectedSource: string;
  onSelectSource: (source: string) => void;
  onNotice: (message: string) => void;
}) {
  const selected =
    sourceOptions.find((source) => source.id === selectedSource) ??
    sourceOptions[0]!;

  return (
    <div className="add-view view-scroll">
      <div className="view-heading">
        <div>
          <span className="eyebrow">Importación sin fricción</span>
          <h1>Añadir contenido</h1>
        </div>
      </div>
      <div className="add-layout">
        <Panel className="source-list">
          <div className="panel-heading">
            <div>
              <small>Origen</small>
              <h2>Elige una fuente</h2>
            </div>
          </div>
          {sourceOptions.map((source) => (
            <FocusButton
              key={source.id}
              focusId={`source-${source.id}`}
              focused={focusedId === `source-${source.id}`}
              className={selectedSource === source.id ? "is-selected" : ""}
              onClick={() => onSelectSource(source.id)}
            >
              <Icon name={source.icon} />
              <span>
                <strong>{source.label}</strong>
                <small>{source.description}</small>
              </span>
              <em>{source.milestone}</em>
            </FocusButton>
          ))}
        </Panel>
        <Panel className="scanner-panel">
          <span className="scanner-visual">
            <Icon name={selected.icon} />
            <i />
          </span>
          <span className="status-pill status-pill--info">
            Disponible en {selected.milestone}
          </span>
          <h2>{selected.label}</h2>
          <p>{selected.description}</p>
          <div className="scanner-summary">
            <span>
              <Icon name="check" /> No mueve archivos
            </span>
            <span>
              <Icon name="shield" /> Requiere confirmación
            </span>
            <span>
              <Icon name="gamepad" /> Recorrido con mando
            </span>
          </div>
          <FocusButton
            focusId="source-review"
            focused={focusedId === "source-review"}
            className="primary-action"
            onClick={() =>
              onNotice(
                `${selected.label}: el flujo se activará en ${selected.milestone}; ahora solo es una vista segura.`,
              )
            }
          >
            Revisar alcance
          </FocusButton>
        </Panel>
      </div>
    </div>
  );
}

function SettingsView({
  focusedId,
  motionEnabled,
  highContrast,
  graphicsPreference,
  effectiveProfile,
  viewportMetrics,
  benchmark,
  benchmarkRunning,
  onToggleMotion,
  onToggleContrast,
  onSelectGraphics,
  onRunBenchmark,
  onNotice,
}: {
  focusedId: string;
  motionEnabled: boolean;
  highContrast: boolean;
  graphicsPreference: GraphicsPreference;
  effectiveProfile: HomeBenchmarkResult["profile"];
  viewportMetrics: ViewportMetrics;
  benchmark: HomeBenchmarkResult | null;
  benchmarkRunning: boolean;
  onToggleMotion: () => void;
  onToggleContrast: () => void;
  onSelectGraphics: (preference: GraphicsPreference) => void;
  onRunBenchmark: () => void;
  onNotice: (message: string) => void;
}) {
  return (
    <div className="settings-view view-scroll">
      <div className="view-heading">
        <div>
          <span className="eyebrow">Tu experiencia</span>
          <h1>Configuración</h1>
        </div>
      </div>
      <div className="settings-grid">
        <Panel className="settings-card settings-card--graphics">
          <div className="panel-heading">
            <div>
              <small>Gráficos</small>
              <h2>Escena tridimensional</h2>
            </div>
            <Icon name="spark" />
          </div>
          <p>
            Figuras WebGL en tiempo real con degradación automática según la
            resolución renderizada.
          </p>
          <div
            className="quality-options"
            role="radiogroup"
            aria-label="Calidad gráfica"
          >
            {graphicsOptions.map((option) => (
              <FocusButton
                key={option.id}
                focusId={`quality-${option.id}`}
                focused={focusedId === `quality-${option.id}`}
                className={graphicsPreference === option.id ? "is-active" : ""}
                role="radio"
                aria-checked={graphicsPreference === option.id}
                onClick={() => onSelectGraphics(option.id)}
              >
                {option.label}
              </FocusButton>
            ))}
          </div>
          <span className="quality-readout">
            Perfil efectivo: <strong>{effectiveProfile}</strong> ·{" "}
            {viewportMetrics.width}×{viewportMetrics.height} @{" "}
            {viewportMetrics.devicePixelRatio.toFixed(2)}x
          </span>
          <FocusButton
            focusId="setting-motion"
            focused={focusedId === "setting-motion"}
            className="toggle-row"
            role="switch"
            aria-checked={motionEnabled}
            onClick={onToggleMotion}
          >
            <span>Movimiento ambiental</span>
            <i className={motionEnabled ? "is-on" : ""} />
          </FocusButton>
        </Panel>
        <Panel className="settings-card">
          <div className="panel-heading">
            <div>
              <small>Accesibilidad</small>
              <h2>Legibilidad</h2>
            </div>
            <Icon name="profile" />
          </div>
          <p>Aumenta separación, bordes y contraste de las superficies.</p>
          <FocusButton
            focusId="setting-contrast"
            focused={focusedId === "setting-contrast"}
            className="toggle-row"
            role="switch"
            aria-checked={highContrast}
            onClick={onToggleContrast}
          >
            <span>Contraste reforzado</span>
            <i className={highContrast ? "is-on" : ""} />
          </FocusButton>
        </Panel>
        <Panel className="settings-card">
          <div className="panel-heading">
            <div>
              <small>Control</small>
              <h2>Mando principal</h2>
            </div>
            <Icon name="gamepad" />
          </div>
          <p>La detección actual usa Gamepad API mientras llega LIMEN Input.</p>
          <FocusButton
            focusId="setting-controller"
            focused={focusedId === "setting-controller"}
            className="secondary-action"
            onClick={() =>
              onNotice(
                "Pulsa cualquier botón del mando para activarlo; A selecciona y B vuelve.",
              )
            }
          >
            Ver controles
          </FocusButton>
        </Panel>
        <Panel className="settings-card settings-card--benchmark">
          <div className="panel-heading">
            <div>
              <small>Diagnóstico local</small>
              <h2>Presupuesto de frame</h2>
            </div>
            <Icon name="spark" />
          </div>
          <p>
            Mide diez segundos en este dispositivo. No envía telemetría ni
            guarda identificadores de hardware.
          </p>
          <div className="benchmark-metrics" aria-live="polite">
            <span>
              <small>FPS medio</small>
              <strong>
                {benchmark ? benchmark.averageFps.toFixed(1) : "—"}
              </strong>
            </span>
            <span>
              <small>Frame P95</small>
              <strong>
                {benchmark ? `${benchmark.p95FrameMs.toFixed(1)} ms` : "—"}
              </strong>
            </span>
            <span>
              <small>Heap JS</small>
              <strong>
                {benchmark?.usedHeapMb
                  ? `${benchmark.usedHeapMb.toFixed(1)} MB`
                  : "N/D"}
              </strong>
            </span>
          </div>
          <FocusButton
            focusId="setting-benchmark"
            focused={focusedId === "setting-benchmark"}
            className="secondary-action"
            disabled={benchmarkRunning}
            onClick={onRunBenchmark}
          >
            {benchmarkRunning ? "Midiendo 10 s…" : "Medir rendimiento"}
          </FocusButton>
        </Panel>
      </div>
    </div>
  );
}

function DetailView({
  game,
  focusedId,
  onPlay,
  onNotice,
}: {
  game: GameSummary;
  focusedId: string;
  onPlay: () => void;
  onNotice: (message: string) => void;
}) {
  return (
    <div className="detail-view view-scroll" style={gameColors(game)}>
      <Panel className="detail-hero">
        <img
          src={game.artwork}
          alt=""
          style={{ objectPosition: game.artworkPosition }}
        />
        <div className="detail-hero__scrim" />
        <div className="detail-hero__copy">
          <span className="eyebrow">Ficha local</span>
          <h1>{game.title}</h1>
          <p>{game.subtitle}</p>
          <div className="hero-panel__metadata">
            <PlatformBadge>{game.platform}</PlatformBadge>
            <span>
              <Icon name="check" /> {game.compatibility}
            </span>
            <span>{game.playtime}</span>
          </div>
          <div className="hero-panel__actions">
            <FocusButton
              focusId="detail-play"
              focused={focusedId === "detail-play"}
              className="primary-action"
              onClick={onPlay}
            >
              <span className="play-triangle" /> Jugar
            </FocusButton>
            <FocusButton
              focusId="detail-manage"
              focused={focusedId === "detail-manage"}
              className="secondary-action"
              onClick={() =>
                onNotice("Gestión de contenido y perfiles: prevista para M3.")
              }
            >
              Gestionar contenido
            </FocusButton>
          </div>
        </div>
      </Panel>
      <div className="detail-grid">
        <Panel>
          <Icon name="community" />
          <h2>Mods</h2>
          <strong>0 activos</strong>
          <span>Comunidad · M3</span>
        </Panel>
        <Panel>
          <Icon name="apps" />
          <h2>Versiones</h2>
          <strong>1 detectada</strong>
          <span>Atlas · M2</span>
        </Panel>
        <Panel>
          <Icon name="library" />
          <h2>Partidas</h2>
          <strong>Local</strong>
          <span>Vault · M3</span>
        </Panel>
        <Panel>
          <Icon name="settings" />
          <h2>Configuración</h2>
          <strong>Predeterminada</strong>
          <span>Perfil local</span>
        </Panel>
      </div>
    </div>
  );
}

export function App() {
  const [view, setView] = useState<ViewId>("home");
  const [focusedId, setFocusedId] = useState("play-featured");
  const [featuredIndex, setFeaturedIndex] = useState(0);
  const [selectedGame, setSelectedGame] = useState(games[0]!);
  const [notice, setNotice] = useState<string | null>(null);
  const [libraryFilter, setLibraryFilter] =
    useState<(typeof libraryFilters)[number]>("Todos");
  const [selectedSource, setSelectedSource] = useState("folder");
  const [motionEnabled, setMotionEnabled] = useState(true);
  const [highContrast, setHighContrast] = useState(false);
  const [graphicsPreference, setGraphicsPreference] =
    useState<GraphicsPreference>("auto");
  const [benchmark, setBenchmark] = useState<HomeBenchmarkResult | null>(null);
  const [benchmarkRunning, setBenchmarkRunning] = useState(false);
  const [inputMethod, setInputMethod] = useState<InputMethod>("keyboard");
  const previousView = useRef<ViewId>("home");
  const returnFocus = useRef("play-featured");
  const sawController = useRef(false);
  const viewportMetrics = useViewportMetrics();
  const prefersReducedMotion = usePrefersReducedMotion();

  const featured = games[featuredIndex] ?? games[0]!;
  const effectiveProfile = resolveGraphicsProfile(
    graphicsPreference,
    viewportMetrics,
  );
  const sceneEnabled =
    motionEnabled && !prefersReducedMotion && effectiveProfile !== "2d";
  const showNotice = useCallback((message: string) => setNotice(message), []);

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
    if (view === "detail") {
      setView(
        previousView.current === "detail" ? "home" : previousView.current,
      );
      setFocusedId(returnFocus.current);
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
    if (controllerConnected) {
      sawController.current = true;
      setInputMethod("gamepad");
      showNotice("Mando conectado · los glifos se han actualizado.");
    } else if (sawController.current) {
      sawController.current = false;
      setInputMethod("keyboard");
      showNotice(
        "Mando desconectado · puedes continuar con teclado y volver a conectarlo.",
      );
    }
  }, [controllerConnected, showNotice]);

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
      if (
        !action ||
        (event.repeat && (action === "accept" || action === "back"))
      )
        return;
      event.preventDefault();
      setInputMethod("keyboard");
      handleAction(action);
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [handleAction]);

  useEffect(() => {
    let frame = 0;
    let attempts = 0;

    const synchronizeFocus = () => {
      const element = elementForFocusId(focusedId);
      if (!element) {
        attempts += 1;
        if (attempts < 5)
          frame = window.requestAnimationFrame(synchronizeFocus);
        return;
      }

      element.focus({ preventScroll: true });
      element.scrollIntoView({
        behavior: "auto",
        block: "nearest",
        inline: "nearest",
      });

      const rect = element.getBoundingClientRect();
      window.dispatchEvent(
        new CustomEvent("limen-focus-move", {
          detail: {
            x: ((rect.left + rect.width / 2) / window.innerWidth) * 2 - 1,
            y: -(((rect.top + rect.height / 2) / window.innerHeight) * 2 - 1),
          },
        }),
      );

      const gameId = element.dataset.gameId;
      const gameIndex = games.findIndex((game) => game.id === gameId);
      if (gameIndex >= 0 && gameIndex < 5) setFeaturedIndex(gameIndex);
    };

    frame = window.requestAnimationFrame(synchronizeFocus);
    return () => window.cancelAnimationFrame(frame);
  }, [focusedId, view]);

  useEffect(() => {
    if (!notice) return;
    const timeout = window.setTimeout(() => setNotice(null), 5200);
    return () => window.clearTimeout(timeout);
  }, [notice]);

  const chooseView = (nextView: Exclude<ViewId, "detail">) => {
    setView(nextView);
    setFocusedId(`nav-${nextView}`);
    setNotice(null);
  };

  const openGame = (game: GameSummary, focusId: string) => {
    previousView.current = view;
    returnFocus.current = focusId;
    setSelectedGame(game);
    setView("detail");
    setFocusedId("detail-play");
    setNotice(null);
  };

  const runBenchmark = async () => {
    if (benchmarkRunning) return;
    setBenchmarkRunning(true);
    const result = await runHomeBenchmark(
      10_000,
      sceneEnabled ? effectiveProfile : "2d",
    );
    setBenchmark(result);
    setBenchmarkRunning(false);
    showNotice(
      `Medición completada: ${result.averageFps.toFixed(1)} FPS · P95 ${result.p95FrameMs.toFixed(1)} ms.`,
    );
  };

  const openFeatured = () => openGame(featured, "details-featured");
  const playNotice = () =>
    showNotice(
      "LIMEN Core empezará en M2. Esta versión todavía no lanza procesos ni toca tus juegos.",
    );

  return (
    <div
      className={`app-shell ${sceneEnabled ? "" : "is-motion-reduced"} ${highContrast ? "is-high-contrast" : ""}`}
      data-view={view}
      onPointerDownCapture={(event) => {
        setInputMethod("pointer");
        const element = (event.target as HTMLElement).closest<HTMLElement>(
          "[data-focus-id]",
        );
        if (element?.dataset.focusId) setFocusedId(element.dataset.focusId);
      }}
    >
      {sceneEnabled ? (
        <SceneErrorBoundary>
          <Suspense fallback={<SceneFallback />}>
            <AmbientScene motionEnabled quality={effectiveProfile} />
          </Suspense>
        </SceneErrorBoundary>
      ) : (
        <SceneFallback />
      )}

      <aside className="side-nav">
        <div className="brand" aria-label="LIMEN">
          <span className="brand__halo" aria-hidden="true">
            <i />
          </span>
          <span className="brand__copy">
            <strong>LIMEN</strong>
            <small>HOME</small>
          </span>
        </div>
        <nav aria-label="Ecosistema LIMEN">
          {navigation.map((item) => (
            <FocusButton
              key={item.id}
              focusId={`nav-${item.id}`}
              focused={focusedId === `nav-${item.id}`}
              className={`side-nav__item ${view === item.id ? "is-active" : ""}`}
              onClick={() => chooseView(item.id)}
              aria-current={view === item.id ? "page" : undefined}
            >
              <Icon name={item.icon} />
              <span>{item.label}</span>
            </FocusButton>
          ))}
        </nav>
        <FocusButton
          focusId="profile"
          focused={focusedId === "profile"}
          className="profile-card"
          onClick={() =>
            showNotice(
              "Perfil local Diego · sincronización de cuenta prevista para M3.",
            )
          }
        >
          <span>DV</span>
          <div>
            <strong>Diego</strong>
            <small>Perfil local</small>
          </div>
        </FocusButton>
      </aside>

      <header className="top-bar">
        <div>
          <span className="top-bar__context">LIMEN HOME</span>
          <strong>{viewTitles[view]}</strong>
        </div>
        <div className="system-status">
          <FocusButton
            focusId="status-search"
            focused={focusedId === "status-search"}
            className="status-button"
            onClick={() =>
              showNotice(
                "La búsqueda global llegará con el índice local de M2.",
              )
            }
            aria-label="Buscar"
          >
            <Icon name="search" />
          </FocusButton>
          <FocusButton
            focusId="status-wifi"
            focused={focusedId === "status-wifi"}
            className="status-button"
            onClick={() =>
              showNotice(
                "Red disponible · LIMEN no ha realizado conexiones externas.",
              )
            }
            aria-label="Estado de red"
          >
            <Icon name="wifi" />
          </FocusButton>
          <FocusButton
            focusId="status-battery"
            focused={focusedId === "status-battery"}
            className="status-button status-button--battery"
            onClick={() =>
              showNotice(
                "Batería simulada en la vista web · la lectura del sistema llegará con Tauri.",
              )
            }
            aria-label="Batería al 100%"
          >
            <Icon name="battery" />
            <span>100%</span>
          </FocusButton>
          <FocusButton
            focusId="status-controller"
            focused={focusedId === "status-controller"}
            className={`status-button controller-button ${controllerConnected ? "is-connected" : ""}`}
            onClick={() =>
              showNotice(
                controllerConnected
                  ? "Mando detectado · A seleccionar · B volver."
                  : "Conecta un mando y pulsa cualquier botón para activarlo.",
              )
            }
            aria-label={
              controllerConnected ? "Mando conectado" : "Mando no detectado"
            }
          >
            <Icon name="gamepad" />
            <i />
          </FocusButton>
        </div>
      </header>

      <main className="main-content">
        {view === "home" && (
          <HomeView
            featured={featured}
            featuredIndex={featuredIndex}
            focusedId={focusedId}
            onPlay={playNotice}
            onDetails={openFeatured}
            onSlide={setFeaturedIndex}
            onChooseGame={openGame}
          />
        )}
        {view === "library" && (
          <LibraryView
            focusedId={focusedId}
            filter={libraryFilter}
            onFilter={setLibraryFilter}
            onChooseGame={openGame}
            onSearch={() =>
              showNotice("Búsqueda: prevista para el índice local de M2.")
            }
          />
        )}
        {view === "discover" && (
          <DiscoverView
            focusedId={focusedId}
            onChooseGame={openGame}
            onNotice={showNotice}
          />
        )}
        {view === "community" && (
          <CommunityView focusedId={focusedId} onNotice={showNotice} />
        )}
        {view === "apps" && (
          <AppsView focusedId={focusedId} onNotice={showNotice} />
        )}
        {view === "add" && (
          <AddView
            focusedId={focusedId}
            selectedSource={selectedSource}
            onSelectSource={setSelectedSource}
            onNotice={showNotice}
          />
        )}
        {view === "settings" && (
          <SettingsView
            focusedId={focusedId}
            motionEnabled={motionEnabled}
            highContrast={highContrast}
            graphicsPreference={graphicsPreference}
            effectiveProfile={sceneEnabled ? effectiveProfile : "2d"}
            viewportMetrics={viewportMetrics}
            benchmark={benchmark}
            benchmarkRunning={benchmarkRunning}
            onToggleMotion={() => setMotionEnabled((enabled) => !enabled)}
            onToggleContrast={() => setHighContrast((enabled) => !enabled)}
            onSelectGraphics={setGraphicsPreference}
            onRunBenchmark={() => void runBenchmark()}
            onNotice={showNotice}
          />
        )}
        {view === "detail" && (
          <DetailView
            game={selectedGame}
            focusedId={focusedId}
            onPlay={playNotice}
            onNotice={showNotice}
          />
        )}
      </main>

      <footer className="control-footer">
        {inputMethod === "gamepad" ? (
          <>
            <ControllerHint glyph="A">Seleccionar</ControllerHint>
            <ControllerHint glyph="B">Atrás</ControllerHint>
            <ControllerHint glyph={<Icon name="menu" />}>Menú</ControllerHint>
          </>
        ) : inputMethod === "pointer" ? (
          <>
            <ControllerHint glyph="●">Clic</ControllerHint>
            <ControllerHint glyph="Esc">Atrás</ControllerHint>
            <ControllerHint glyph="M">Menú</ControllerHint>
          </>
        ) : (
          <>
            <ControllerHint glyph="↵">Seleccionar</ControllerHint>
            <ControllerHint glyph="Esc">Atrás</ControllerHint>
            <ControllerHint glyph="M">Menú</ControllerHint>
          </>
        )}
        <span>M1 · PERFIL {sceneEnabled ? effectiveProfile : "2D"}</span>
      </footer>

      {notice && (
        <button
          className="notice"
          type="button"
          onClick={() => setNotice(null)}
          role="status"
          aria-live="polite"
        >
          <Icon name="check" />
          <span>{notice}</span>
          <small>Cerrar</small>
        </button>
      )}
    </div>
  );
}

# LIMEN — Roadmap verificable

Estado: propuesta 0.1
Estrategia: demostrar una experiencia estrecha y excelente antes de ampliar sistemas o funciones.

## Reglas del roadmap

- Un hito no comienza hasta cumplir la salida del anterior.
- No se desarrollan Home, Overlay, comunidad y múltiples Bridges a la vez.
- Cada hito termina con una demo reproducible y criterios binarios.
- PCSX2, juegos y BIOS son dependencias externas proporcionadas por el usuario.
- SteamOS es objetivo arquitectónico desde el inicio, pero la primera integración real se estabiliza en Windows.
- Si un spike invalida una decisión, se actualiza `DECISIONS.md` antes de continuar.

## M0 — Definición y decisión tecnológica

Estado: **completado — 2026-08-07**

Entregables:

- [x] `SPEC.md`.
- [x] `ARCHITECTURE.md`.
- [x] `ROADMAP.md`.
- [x] `DECISIONS.md`.
- [x] `AGENTS.md`.
- [x] Elegir **A — Rust/Tauri/React** en D-001.
- [x] Elegir un monorepo con varios procesos y paquetes en D-008.
- [x] Decidir protocolo local inicial en D-002.
- [x] Acordar Apache-2.0 como licencia del código propio en D-006.
- [x] Inicializar estructura y toolchain de la opción elegida.

Salida:

- El propietario del producto registra explícitamente las decisiones pendientes.
- No quedan contradicciones entre especificación y arquitectura.

## M1 — UI-01: prueba visual controller-first

Estado: **en curso**

Objetivo: demostrar que el renderer elegido puede sentirse como una consola antes de construir runtimes.

Incluye:

- Home basada en el punto medio de las referencias visuales.
- Hero de Final Fantasy X con arte de prueba propio o placeholder.
- Continuar jugando y recientes.
- 100–200 fichas simuladas y virtualizadas.
- Motor de foco espacial.
- Entrada de mando mediante backend temporal o simulado.
- Transiciones, fondo dinámico, calidad escalable y glifos adaptativos.
- Perfil 2D/reduced-motion completamente funcional.

Pruebas:

- 3840×2160/60 fps en un PC capaz definido para la prueba.
- 1920×1080/60 fps en el hardware portátil de referencia.
- Sesión de navegación de 15 minutos sin perder foco.
- Pulsaciones rápidas y direcciones mantenidas no desincronizan UI/estado.
- Desconectar y reconectar mando no obliga a usar ratón.

Salida:

- La demo puede manejarse de principio a fin con mando.
- Existe un informe con frame times, memoria y degradaciones aplicadas.
- Si falla, se evalúa la siguiente opción de D-001 antes de construir Core.

## M2 — CORE-01: contrato y sesión simulada

Objetivo: separar físicamente Home de Core y probar el ciclo de vida sin emulador.

Incluye:

- Proceso Core independiente.
- API local v1 con autenticación efímera.
- Snapshot de Home y stream de eventos.
- Máquina de estados de sesión.
- Bridge falso que lanza un proceso de prueba controlado.
- Reconexión después de reiniciar Home.
- Persistencia mínima y logs estructurados.
- Runtime Console mínima como cliente de solo lectura: salud de módulos, línea temporal de la sesión y eventos sanitizados reales del Core.

Pruebas:

- Tests de contrato y compatibilidad de versiones.
- Core sigue supervisando si Home termina.
- Crash, timeout, cancelación y salida normal tienen resultados distintos.
- Ningún launch pasa por PowerShell, `cmd`, Bash o interpolación de shell.

Salida:

- Home puede cambiarse por un cliente CLI de prueba y la sesión sigue funcionando.
- La Runtime Console puede observar la misma sesión simulada sin recibir privilegios de orquestación.

## M3 — PS2-01: auditoría del Bridge PCSX2

Objetivo: comprobar el control real de PCSX2 antes de conectarlo a la experiencia final.

Incluye:

- Selección manual del ejecutable oficial.
- Detección y allowlist de versiones probadas.
- Auditoría de argumentos `no GUI`, `batch` y `fullscreen` de esa versión.
- Investigación del directorio de configuración aislado.
- Validación de juego y BIOS sin copiarlos.
- Lanzador de diagnóstico fuera de Home.
- Salida ordenada y escalada por timeout.

Pruebas negativas:

- Primera ejecución sin configuración.
- BIOS ausente.
- Ruta con espacios y caracteres Unicode.
- Versión desconocida.
- Crash del runtime.
- Diálogo inesperado o ventana principal visible.

Salida:

- Una matriz documenta exactamente qué versión y flags cumplen el recorrido.
- No se automatizan clics para esconder incompatibilidades.

## M4 — VERTICAL-01: Final Fantasy X de extremo a extremo

Objetivo: cumplir la promesa central del prototipo 0.1.

Incluye:

- Configuración del juego mediante datos locales del usuario.
- Home → Core → Bridge PS2 → PCSX2.
- Ocultación segura de Home durante la sesión.
- Juego fullscreen sin GUI de PCSX2.
- Combinación de salida accesible.
- Regreso a Home con foco y resultado de sesión.

Salida:

- Se cumplen los ocho criterios de aceptación de `SPEC.md`.
- La demo se repite tres veces desde arranque limpio sin intervención técnica.
- Un fallo deliberado vuelve a Home con explicación y diagnóstico exportable.

## M5 — HARDEN-01: recuperación y paquete Windows

Objetivo: convertir el vertical slice en una prueba distribuible sin ampliar alcance.

Incluye:

- Instalador o paquete de desarrollo de LIMEN; no incluye PCSX2 ni contenido protegido.
- Migraciones y rutas de datos por usuario.
- Recuperación tras crash/reinicio.
- Firma y actualización se diseñan, aunque la actualización automática puede posponerse.
- Matriz de Windows 11, escalado, monitores y mandos.
- Auditoría de logs y datos personales.

Salida:

- Otro equipo puede instalar LIMEN, señalar sus dependencias legítimas y reproducir la demo siguiendo una guía corta.

## M6 — PORT-01: SteamOS/Linux

Objetivo: validar las abstracciones de plataforma en hardware real bajo Gamescope/Wayland.

Incluye:

- Core y Home nativos para Linux; no se da por suficiente ejecutar la build Windows bajo Proton.
- Backend SDL3 y evaluación de Steam Input.
- IPC mediante Unix domain socket.
- Supervisión de procesos Linux.
- Paquete compatible con el modelo inmutable de SteamOS; Flatpak/AppImage se decide mediante spike.
- PCSX2 Linux como runtime externo seleccionado por el usuario.

Pruebas:

- Game Mode y Desktop Mode.
- Suspensión/reanudación.
- Teclado en pantalla para campos imprescindibles.
- Cambio de mando y dock/undock.
- Foco y retorno bajo Gamescope.

Salida:

- El mismo contrato y dominio ejecutan el vertical slice sin ramas de lógica de producto específicas para Linux.

## M7 — INPUT-01 y OVERLAY-01

Objetivo: convertir Input en servicio completo y validar un overlay real.

Incluye:

- Jugadores, perfiles, batería, vibración y hotplug.
- UI propia de emparejamiento hasta el límite permitido por el sistema.
- Overlay en proceso separado.
- Reanudar, salir y funciones que el Bridge PS2 demuestre controlar.
- Degradación segura si fullscreen exclusivo impide el overlay.

Salida:

- Desconexiones y cambios de jugador se resuelven sin abrir PCSX2.
- Overlay funciona en la matriz soportada o declara con precisión sus limitaciones.

## M8 — LIBRETRO-01

Objetivo: alojar un core sencillo y probar el modelo unificado sin empezar por PS2.

Incluye:

- Host Libretro aislado.
- Vídeo, audio, input, timing y cierre.
- Un core homebrew o de contenido libre para pruebas.
- Mismo modelo de sesión, Input y Vault que un runtime externo.

Salida:

- Home no necesita saber si la sesión usa Libretro o un proceso externo.

## M9 — ATLAS-01 y VAULT-01

Objetivo: identificación reproducible y datos seguros.

Incluye:

- Escaneo opt-in.
- Huellas, ediciones y overrides.
- Guardados, estados y backups por runtime.
- Importación/exportación local.
- Sincronización remota solo después de resolver proveedor, cifrado y conflictos.

## M10 — COMMUNITY-01

Objetivo: instalación comunitaria declarativa, verificable y reversible.

Incluye:

- Esquema de receta.
- Permisos y raíces concedidas.
- Verificación, descarga, backup, parche, rollback y registro.
- Moderación/firma como capas de confianza, no como sustituto del sandbox.
- UI de las referencias de instalación comunitaria.

No incluye:

- Scripts arbitrarios de un clic.
- ROMs, BIOS, firmware o bypass de DRM.
- Catálogo público antes de superar threat model y auditoría.

## Después del prototipo

Solo después de M4/M5 se priorizan Bridges adicionales. El orden se decide por madurez de control silencioso, no por popularidad. PS3/PS4 no se consideran hasta que la supervisión de runtimes externos, Overlay y Vault sean fiables.

## Riesgos que pueden detener una línea de trabajo

| Riesgo | Comprobación temprana | Respuesta |
|---|---|---|
| Renderer no sostiene UI rica | M1 | Cambiar renderer sin tocar Core. |
| WebView Linux diverge | M1/M6 | Baseline WebGL2, fallback 2D o opción nativa. |
| PCSX2 muestra UI inevitable | M3 | Fijar versión compatible o revisar el Bridge; no ocultar con automatización frágil. |
| Overlay falla en exclusivo | M7 | Borderless recomendado o funciones fuera del overlay. |
| SteamOS inmutable impide integración | M6 | Paquete soportado, portales y permisos explícitos. |
| Recetas amplían privilegios | M10 | Reducir operaciones permitidas; no lanzar catálogo. |

# LIMEN — Registro de decisiones

Estado: las decisiones abiertas requieren confirmación del propietario del producto antes de implementar.

## Resumen

| ID | Decisión | Estado | Recomendación |
|---|---|---|---|
| D-001 | Pila de Core y Home | **ACEPTADA — 2026-08-07** | Opción A |
| D-002 | Protocolo local | **ACEPTADA — 2026-08-07** | JSON tipado sobre IPC local |
| D-003 | Backends de mando | PROPUESTA | GameInput + SDL3 |
| D-004 | Modelo de runtime PS2 | ACEPTADA POR ESPECIFICACIÓN | PCSX2 oficial externo mediante Bridge |
| D-005 | Baseline gráfico web | **ACEPTADA CON D-001** | WebGL2; WebGPU opcional |
| D-006 | Licencia de LIMEN | **ACEPTADA — 2026-08-07** | Apache-2.0 para el código propio |
| D-007 | Empaquetado SteamOS | POSPUESTA A M6 | Spike Flatpak/AppImage |
| D-008 | Organización del código | **ACEPTADA — 2026-08-07** | Un monorepo, varios procesos y paquetes |

## D-001 — Pila inicial

Estado: **aceptada el 7 de agosto de 2026 por el propietario del producto**.

Decisión: **A — Rust + Tauri 2 + React/TypeScript**.

### Criterios

- Windows 11 y SteamOS/Linux.
- UI cinematográfica 2D/3D a 60 fps.
- Iteración visual rápida.
- Integración nativa de procesos, mandos, almacenamiento y ventanas.
- Core separado y renderer reemplazable.
- Coste de mantenimiento y licencias razonable.

### Opción A — Rust + Tauri 2 + React/TypeScript

```text
LIMEN Core: Rust, runtime async, SQLite
API local: contrato versionado independiente de Tauri
Home host: Tauri 2
UI: React + TypeScript + Motion
3D: Three.js / React Three Fiber
Input: puerto Rust; GameInput en Windows, SDL3 en SteamOS
```

Ventajas:

- Es la ruta más rápida para reproducir las referencias: layout, vídeo, tipografía, blur, animaciones y 3D tienen un ecosistema maduro.
- Rust encaja bien con procesos, concurrencia, seguridad de memoria y un Core portable.
- Tauri mantiene una frontera natural entre WebView y host nativo y permite que la mayor parte del sistema no viva en JavaScript.
- La UI puede sustituirse sin reescribir dominio, Bridges o persistencia si respetamos la API local.
- Licencias base permisivas y binario de host pequeño al reutilizar el WebView del sistema.

Riesgos:

- Tauri usa WebView2 en Windows y WebKitGTK en Linux; no son el mismo motor y pueden diferir en CSS, codecs y GPU.
- WebGPU no debe asumirse idéntico en SteamOS. Una escena crítica basada solo en WebGPU elevaría el riesgo.
- Overlays, foco entre procesos y fullscreen siguen necesitando código nativo; Tauri no los resuelve automáticamente.
- Una UI web sin disciplina puede consumir memoria, recargar assets o introducir navegación de teclado impropia de consola.

Mitigaciones obligatorias:

- WebGL2 como renderer 3D mínimo; WebGPU solo mediante detección de capacidad.
- Core en proceso independiente; no convertir comandos Tauri en el contrato de dominio.
- Presupuesto de frame, memoria y assets desde M1.
- Prueba en WebView2 y WebKitGTK/Gamescope antes de ampliar funciones.
- Fallback 2D completo y `prefers-reduced-motion`/modo de bajo consumo.

### Opción B — .NET 10 + Avalonia 12

```text
LIMEN Core: C#/.NET, SQLite
API local: contrato versionado
Home: Avalonia XAML + Skia
3D: superficie GPU/custom composition cuando sea necesaria
Input: GameInput interop en Windows, SDL3 en SteamOS
```

Ventajas:

- Excelente productividad para servicios, modelos de estado, tooling y Windows.
- Avalonia usa Win32 directamente en Windows y ofrece Linux/X11; el backend Wayland nativo existe, aunque su estado debe validarse.
- Renderizado consistente con Skia, controles accesibles y una sola plataforma de desarrollo.
- Interop de APIs Windows más directo para un equipo cómodo con C#.

Riesgos:

- La dirección 2D es viable, pero el 3D cinematográfico exige más trabajo propio o integración de una superficie GPU.
- Menor ecosistema de componentes para motion/3D de este estilo que React/Three.js.
- El backend Wayland nativo de Avalonia se documenta como experimental; XWayland sería la base inicial más conservadora.
- Es fácil acoplar UI, servicios y estado dentro de un solo proceso .NET si no se impone la separación.

Cuándo elegirla:

- Si se priorizan tooling, Windows y una base C# uniforme sobre la velocidad de iteración visual 3D.

### Opción C — C++20 + Qt 6/QML/Qt Quick 3D

```text
LIMEN Core: C++20 o servicio Rust separado
Home: Qt Quick/QML
3D: Qt Quick 3D
Gráficos: Qt RHI (D3D/Vulkan/OpenGL)
Input: SDL3 + adaptador GameInput
Build: CMake
```

Ventajas:

- Mayor control nativo y un scene graph acelerado diseñado para UI.
- Qt Quick mezcla 2D y 3D y abstrae Direct3D, Vulkan y OpenGL.
- Plataforma madura para Windows/Linux y buena ruta hacia un dispositivo embebido.
- Mejor posición de partida si LIMEN acaba necesitando un compositor profundamente nativo.

Riesgos:

- Curva y coste de desarrollo mayores para un proyecto pequeño.
- QML/C++ exige más disciplina de ownership, threading y tooling.
- Qt tiene modelo dual comercial/LGPL/GPL, con obligaciones de distribución que deben revisarse antes de adoptar módulos.
- La iteración y disponibilidad de perfiles web/TypeScript suele ser menor que en la opción A.

Cuándo elegirla:

- Si el control de render y ventana nativos pesa más que la velocidad de prototipado y se acepta su coste/licencia.

### Comparación

Puntuación orientativa: 1 deficiente, 5 excelente para este proyecto y esta fase.

| Criterio | A: Rust/Tauri/React | B: .NET/Avalonia | C: C++/Qt Quick |
|---|---:|---:|---:|
| Velocidad de prototipo visual | 5 | 3 | 3 |
| Ecosistema 2D + 3D | 5 | 3 | 5 |
| Integración Windows | 4 | 5 | 5 |
| Ruta SteamOS | 3 | 4 | 4 |
| Control gráfico/ventanas | 3 | 3 | 5 |
| Seguridad y robustez de Core | 5 | 4 | 3 |
| Facilidad para sustituir Home | 5 | 5 | 5 |
| Simplicidad de licencias base | 5 | 5 | 3 |
| Coste para un equipo pequeño | 5 | 4 | 2 |
| **Total** | **40** | **36** | **35** |

La puntuación de SteamOS para A no es una condena a la tecnología: refleja que el WebView de Linux debe verificarse pronto y que el overlay no será web. La arquitectura separada contiene ese riesgo.

### Decisión y condiciones

Se elige **A: Rust + Tauri 2 + React/TypeScript**, con cuatro condiciones no negociables:

1. Tauri aloja Home; no aloja conceptualmente todo LIMEN.
2. Core vive en proceso propio y expone una API que otro renderer pueda consumir.
3. WebGL2 es el baseline; WebGPU es una mejora opcional.
4. M1 es una prueba de descarte: si no cumple 4K/60 en PC capaz y 1080p/60 en el portátil de referencia con 100–200 fichas, se revisa B/C antes de construir Bridges.

Registro: `D-001 = A — Rust/Tauri/React`.

## D-002 — Protocolo local

Estado: **aceptada el 7 de agosto de 2026 por el propietario del producto**.

Decisión: mensajes JSON UTF-8 tipados y validados por JSON Schema sobre IPC local del sistema.

- Named pipe por usuario en Windows.
- Unix domain socket con permisos de usuario en Linux.
- Frames con longitud `u32` little-endian seguida del payload JSON, con límite inicial de 1 MiB.
- Conexión bidireccional para comandos/consultas y conexión dedicada para la suscripción de eventos.
- Handshake inicial con `api_major`, capacidades del cliente y secreto efímero entregado por el host.
- `schemas/v1` será la fuente neutral; los tipos Rust y TypeScript se validarán contra ella.
- Todo mensaje lleva `message_version`; las peticiones, `request_id`; y los eventos de sesión, `session_id` y secuencia.

### Opción recomendada

La opción aceptada parte de mensajes JSON tipados y validados por esquema sobre transporte local del sistema:

- Named pipe en Windows.
- Unix domain socket en Linux.
- Canal separado o multiplexado para eventos.
- `api_major`, `message_version`, `request_id`, `session_id` y secuencia de evento.

Ventajas: inspeccionable durante el prototipo, independiente de lenguaje y sin puerto TCP. El contrato podrá migrar a Protobuf si las mediciones lo justifican.

El límite y el framing evitan depender de fronteras de lectura del transporte. Una versión mayor desconocida, un frame sobredimensionado, un JSON inválido o un handshake no autenticado fallan cerrados.

Alternativa: Protobuf/gRPC desde el inicio. Aporta IDL y streaming maduros, pero complica named pipes, el bridge hacia una WebView y el debugging temprano.

Rechazado: usar únicamente `invoke`/eventos de Tauri. Acoplaría Core al host visual e impediría que sobreviva a Home.

## D-003 — Input

Estado: propuesta.

Decisión recomendada:

- Modelo canónico propio de acciones, jugadores y capacidades.
- GameInput v1 como backend Windows.
- SDL3 Gamepad como backend SteamOS/Linux y fallback portable.
- Steam Input como integración opcional, nunca como modelo de dominio.

Justificación: GameInput ofrece callbacks, baja latencia, haptics y dispositivos en Windows; SDL3 normaliza gamepads, hotplug y mappings en varias plataformas.

## D-004 — PCSX2 oficial externo primero

Estado: aceptada por la especificación.

El vertical slice usa PCSX2 oficial como proceso externo mediante Bridge. No usa LRPS2 como atajo y no enlaza código PCSX2 dentro de Core.

Consecuencia: la primera prueba demuestra supervisión de un runtime moderno real. Libretro llega después con un core de contenido libre y sencillo.

## D-005 — Baseline gráfico

Estado: **aceptada como condición de D-001**.

- DOM/CSS para texto, navegación, accesibilidad y paneles.
- Motion/Web Animations para transiciones 2D.
- Three.js con WebGL2 para escena espacial.
- WebGPU solo si una capability probe y benchmarks por plataforma lo aprueban.
- Ninguna acción esencial existe únicamente dentro del canvas 3D.

## D-006 — Licencia de LIMEN

Estado: **aceptada el 7 de agosto de 2026 por el propietario del producto**.

Decisión: el código y la documentación originales de LIMEN se publican bajo **Apache License 2.0**, salvo que un archivo indique expresamente otra licencia compatible.

- Cada dependencia conserva su propia licencia y debe entrar en el inventario legal antes de distribuir binarios.
- La licencia de LIMEN no cubre ROMs, BIOS, firmware, juegos, arte comercial, emuladores ni otros componentes externos.
- Ejecutar un runtime externo mediante un Bridge no autoriza a redistribuirlo.
- Los paquetes y binarios deberán incluir los avisos exigidos por sus dependencias cuando comience M5.

Debe elegirse antes de publicar binarios o adoptar dependencias con obligaciones recíprocas. Alternativas iniciales:

- Proyecto abierto permisivo (Apache-2.0/MIT).
- Core abierto y servicios/catálogos separados.
- Producto propietario con inventario legal completo.

Esta decisión afecta especialmente a Libretro/RetroArch, PCSX2 y cualquier distribución de binarios. Usar una API o ejecutar un proceso no concede automáticamente derecho a redistribuirlo.

## D-007 — Paquete SteamOS

Estado: pospuesta a M6.

SteamOS usa un sistema base inmutable y Valve recomienda instalar aplicaciones adicionales mediante Flatpak. LIMEN necesita, además, lanzar runtimes y acceder a bibliotecas elegidas por el usuario; por eso Flatpak, AppImage y un posible helper con permisos deben compararse mediante un threat model y una prueba real.

## D-008 — Monorepo con límites internos

Estado: **aceptada el 7 de agosto de 2026**.

Todo el código propio de LIMEN comienza en un único repositorio. Esto no convierte LIMEN en un proceso monolítico: el repositorio contiene varios ejecutables y librerías con dependencias dirigidas.

Razones:

- Home, Core, contratos y el primer Bridge evolucionarán juntos durante el vertical slice.
- Un cambio de contrato puede actualizar productor, cliente generado y tests en un solo commit atómico.
- Una sola CI puede comprobar Rust, TypeScript, contratos y el recorrido integrado.
- Evita versionar y publicar paquetes privados antes de que sus fronteras sean estables.
- Simplifica el trabajo de un equipo pequeño sin renunciar a separar procesos.

Reglas:

- `apps/home-ui` no depende de crates de Core ni contiene llamadas de sistema.
- `apps/home-host` es un adaptador Tauri fino; no contiene reglas de producto.
- `services/core` es el único ejecutable autoritativo de dominio.
- Los módulos Rust se separan en crates según responsabilidad, no en repositorios.
- Los contratos son la dependencia compartida; Core nunca depende de Home.
- Las carpetas futuras no se crean hasta que su hito empiece.

Solo se considerará dividir repositorios cuando exista una razón operativa real:

- SDK público con ciclo de releases independiente.
- Catálogo comunitario con permisos, moderación y colaboradores distintos.
- Servicio cloud desplegado independientemente.
- Assets de gran tamaño con distribución/licencia propia.
- Fork de un proyecto externo que deba mantener su historia y licencia.

Los emuladores y el contenido del usuario nunca se incorporan al monorepo.

## Fuentes primarias de la comparación

- [Tauri: modelo de procesos y WebViews por plataforma](https://v2.tauri.app/concept/process-model/)
- [Avalonia: Windows](https://docs.avaloniaui.net/docs/platform-specific-guides/windows)
- [Avalonia: Linux, X11 y estado de Wayland](https://docs.avaloniaui.net/docs/platform-specific-guides/linux)
- [Avalonia: renderizado personalizado e interop GPU](https://docs.avaloniaui.net/docs/graphics-animation/custom-rendering)
- [Qt Quick Scene Graph](https://doc.qt.io/qt-6/qtquick-visualcanvas-scenegraph.html)
- [Qt Quick 3D](https://doc.qt.io/qt-6/qtquick3d-index.html)
- [Licencias de Qt](https://doc.qt.io/qt-6/licensing.html)
- [GameInput](https://learn.microsoft.com/en-us/gaming/gdk/docs/features/common/input/overviews/input-overview)
- [SDL3 Gamepad](https://wiki.libsdl.org/SDL3/CategoryGamepad)
- [SteamOS/Steam Deck FAQ](https://partner.steamgames.com/doc/steamdeck/faq?l=english)

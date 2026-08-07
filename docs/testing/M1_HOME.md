# M1 Home — guía de prueba

Estado: candidato de cierre técnico; listo para la aceptación física en el
dispositivo de referencia.

## Ejecutar en navegador

Desde la raíz del repositorio:

```powershell
corepack pnpm install
corepack pnpm dev
```

Abre `http://127.0.0.1:1420/`. La URL solo escucha en el equipo local.

Controles disponibles:

- Flechas o WASD: mover el foco.
- `Enter` o espacio: aceptar.
- `Escape` o retroceso: volver.
- Mando estándar: cruceta/stick izquierdo, botón sur para aceptar y botón este para volver.

La Gamepad API del navegador empieza a exponer un mando después de pulsar un botón por primera vez. El indicador superior cambia de **Teclado** a **Mando** cuando se detecta.

En **Configuración → Gráficos** se puede seleccionar Auto, Calidad,
Equilibrado, Rendimiento o Solo 2D. La misma pantalla permite ejecutar una
medición local de diez segundos. No se envían datos ni se conserva información
del hardware.

## Ejecutar como ventana Tauri

Requiere Rust estable y las dependencias de sistema de Tauri:

```powershell
corepack pnpm tauri:dev
```

En este punto Home Host es deliberadamente fino: solo crea la ventana. El IPC con Core comienza en M2.

## Comprobaciones automáticas

```powershell
corepack pnpm format:check
corepack pnpm lint
corepack pnpm typecheck
corepack pnpm test
corepack pnpm build
python tools/check_repository.py
```

## Evidencia automática

- Build web de producción correcta.
- Dieciséis pruebas unitarias sobre datos simulados, navegación espacial,
  virtualización, perfiles gráficos, frame times y entrada temporal de mando.
- 160 fichas simuladas con nombres y arte originales; no se incluye arte comercial.
- Biblioteca virtualizada: a 1920×1080 se montan normalmente 35 fichas de 160;
  una fila fuera de pantalla puede mantenerse temporalmente para restaurar el
  foco exacto.
- Escena Three.js/R3F animada en tiempo real, con parallax por puntero y por foco de teclado/mando.
- Calidad automática según píxeles renderizados, DPR acotado y reducción de
  geometría/antialias en perfiles inferiores.
- Fallback raster original disponible para selección 2D, preferencia de
  movimiento reducido o fallo de WebGL/React.
- Simulación determinista de quince minutos a 60 Hz con direcciones mantenidas
  y una desconexión/reconexión por minuto.
- Aceptar y volver se procesan por flanco; las direcciones mantenidas usan
  retardo y cadencia estables.

## Evidencia visual y de interacción

- Recorrido de teclado y mando comprobado entre Inicio, Biblioteca, Descubrir, Comunidad, Aplicaciones, Añadir contenido, Configuración y Ficha del juego.
- Todos los elementos que parecen accionables responden; las funciones de M2/M3 muestran de forma explícita que aún no modifican archivos ni lanzan procesos.
- La paleta de navegación usa azul/cian/violeta; el verde se reserva para estados semánticos positivos.
- Acción Jugar muestra el límite honesto del prototipo; no simula una sesión inexistente.
- Biblioteca completa accesible y desplazable; se recorrieron 120 posiciones y
  el retorno desde la ficha restauró exactamente `library-game-121`.
- Inspección visual manual a 3840×2160, 1920×1080 y 1145×918, sin desbordamiento
  del documento.
- Perfil Solo 2D verificado sin ningún canvas WebGL montado.
- Sin errores ni advertencias en consola durante el recorrido comprobado.

## Informe de rendimiento — 2026-08-07

Equipo de desarrollo usado para la medición: Intel Core i5-9400F, NVIDIA
GeForce GTX 1650 y 15,9 GB de RAM. Cada muestra usa `requestAnimationFrame`
durante diez segundos con la aplicación visible.

| Resolución | Perfil Auto aplicado | FPS medio | Frame P95 | Heap JS |
| --- | --- | ---: | ---: | ---: |
| 1920×1080 | Calidad | 59,6 | 16,8 ms | 34,4 MB |
| 3840×2160 | Rendimiento | 60,0 | 16,8 ms | 33,6 MB |

Degradaciones del perfil Rendimiento: DPR máximo 1, antialias desactivado,
menos figuras secundarias, portal de 32 segmentos, trazos de 24×4 segmentos y
grano reducido. Si WebGL falla o el usuario solicita movimiento reducido, se
usa el fondo 2D original y se desmonta el canvas.

Estas cifras prueban el presupuesto en el PC indicado, no en cualquier equipo.
La aceptación física final consiste en ejecutar la misma medición a
1920×1080 en la ROG Xbox Ally de referencia y completar un recorrido real con
el mando, incluida una desconexión/reconexión. El propietario ya puede hacerlo
desde Configuración sin herramientas de desarrollo.

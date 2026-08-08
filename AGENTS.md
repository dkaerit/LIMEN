# AGENTS.md — Reglas de trabajo para LIMEN

Este archivo se aplica a todo el repositorio. Todo agente o colaborador debe leer `SPEC.md`, `ARCHITECTURE.md`, `ROADMAP.md` y `DECISIONS.md` antes de modificar el proyecto.

## 1. Estado actual

El proyecto está en **M2 — CORE-01: contrato y sesión simulada**.

**D-001, D-002, D-006 y D-008 están resueltas:** Rust + Tauri 2 + React/TypeScript, JSON tipado sobre IPC local, Apache-2.0 y un monorepo con varios procesos. M1 materializó Home, su host fino y paquetes visuales. M2 materializa contratos, dominio, sesión, Bridge falso, Core y la Runtime Console mínima.

## 2. Objetivo inmutable

LIMEN debe permitir seleccionar un juego, ejecutarlo y volver a Home sin mostrar escritorio, terminal, carpetas ni GUI del runtime. Optimizar una parte no justifica romper esa experiencia.

## 3. Invariantes arquitectónicos

- Home es reemplazable y nunca es la fuente de verdad.
- Core es un proceso independiente y no depende de React, Tauri, Avalonia, Qt u otro renderer.
- Home solo usa la API local versionada para operaciones de dominio.
- Solo Core/Session Manager inicia y supervisa procesos.
- Cada runtime se integra mediante un Bridge de capacidades explícitas.
- Input emite acciones semánticas y asignaciones de jugador; la UI no interpreta directamente botones físicos.
- Atlas identifica y recomienda; no lanza.
- Vault posee guardados, configuración y backups.
- Overlay es un cliente separado del Core.
- Un frontend Libretro se ejecutará aislado del proceso Core.
- Todo el código propio comienza en un monorepo; compartir repositorio no permite saltarse límites entre procesos o paquetes.

No se puede relajar una de estas reglas sin una decisión registrada y aprobación del propietario.

## 4. Alcance del primer prototipo

El único recorrido real inicial es:

```text
Home → Final Fantasy X → Bridge PS2 → PCSX2 oficial sin GUI
→ fullscreen → salir → volver al mismo elemento de Home
```

No añadir más sistemas, tiendas, cuentas, cloud, mods, scraping general, Overlay completo o catálogo comunitario hasta superar M4.

## 5. Seguridad, legalidad y privacidad

- Nunca descargar, crear, copiar, inspeccionar o añadir al repositorio ROMs, ISOs, BIOS, firmware, claves, tickets o contenido propietario.
- No incluir carátulas, logos o material comercial salvo que exista permiso/licencia documentado. Los tests visuales usan placeholders o assets propios.
- No descargar ni redistribuir emuladores sin autorización explícita y revisión de licencia. Para pruebas, el usuario selecciona una instalación externa.
- No registrar rutas personales completas, tokens, correos, nombres de usuario o identificadores de hardware en fixtures, snapshots o issues.
- Nunca ejecutar texto comunitario como shell. Las futuras recetas son manifiestos declarativos con operaciones allowlisted.
- Normalizar rutas y comprobar que permanecen bajo raíces autorizadas antes de escribir, mover, extraer o borrar.
- Toda modificación de configuración externa requiere backup, registro y rollback.
- Lanzar procesos con ejecutable y lista de argumentos; nunca concatenar una línea para `cmd`, PowerShell o Bash.
- Ninguna UI web recibe secretos permanentes o acceso general al sistema de archivos.

## 6. Flujo de trabajo

1. Identificar el hito activo y sus criterios de salida.
2. Leer las decisiones relacionadas y comprobar que no estén abiertas.
3. Proponer la unidad más pequeña que produzca evidencia.
4. Implementar primero puertos/contratos y después adaptadores.
5. Añadir tests proporcionales al riesgo.
6. Ejecutar las verificaciones relevantes en Windows; Linux/SteamOS cuando el hito lo requiera.
7. Actualizar documentación y ADR si cambió una suposición.
8. Entregar cambios en commits pequeños, intencionales y fáciles de revertir.

No mezclar refactors amplios con una función. No reescribir trabajo existente si una modificación localizada basta.

## 7. Git y archivos

- No hacer commits hasta que Git tenga nombre/correo configurados por el usuario.
- No reescribir historia, forzar push ni borrar cambios ajenos.
- Un commit debe representar una sola intención y pasar sus tests.
- Antes del primer código, crear `.gitignore` y defensas para datos locales según la estructura elegida.
- Nunca usar patrones de ignore tan amplios que oculten código o fixtures legítimos sin revisión.
- Binarios, cachés, perfiles locales, logs, capturas, juegos y firmware quedan fuera de Git.
- Los ejemplos de configuración usan rutas ficticias y portables.

## 8. Contratos y compatibilidad

- Los cambios incompatibles requieren nueva versión mayor de API y migración documentada.
- Todo mensaje incluye identificador y versión; los eventos de sesión incluyen secuencia.
- Clientes desconocidos fallan cerrados con explicación.
- Un Bridge fija rangos/versiones probados del runtime. No se declara compatibilidad basándose solo en que el proceso arrancó.
- Las capacidades ausentes se representan como ausentes; no se simulan con automatización de teclado/ratón frágil.
- Los formatos de terceros se aíslan dentro del Bridge correspondiente.

## 9. UI y mando

- Cada recorrido se prueba sin ratón.
- El foco actual siempre es visible, estable y restaurable.
- Aceptar/volver se modelan por posición/acción y luego se muestran con el glifo del dispositivo.
- Las animaciones nunca bloquean el estado ni descartan silenciosamente entradas.
- Texto e información esencial permanecen en una capa accesible; el canvas 3D es progresivo.
- Toda pantalla rica tiene modo de movimiento reducido y calidad baja.
- Las listas grandes se virtualizan y las imágenes se cargan/cancelan según visibilidad.
- No afirmar 60 fps sin registrar hardware, resolución, perfil, percentiles de frame time y memoria.

## 10. Tests mínimos por área

### Core

- Máquina de estados, idempotencia, timeouts y recuperación.
- Validación de esquema y compatibilidad de API.
- Home desconectado durante una sesión.

### Bridge

- Versiones soportadas y rechazadas.
- Paths Unicode, espacios y raíces no permitidas.
- Golden plan con `argv` estructurado.
- Falta de firmware/contenido y diálogo inesperado.
- Salida normal, crash y proceso colgado.

### Input

- Hotplug y reconexión.
- Dead zones y repetición.
- Cambio de glifos y asignación de jugador.
- Backend virtual para tests deterministas.

### Home

- Navegación espacial y restauración de foco.
- 100–200 fichas.
- Pulsaciones rápidas durante transiciones.
- Fallback 2D y movimiento reducido.
- Capturas visuales a 16:9 y 16:10.

### Recetas futuras

- Traversal, symlinks, zip bombs, hashes incorrectos, rollback parcial y denegación de permisos.

## 11. Observabilidad

- Logs estructurados con `request_id`, `session_id`, módulo y código de error.
- Separar mensajes para usuario de detalles técnicos.
- Redactar rutas y secretos antes de exportar.
- No guardar input crudo ni telemetría personal por defecto.
- Runtime Console observa eventos; no obtiene privilegios para alterar Core salvo comandos de diagnóstico explícitos.

## 12. Acciones que requieren aprobación explícita

- Resolver o cambiar D-001/D-002/D-006.
- Añadir una dependencia de runtime o un framework principal.
- Descargar/ejecutar un emulador o acceder a contenido del usuario.
- Escribir fuera de los directorios propios de LIMEN.
- Añadir red, cuentas, telemetría o servicios cloud.
- Habilitar scripts o código comunitario.
- Sustituir Explorer, activar modo kiosco o modificar configuración global del sistema.
- Publicar paquetes, repositorios o releases.

## 13. Definición de terminado

Una tarea no está terminada porque compile. Debe:

- Cumplir el criterio del hito.
- Respetar los invariantes.
- Tener tests o evidencia reproducible.
- Fallar de manera segura y comprensible.
- No introducir material prohibido ni datos personales.
- Actualizar la documentación afectada.
- Indicar limitaciones conocidas sin ocultarlas.

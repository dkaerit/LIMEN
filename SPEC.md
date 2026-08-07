# LIMEN — Especificación de producto

Estado: borrador 0.1
Fase: implementación M1, prueba visual controller-first
Plataforma inicial: Windows 11
Plataforma objetivo posterior: SteamOS/Linux

## 1. Definición

LIMEN es un entorno universal de ejecución de videojuegos que ofrece la sencillez y continuidad de una consola sobre un PC. Reúne juegos y aplicaciones compatibles en una biblioteca manejable con mando, traduce la configuración necesaria para cada runtime y mantiene ocultas las interfaces de los emuladores o ejecutores subyacentes.

LIMEN no es un emulador ni una tienda de ROMs. Es la capa de producto situada entre la persona, su contenido legítimo y los runtimes que lo ejecutan.

La promesa principal es:

> Seleccionar un juego, jugar y volver a LIMEN sin ver el escritorio, carpetas, terminales ni la interfaz del runtime.

## 2. Principios de producto

1. **Controller-first.** Toda experiencia normal debe poder completarse con un mando.
2. **Una biblioteca, distintos runtimes.** La procedencia técnica del juego no debe dominar la experiencia.
3. **El runtime es invisible.** PCSX2, Dolphin, RPCS3 u otro ejecutor aportan ejecución; LIMEN aporta la interacción y la configuración.
4. **El usuario conserva el control.** LIMEN no adquiere, copia ni bloquea contenido sin consentimiento explícito.
5. **Operaciones reversibles.** Toda modificación a configuraciones o contenido debe tener previsualización, copia de seguridad y rollback cuando corresponda.
6. **La interfaz es reemplazable.** LIMEN Home representa el estado de LIMEN Core, pero Core nunca depende de Home.
7. **Local y offline por defecto.** Jugar a contenido ya configurado no debe requerir una cuenta ni conexión a Internet.
8. **Errores comprensibles.** Nunca se debe dejar al usuario ante una ventana técnica sin explicación o ruta de recuperación.

## 3. Qué es y qué no es

### Es

- Un shell visual de biblioteca y sesiones de juego.
- Un servicio local que identifica contenido, prepara configuraciones, lanza runtimes y supervisa su ciclo de vida.
- Una capa común de entrada, perfiles, guardados, diagnósticos y, más adelante, instalación comunitaria.
- Un frontend propio para cores Libretro y un supervisor de runtimes externos cuando no exista un core adecuado.

### No es

- Un sistema operativo en la primera etapa.
- Una modificación de Windows o SteamOS.
- Un distribuidor de ROMs, BIOS, firmware, claves o archivos protegidos.
- Una promesa de compatibilidad universal desde el primer lanzamiento.
- Una forma de reempaquetar PCSX2 u otros proyectos ignorando sus licencias.
- Una plataforma que ejecute recetas comunitarias arbitrarias con permisos ilimitados.

## 4. Usuarios y contextos

### Usuario principal

Una persona que juega en un PC de salón o una portátil tipo ROG Xbox Ally/Steam Deck y quiere una experiencia de consola sin gestionar manualmente launchers, archivos de configuración y menús distintos.

### Contextos objetivo

- Pantalla integrada de 7–8 pulgadas, 16:9 o 16:10.
- Televisor o monitor a 1080p, 1440p o 4K.
- Uso con mando Xbox, PlayStation, Nintendo/8BitDo o mando genérico compatible.
- Teclado y ratón reservados para accesibilidad, recuperación y modo desarrollador.

## 5. Vocabulario

- **Home:** interfaz principal sustituible.
- **Core:** servicio local sin interfaz que conserva el estado y orquesta el sistema.
- **Bridge:** adaptador que traduce una intención de LIMEN a capacidades, configuración y ciclo de vida de un runtime concreto.
- **Runtime:** core Libretro, emulador oficial externo, juego de PC o aplicación que ejecuta contenido.
- **Input:** servicio que normaliza dispositivos físicos, acciones y jugadores.
- **Atlas:** identificación de juegos, metadatos y perfiles de compatibilidad.
- **Vault:** guardados, capturas, configuraciones y copias de seguridad.
- **Overlay:** interfaz durante una sesión de juego.
- **Recipe/receta:** descripción declarativa y auditable de una instalación o transformación comunitaria.
- **Runtime Console:** vista de diagnóstico para desarrolladores; nunca forma parte del recorrido normal.

## 6. Experiencia objetivo

### 6.1 Inicio y biblioteca

Al abrir LIMEN, Home recupera del Core el usuario local, la biblioteca, el juego seleccionado, las sesiones recientes y el estado de dispositivos. La pantalla es inmediatamente navegable con mando y conserva el último foco válido.

Home ofrece, como mínimo:

- Inicio.
- Biblioteca.
- Añadir juegos.
- Ajustes.
- Estado del perfil, red, batería y mando cuando el sistema lo permita.
- Juego destacado.
- Continuar jugando.
- Recientes.

### 6.2 Dirección visual

La dirección oficial parte del punto medio entre las dos primeras referencias de Home:

- Lectura clara de una interfaz moderna: jerarquía, título, plataforma, estado y acción **JUGAR**.
- Espacio tridimensional inspirado en la era PS2 sin copiar su identidad: niebla azul, profundidad, cubos suspendidos, órbitas y estelas luminosas.
- Una fila principal de juegos integrada en el escenario; se evita tanto una cuadrícula genérica tipo streaming como una escena 3D que dificulte encontrar contenido.
- Cristal, agua, reflejos y luz como lenguaje propio de LIMEN y de la idea de umbral.
- Tipografía legible, alto contraste y foco inequívoco incluso en una pantalla pequeña.
- Iconos y glifos originales; no se copian logotipos, sonidos ni elementos protegidos de Xbox o PlayStation.

El sistema visual debe poder bajar calidad dinámicamente: menos partículas, vídeo desactivado, blur simplificado y escena 2D de respaldo, sin perder funcionalidad.

### 6.3 Navegación con mando

- El foco se mueve espacialmente y nunca se pierde.
- La acción principal usa la posición sur del mando; volver usa la posición este. Los glifos reflejan el dispositivo activo.
- Mantener una dirección aplica repetición con retraso y aceleración definidos.
- Cada pantalla recuerda el foco al volver.
- Las animaciones no bloquean entradas rápidas ni ponen el estado visual y lógico en desacuerdo.
- Cambio de dispositivo, desconexión y reconexión producen una notificación LIMEN.
- Debe existir una ruta de recuperación con teclado y ratón.

### 6.4 Inicio de un juego

1. El usuario activa **JUGAR**.
2. Home envía una intención de inicio al Core; no construye comandos ni toca archivos.
3. Core valida juego, runtime, BIOS/firmware cuando aplique, permisos y perfil.
4. El Bridge genera una configuración de sesión y un plan de lanzamiento auditable.
5. Core oculta Home, inicia el runtime directamente en el juego y supervisa el proceso.
6. No aparece la GUI del runtime, el escritorio ni una terminal.
7. Al terminar, Core registra el resultado y Home reaparece en el mismo juego.

### 6.5 Durante el juego

La primera versión solo necesita una combinación segura para salir y regresar a Home. LIMEN Overlay llegará después y ofrecerá, según las capacidades reales del Bridge:

- Reanudar.
- Mandos y jugadores.
- Guardar/cargar estado.
- Capturas.
- Rendimiento.
- Ajustes compatibles con aplicación en caliente.
- Salir a Home.

Una opción solo se muestra si el runtime declara esa capacidad; LIMEN no fingirá control que no posee.

### 6.6 Comunidad y aplicaciones externas

Las referencias visuales de comunidad, permisos e instalación representan una fase futura. El objetivo es permitir recetas declarativas, perfiles, traducciones y aplicaciones externas con:

- Autor, versión, compatibilidad y tamaño visibles.
- Lista exacta de acciones y permisos.
- Verificación de versión y hashes.
- Copia de seguridad previa.
- Progreso por pasos.
- Resultado verificable y desinstalación reversible.
- Aislamiento del proceso cuando la plataforma lo permita.

Esta función no forma parte del prototipo 0.1.

## 7. Requisitos funcionales

### Home

- **HOME-001:** Home consume exclusivamente la API local versionada del Core para operaciones de dominio.
- **HOME-002:** La biblioteca completa es navegable sin ratón.
- **HOME-003:** Cerrar o reiniciar Home no detiene ni corrompe una sesión ya controlada por Core.
- **HOME-004:** La capa 3D es decorativa/progresiva; la información y navegación siguen disponibles si falla.
- **HOME-005:** Los glifos cambian según el último dispositivo activo sin provocar saltos de layout.

### Core y sesiones

- **CORE-001:** Core funciona sin Home y mantiene una única fuente de verdad para biblioteca y sesiones.
- **CORE-002:** Solo puede existir una sesión de juego activa por usuario en 0.1.
- **CORE-003:** Cada sesión tiene identificador, estados explícitos, tiempos, runtime, Bridge y resultado.
- **CORE-004:** Core sobrevive a la caída de Home y detecta la salida inesperada del runtime.
- **CORE-005:** Ningún argumento de proceso se construye mediante concatenación de comandos de shell.

### Bridges y runtimes

- **BRIDGE-001:** Cada Bridge declara versión, plataformas, runtime compatible y capacidades.
- **BRIDGE-002:** El Bridge valida antes de lanzar y devuelve errores estructurados.
- **BRIDGE-003:** La configuración se genera por capas y no modifica silenciosamente configuraciones ajenas.
- **BRIDGE-004:** Un runtime externo se inicia sin su biblioteca, menús ni setup wizard visibles.
- **BRIDGE-005:** Si no puede garantizarse el modo silencioso para una versión concreta, el Bridge la marca como incompatible.

### Input

- **INPUT-001:** Home recibe acciones semánticas (`navigate_left`, `accept`, `back`), no botones físicos.
- **INPUT-002:** Input admite hotplug, reasignación de jugador y perfiles por dispositivo.
- **INPUT-003:** Windows usa un backend nativo con GameInput cuando esté disponible; SteamOS usa un backend portátil compatible con su ecosistema.
- **INPUT-004:** El primer emparejamiento Bluetooth puede mostrar el consentimiento obligatorio del sistema y debe explicarse antes de solicitarlo.

### Atlas y Vault

- **ATLAS-001:** La identificación nunca depende solo del nombre del archivo.
- **ATLAS-002:** Un perfil indica procedencia, versión y nivel de confianza.
- **VAULT-001:** Guardados, ajustes y copias de seguridad se separan por usuario, juego y runtime.
- **VAULT-002:** Los datos personales y rutas locales nunca se guardan en Git.

### Legal y seguridad

- **SAFE-001:** LIMEN no descarga ROMs, BIOS, firmware ni claves propietarias.
- **SAFE-002:** El usuario proporciona contenido y dumps obtenidos legalmente.
- **SAFE-003:** Toda descarga permitida muestra procedencia, licencia, tamaño y hash cuando exista.
- **SAFE-004:** Las recetas comunitarias no ejecutan código arbitrario por defecto.
- **SAFE-005:** Logs exportables eliminan tokens, nombres de usuario y rutas personales cuando sea posible.

## 8. Prototipo 0.1: vertical slice PS2

### Alcance exacto

```text
LIMEN Home
  → Final Fantasy X configurado por el usuario
  → LIMEN Core
  → Bridge PS2
  → PCSX2 oficial sin GUI
  → juego a pantalla completa
  → salida controlada
  → regreso al mismo elemento de Home
```

El título se usa como caso de prueba; ni el juego, ni su arte comercial, ni BIOS, ni PCSX2 se incorporan al repositorio.

### Prerrequisitos del usuario

- Copia legal del juego en una ubicación externa al repositorio.
- Dump legal de BIOS cuando sea necesario.
- Binario oficial de PCSX2 instalado o seleccionado por el usuario.
- Un mando compatible o teclado para recuperación.

### Incluido

- Una pantalla Home funcional con datos simulados y un juego configurable.
- Navegación completa con mando.
- Core ejecutado como proceso separado de Home.
- Contrato local versionado.
- Bridge PS2 mínimo.
- Validación de rutas y versión de PCSX2.
- Perfil aislado de configuración de PCSX2 para LIMEN.
- Inicio `no GUI + batch + fullscreen`, sujeto a validación contra la versión soportada.
- Supervisión, salida y retorno a Home.
- Logs de diagnóstico estructurados.

### No incluido

- Overlay completo.
- Frontend Libretro.
- Descarga o actualización automática de PCSX2.
- Escaneo general de biblioteca.
- Atlas remoto.
- Sincronización cloud.
- Mods, traducciones o recetas comunitarias.
- PS3, PS4, Switch u otros sistemas.
- Sustitución de Explorer o modo kiosco.

### Criterios de aceptación

1. Desde un arranque limpio, la persona puede iniciar y terminar la sesión sin usar ratón.
2. No aparece la ventana principal, biblioteca, menú o setup wizard de PCSX2.
3. No aparece consola ni escritorio durante la transición normal.
4. Si falta juego, BIOS, runtime o configuración, LIMEN no lanza y muestra una corrección concreta.
5. Tras salida normal o crash del runtime, Home vuelve y recupera el foco en Final Fantasy X.
6. Reiniciar Home durante el juego no mata la sesión; al volver, consulta su estado al Core.
7. Los procesos de LIMEN no aceptan rutas mediante interpolación de shell.
8. El repositorio no contiene ROMs, BIOS, firmware, carátulas comerciales, datos personales ni binarios del emulador.

## 9. Objetivos no funcionales

- **Rendimiento de Home:** 60 fps como objetivo; 1080p en portátil de entrada y 4K en hardware capaz mediante perfiles de calidad.
- **Prueba de estrés visual:** 100–200 juegos simulados sin pérdida de foco ni cargas visibles al navegar.
- **Respuesta:** confirmación visual inmediata; las tareas lentas son asíncronas y cancelables cuando sea seguro.
- **Memoria:** imágenes virtualizadas, decodificadas al tamaño necesario y con caché acotada.
- **Disponibilidad:** Core recupera estado después de un cierre inesperado.
- **Portabilidad:** lógica de dominio y contratos sin dependencias de Windows; adaptadores de plataforma en los bordes.
- **Accesibilidad:** escala de texto, reducción de movimiento, contraste alto, subtítulos para información sonora y ruta alternativa a efectos 3D.
- **Idioma inicial:** español, con arquitectura preparada para internacionalización.

## 10. Fronteras conocidas

- En Windows de escritorio, el primer emparejamiento Bluetooth siempre puede mostrar un diálogo del sistema para consentimiento.
- Shell Launcher solo es una opción posterior en ediciones compatibles y no impide por sí solo acceder a otros componentes de Windows.
- Un overlay fiable sobre fullscreen exclusivo es dependiente de plataforma y runtime; debe validarse, no asumirse.
- Un runtime externo puede cambiar argumentos o formato de configuración entre versiones. Cada Bridge debe fijar y probar versiones compatibles.
- El soporte SteamOS requiere pruebas reales bajo Gamescope/Wayland y un modelo de distribución compatible con el sistema inmutable.

## 11. Puerta de decisión actual

**D-001, D-002, D-006 y D-008 están resueltas:** LIMEN usa Rust + Tauri 2 + React/TypeScript, JSON tipado sobre IPC local, Apache-2.0 y un monorepo con varios procesos y paquetes. M1 valida Home antes de construir Core.

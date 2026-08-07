# LIMEN

[![CI](https://github.com/dkaerit/LIMEN/actions/workflows/ci.yml/badge.svg)](https://github.com/dkaerit/LIMEN/actions/workflows/ci.yml)

LIMEN es un entorno universal de ejecución de videojuegos orientado a mando. Su objetivo es ofrecer una experiencia de consola sobre Windows y SteamOS: seleccionar un juego, ejecutarlo mediante el runtime adecuado y volver a Home sin mostrar escritorios, terminales ni interfaces técnicas.

## Estado

LIMEN está en **M1: prueba visual controller-first**. La pila inicial ya está decidida:

- Core y servicios nativos: Rust.
- Home: Tauri 2 + React + TypeScript.
- Presentación 3D: Three.js mediante React Three Fiber, con WebGL2 como baseline y fallback estático.
- Organización: monorepo con varios procesos y paquetes.

La primera Home ejecutable usa datos simulados y una escena 3D animada en tiempo real para validar composición, foco, mando y rendimiento antes de construir Core.

## Documentación principal

- [Especificación de producto](SPEC.md)
- [Arquitectura y límites de procesos](ARCHITECTURE.md)
- [Roadmap verificable](ROADMAP.md)
- [Registro de decisiones](DECISIONS.md)
- [Reglas para agentes y colaboradores](AGENTS.md)
- [Guía de contribución](CONTRIBUTING.md)
- [Política de seguridad](SECURITY.md)

## Primer vertical slice

```text
LIMEN Home
  → LIMEN Core
  → Bridge PS2
  → PCSX2 oficial sin GUI
  → juego a pantalla completa
  → regreso a LIMEN Home
```

El usuario proporciona por separado cualquier runtime, juego, BIOS o firmware que posea legalmente. Este repositorio no contiene ni distribuye ROMs, imágenes de disco, BIOS, firmware, claves, emuladores o arte comercial.

## Ejecutar la Home de desarrollo

Requisitos: Node.js 20.19 o posterior y Corepack.

```powershell
corepack pnpm install
corepack pnpm dev
```

La terminal mostrará una URL local. La interfaz admite flechas/WASD, `Enter`/espacio para aceptar, `Escape` para volver y un mando mediante la Gamepad API del navegador.

La ventana Tauri se podrá ejecutar con `corepack pnpm tauri:dev` después de instalar Rust; el frontend web permite probar el diseño sin ese requisito.

La guía completa y la evidencia de la iteración están en [M1 Home — guía de prueba](docs/testing/M1_HOME.md).

## Comprobaciones

```powershell
python tools/check_repository.py
```

La CI comprueba automáticamente formato, análisis estático, pruebas y compilación de Node, Rust y Tauri para las plataformas objetivo.

## Licencia

El código y la documentación originales de LIMEN se distribuyen bajo [Apache License 2.0](LICENSE). Los juegos, runtimes, firmware, arte y demás componentes externos conservan sus propias licencias y no forman parte de este repositorio.

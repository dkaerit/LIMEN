# LIMEN

[![CI](https://github.com/dkaerit/LIMEN/actions/workflows/ci.yml/badge.svg)](https://github.com/dkaerit/LIMEN/actions/workflows/ci.yml)

LIMEN es un entorno universal de ejecución de videojuegos orientado a mando. Su objetivo es ofrecer una experiencia de consola sobre Windows y SteamOS: seleccionar un juego, ejecutarlo mediante el runtime adecuado y volver a Home sin mostrar escritorios, terminales ni interfaces técnicas.

## Estado

LIMEN está en **M0: definición y preparación del repositorio**. La pila inicial ya está decidida:

- Core y servicios nativos: Rust.
- Home: Tauri 2 + React + TypeScript.
- Presentación 3D: Three.js, con WebGL2 como baseline.
- Organización: monorepo con varios procesos y paquetes.

Todavía no existe una aplicación ejecutable. Antes de los hitos funcionales se están fijando contratos, límites de seguridad, colaboración y CI.

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

## Comprobación disponible hoy

```powershell
python tools/check_repository.py
```

La CI detectará los workspaces Rust, Node y Tauri cuando se creen y activará automáticamente formato, análisis estático, pruebas y compilación para las plataformas objetivo.

## Licencia

El modelo de licencia todavía está abierto en D-006. Hasta que se publique una licencia explícita, no se concede permiso para copiar, modificar o redistribuir el contenido del repositorio.

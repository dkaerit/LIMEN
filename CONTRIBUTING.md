# Contribuir a LIMEN

Gracias por ayudar a construir LIMEN. El proyecto todavía está en fase de arquitectura: una contribución debe respetar los límites de producto antes de ampliar funcionalidad.

## Antes de cambiar código

1. Lee `SPEC.md`, `ARCHITECTURE.md`, `ROADMAP.md`, `DECISIONS.md` y `AGENTS.md`.
2. Comprueba el hito activo y que las decisiones que lo bloquean estén cerradas.
3. Mantén Home, Core, Bridges y adaptadores de plataforma en sus límites definidos.
4. No introduzcas un sistema, runtime o servicio cloud fuera del hito actual.

## Flujo de ramas

- `main` debe permanecer verificable.
- Usa ramas breves como `feature/...`, `fix/...` o `docs/...`.
- Los agentes automatizados usan `agent/...`.
- Evita mezclar refactors generales con una función.
- Prefiere commits pequeños con una intención clara.

## Comprobaciones

Disponible desde la fase documental:

```powershell
python tools/check_repository.py
```

Cuando existan los workspaces correspondientes, la CI exigirá:

```text
pnpm run format:check
pnpm run lint
pnpm run typecheck
pnpm run test (si existe)
pnpm run build

cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo build --workspace --all-features
```

Si existe una aplicación Tauri, también se compilará sin empaquetar releases en Windows y Linux.

## Pull requests

Una PR debe explicar:

- Qué cambia y por qué.
- Qué hito o decisión cubre.
- Cómo se ha validado.
- Qué riesgos o limitaciones conserva.
- Capturas o vídeo si cambia Home u Overlay.

Las PRs no deben incluir artefactos generados, dependencias instaladas, datos locales ni contenido protegido.

## Contenido prohibido

No subas:

- ROMs, ISOs, CHD, BIOS, firmware, claves o tickets.
- Ejecutables o binarios de emuladores.
- Guardados, capturas o perfiles personales.
- Tokens, credenciales, certificados privados o archivos `.env` reales.
- Arte, logos, audio o vídeo comercial sin licencia documentada.

Los tests de emulación utilizarán contenido homebrew/libre o rutas externas proporcionadas por cada desarrollador.

## Seguridad

No publiques vulnerabilidades o datos sensibles en una issue normal. Sigue `SECURITY.md`.

# M1 Home — guía de prueba

Estado: primer vertical visual ejecutable; M1 todavía está en curso.

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

## Evidencia de esta iteración

- Build web de producción correcta.
- Seis pruebas unitarias sobre biblioteca simulada y navegación espacial.
- 160 fichas simuladas sin arte comercial incluido.
- Recorrido de teclado comprobado: **Jugar → primera ficha → ficha derecha**.
- Acción Jugar muestra el límite honesto del prototipo; no simula una sesión inexistente.
- Biblioteca completa accesible y desplazable.
- Inspección visual manual a 1920×1080 y 1280×800.
- Sin errores ni advertencias en consola durante el recorrido comprobado.

Todavía no se afirma un objetivo de 60 fps: faltan la medición prolongada, el hardware de referencia y la virtualización explícita de la biblioteca exigidas para cerrar M1.

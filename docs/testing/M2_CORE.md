# M2 Core — contrato y sesión simulada

Estado: primer checkpoint en implementación.

## Objetivo

Separar el estado autoritativo de Home y demostrar el ciclo de vida con un
proceso controlado que no accede a juegos, firmware, emuladores ni rutas del
usuario.

Este checkpoint materializa únicamente las fronteras necesarias:

- `schemas/v1`: fuente neutral del contrato local.
- `crates/domain`: identificadores y estados puros.
- `crates/contracts`: envelopes, compatibilidad y secreto efímero redactado.
- `crates/session`: máquina de estados determinista y eventos secuenciados.
- `crates/bridge-sdk`: capacidades y `LaunchPlan` con ejecutable absoluto,
  `argv` y entorno separados.
- `crates/bridge-fake`: plan de lanzamiento del runtime falso de M2.
- `services/core`: fuente de verdad, supervisor y ejecutables de simulación.

## Ejecutar

Requiere Rust estable 1.85 o posterior:

```powershell
cargo test --workspace --all-features
cargo build --workspace --all-features
cargo run -p limen-core --bin limen-core -- --self-check
```

También se pueden observar resultados diferenciados:

```powershell
cargo run -p limen-core --bin limen-core -- --simulate normal
cargo run -p limen-core --bin limen-core -- --simulate crash
cargo run -p limen-core --bin limen-core -- --simulate timeout
```

La salida del Core es una línea JSON estructurada sin rutas completas. El
runtime falso es otro proceso del workspace y solo acepta `--mode` y un
identificador portable de juego placeholder.

## Propiedades verificadas

- Una única sesión activa.
- Secuencias monotónicas y replay desde una secuencia conocida.
- Un nuevo cliente obtiene el snapshot autoritativo sin que la sesión dependa
  de Home.
- Salida normal, crash, timeout y cancelación producen resultados distintos.
- Los planes rechazan ejecutables relativos y no exponen argumentos en Debug.
- El proceso se inicia con `std::process::Command`, ejecutable absoluto y lista
  de argumentos; nunca mediante intérprete de shell.
- Un proceso colgado o abandonado se termina y se recolecta.
- Identificadores con traversal, separadores de ruta o tamaño excesivo se
  rechazan antes de entrar al dominio.

## Siguiente checkpoint

Todavía faltan el framing JSON real, transporte por named pipe/Unix socket,
handshake autenticado, persistencia mínima y la Runtime Console de solo lectura.
Esos adaptadores se implementarán sobre los contratos actuales; no se añadirá
una dependencia de runtime sin aprobación explícita del propietario.

# Política de seguridad

## Versiones soportadas

LIMEN todavía no tiene releases. Solo la rama `main` recibe correcciones de seguridad durante la fase previa al prototipo.

## Informar de una vulnerabilidad

Usa de forma privada **Security → Report a vulnerability** en GitHub cuando esté disponible. Si esa opción no aparece, contacta al propietario del repositorio por un canal privado; no abras una issue pública con detalles explotables, secretos o rutas personales.

Incluye, cuando sea posible:

- Componente y revisión afectada.
- Impacto esperado.
- Pasos mínimos de reproducción sin contenido protegido.
- Mitigación conocida.
- Si el informe contiene datos que deban eliminarse.

## Áreas especialmente sensibles

- Escape de rutas al extraer o modificar archivos.
- Ejecución de comandos o recetas no confiables.
- Inyección en argumentos de runtimes externos.
- Acceso excesivo a bibliotecas, guardados o credenciales.
- IPC local sin autenticación o validación de versión.
- Logs que expongan tokens, nombres de usuario o rutas personales.
- Elevación de privilegios, persistencia o modificación del shell del sistema.

## Contenido protegido

No adjuntes ROMs, BIOS, firmware, claves, guardados personales o binarios de terceros a un informe. Usa fixtures sintéticos y hashes redactados.

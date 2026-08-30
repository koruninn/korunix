# Korunix

Korunix es una capa de producto sobre NixOS para administrar el equipo mediante decisiones humanas. Nix conserva la fuente declarativa de verdad y el motor Rust se ocupa únicamente de las operaciones del sistema vivo.

## Instalar después de NixOS

La ruta remota y la ruta local usan el mismo bootstrap. No sustituyen `/etc/nixos`: primero inspeccionan la instalación gráfica existente, preparan la adopción y conservan la configuración de origen.

```sh
nix run github:koruninn/korunix#bootstrap
```

Desde una copia llevada en almacenamiento externo:

```sh
nix run path:/ruta/a/korunix#bootstrap
```

El bootstrap instala en `~/.korunix`. Si ya existe una instalación, conserva `configuracion/`, `generado/` y el historial Git, valida un candidato completo y sustituye el producto mediante un intercambio atómico. Si hay cambios locales en el código del producto, se niega a sobrescribirlos silenciosamente.

## Arquitecturas

Las salidas de producto se publican para `x86_64-linux` y `aarch64-linux`. La detección de bootstrap acepta x86_64 y AArch64 y rechaza expresamente cualquier arquitectura distinta.

## Sin conexión

Korunix es offline-first. Consultar y modificar configuración local, personas, apariencia, escritorios, copias, historial, recuperación, almacenamiento y dispositivos no se bloquea porque falte Internet. Actualizar fuentes remotas o metadatos de firmware sí puede necesitar red. `korunix product --json` muestra esta frontera.

Una copia local del flake permite instalar o actualizar el propio producto desde USB u otro almacenamiento con `nix run path:...#bootstrap`, siempre que Nix tenga disponibles las dependencias necesarias.

## Desarrollo y lanzamiento

`just os` mantiene el flujo cotidiano de aplicación. Antes de publicar una versión se ejecuta una sola puerta integral:

```sh
./scripts/validar-lanzamiento.sh
```

La puerta comprueba formato, Rust, Nix, empaquetado, metadatos de escritorio, arquitecturas, privacidad del source empaquetado y contratos automatizables de robustez. Los escenarios que requieren una instalación gráfica limpia o hardware físico se registran en `LANZAMIENTO.md`; no se sustituyen por una simulación engañosa.

## Configuración y privacidad

Las decisiones humanas viven en `configuracion/`. El estado local sensible y los respaldos viven bajo `XDG_STATE_HOME` o `~/.local/state/korunix`. El source empaquetado excluye `configuracion/equipos`, `configuracion/personas` y `generado/equipos`, así que la distribución no arrastra la configuración personal de quien la construyó.

Korunix no envía telemetría por defecto y las copias portables normales no incluyen credenciales.

# Korunix

Korunix es una configuración de NixOS para varios equipos. La idea es simple:
**cada cosa tiene un lugar claro y el sistema se puede volver a construir sin adivinar qué archivo manda.**

## Si quieres cambiar algo, mira aquí

- **Datos del equipo:** `equipos/korunix/equipo.nix`
- **Aplicaciones normales:** `aplicaciones/paquetes.nix`
- **Noctalia, barra, dock y widgets:** `apariencia/escritorios/noctalia/ajustes.nix`
- **Plugins de Noctalia:** `apariencia/escritorios/noctalia/plugins.nix`
- **Atajos y comportamiento de Umbriel:** `apariencia/escritorios/noctalia/umbriel.nix`
- **Servicios propios de Korunix:** `equipos/korunix/servicios/`

## Archivos que normalmente NO necesitas tocar

- `flake.lock`: Nix lo actualiza.
- `hardware.nix`: describe el hardware detectado.
- `apariencia/escritorios/noctalia/noctalia.nix`: es el motor que convierte
  `ajustes.nix` a TOML y limpia overrides viejos.
- `apariencia/fondos.nix` y `apariencia/escritorios/noctalia/fondos/`: son el
  sistema y la colección de fondos; no se cambian durante estas limpiezas.

## Comandos fáciles

```fish
just revisar
just probar korunix
just korunix
just optiplex
just volver
```

Si cambias algo en la interfaz de Noctalia y quieres conservarlo para siempre,
**decláralo en `ajustes.nix`**. La GUI puede guardar cambios temporales, pero en
un rebuild Korunix vuelve a ser la fuente de verdad.

El grupo `input` se conserva intencionalmente: lo usan Bongo Cat y los mandos de
Xbox. La configuración de red se mantiene separada y no forma parte de estas
limpiezas.

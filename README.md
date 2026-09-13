# Korunix

Configuración declarativa de NixOS para varios equipos, con una base común y capas separadas para aplicaciones, apariencia y decisiones propias de cada máquina.

## Estructura

- `equipos/`: configuración concreta de cada máquina. El nombre de la carpeta se usa también como hostname.
- `modulos/base/`: comportamiento común a todos los equipos, como audio, energía y herramientas de mantenimiento.
- `aplicaciones/`: aplicaciones e integraciones que usa el equipo Korunix.
- `apariencia/`: tema, cursor, escritorios, Noctalia Greeter y configuración visual.
- `plantilla-equipo/`: punto de partida para añadir una máquina nueva.
- `flake.nix`: descubre automáticamente las carpetas dentro de `equipos/` y construye cada configuración con su canal y arquitectura.

## Datos de un equipo

Cada `equipos/<nombre>/equipo.nix` declara como mínimo:

```nix
{
  canal = "stable"; # o "unstable"
  arquitectura = "x86_64-linux";
  persona = "usuario";
}
```

Algunos equipos pueden añadir más datos, por ejemplo una pantalla:

```nix
pantalla = {
  nombre = "DP-1";
  modo = "1920x1080@120";
  escala = 1.0;
};
```

El flake falla con un mensaje claro si falta `canal`, `arquitectura` o `persona`, si alguno está vacío, si el canal no es `stable`/`unstable` o si una pantalla declarada está incompleta.

## Comandos

Validar todas las configuraciones descubiertas:

```bash
just check
```

Construir una generación completa sin activarla:

```bash
just build korunix
```

Probar una configuración sin convertirla en el arranque predeterminado:

```bash
just test korunix
```

Aplicar una configuración:

```bash
just rebuild korunix
```

También existen los atajos:

```bash
just korunix
just optiplex
```

Volver a la generación anterior del equipo actual:

```bash
just rollback
```

Comprobar o aplicar formato Nix:

```bash
just format-check
just format
```

Actualizar `flake.lock`:

```bash
just update
```

Limpiar generaciones antiguas y optimizar el almacén de Nix:

```bash
just clean
```

## Añadir un equipo

1. Copiar `plantilla-equipo/` a `equipos/<nombre>/`.
2. Ajustar `equipo.nix` con canal, arquitectura y persona.
3. Sustituir `hardware.nix` por la configuración generada para la máquina real.
4. Revisar `system.stateVersion` al crear la máquina. Debe representar la versión con la que nació esa instalación y no incrementarse simplemente por actualizar NixOS.
5. Añadir únicamente los servicios y escritorios que necesite ese equipo.
6. Ejecutar `just check`, `just build <nombre>` y después `just test <nombre>` antes del primer `switch`.

## Principios actuales

- Una decisión debe tener una sola fuente de verdad siempre que sea posible.
- Los módulos reutilizables deben importar por sí mismos el soporte externo que necesitan.
- Las preferencias manuales del usuario deben poder prevalecer sobre los valores predeterminados del sistema cuando corresponda.
- X11 se conserva como compatibilidad para aplicaciones mediante XWayland; las sesiones de escritorio nuevas se priorizan en Wayland.
- `system.stateVersion` pertenece a cada máquina y no a la base común.

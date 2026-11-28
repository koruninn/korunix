# Korunix

Esta es la configuración de NixOS de Veskalia, mi ASUS ROG Ally (RC71L) con AMD Ryzen Z1 Extreme.

Aquí tengo todo lo que necesito para reinstalar el sistema sin tener que configurar todo de nuevo.

## ¿Qué es NixOS?

NixOS es una distribución de Linux que permite configurar el sistema mediante archivos `.nix`. En ellos se definen los programas, servicios, usuarios, controladores y demás ajustes del equipo.

Esto permite reproducir una configuración desde cero o volver a una anterior si algo deja de funcionar.

### Flakes

Los flakes permiten reunir una configuración de Nix y sus dependencias en un mismo proyecto. En `flake.nix` se definen las dependencias y en `flake.lock` se guardan sus versiones exactas.

### Sin Home Manager

Home Manager permite gestionar los programas y las configuraciones de cada usuario de forma declarativa.

Korunix no lo utiliza. Toda la configuración se gestiona desde los propios módulos de NixOS.

## Escritorios

Desde la pantalla de inicio de sesión puedes elegir entre tres sesiones.

### Umbriel + Noctalia

Umbriel es un compositor Wayland que organiza las ventanas en columnas desplazables.

Noctalia se encarga de los paneles, el dock, las notificaciones, los controles del sistema y la apariencia del escritorio. Luciole es el explorador de archivos.

| Claro | Oscuro |
| --- | --- |
| ![Umbriel claro](capturas/umbriel/umbriel-claro.png) | ![Umbriel oscuro](capturas/umbriel/umbriel-oscuro.png) |

### GNOME

Un entorno de escritorio con soporte para gestos táctiles, teclado en pantalla y extensiones.

Comparte los colores, los fondos de pantalla y buena parte de la apariencia de Noctalia. Incluye GSConnect para conectar el celular desde GNOME.

| Claro | Oscuro |
| --- | --- |
| ![GNOME claro](capturas/gnome/gnome-claro.png) | ![GNOME oscuro](capturas/gnome/gnome-oscuro.png) |

### Modo consola

Una sesión de Steam Big Picture sobre Gamescope, similar a la de SteamOS.

Incluye Decky Loader y CSS Loader para personalizar Steam. Los complementos y los temas se instalan automáticamente, con mis ajustes de apariencia y los iconos de los mandos de la Ally.

Handheld Daemon se encarga de los mandos integrados, los botones especiales y el consumo de energía de la Ally. El botón de encendido suspende el equipo desde cualquiera de los tres escritorios. No utilizo el lector de huellas porque interfería con la suspensión.

También están disponibles Heroic, Lutris y Prism Launcher.

| Inicio | Menú | Ajustes |
| --- | --- | --- |
| ![Inicio del modo consola](capturas/modo-consola/modo-consola-inicio.png) | ![Menú del modo consola](capturas/modo-consola/modo-consola-menu.png) | ![Ajustes del modo consola](capturas/modo-consola/modo-consola-ajustes.png) |

## EmuDeck

Utilizo EmuDeck para instalar y configurar los emuladores. Steam ROM Manager añade los juegos a la biblioteca de Steam y permite abrirlos desde el modo consola.

Korunix incluye las dependencias y el acceso directo de EmuDeck. Los juegos y las partidas se guardan principalmente en una microSD con la etiqueta EmuDeck. Si reinstalo NixOS, puedo conservar la tarjeta y recuperar la instalación desde EmuDeck.

La [guía de EmuDeck](docs/emudeck.md) explica cómo recuperar los emuladores y los juegos en Steam.

## Aplicaciones

Figma, Darktable, GIMP, Inkscape, Blender, Kdenlive, DaVinci Resolve, OBS, BirdFont, FontForge, Obsidian, LibreOffice, OnlyOffice y Visual Studio Code, entre otras.

La mayoría se instala desde Nixpkgs, aunque también se utilizan Flatpak y AppImage.

## Apariencia

Los colores del sistema se generan a partir del fondo de pantalla mediante Noctalia y se comparten entre los escritorios y las aplicaciones compatibles.

Los fondos están divididos en dos colecciones, una para el día y otra para la noche. El modo claro y oscuro cambia automáticamente según la posición del sol.

El tema de iconos es Hatter y el cursor es Bibata.

## Otros detalles

- Reducción de ruido del micrófono.
- Acceso remoto mediante Tailscale y SSH.
- Transmisión de juegos mediante Sunshine.
- Transferencia de archivos mediante LocalSend.
- Duplicación de pantalla mediante UxPlay.
- Compatibilidad con AppImages.
- Limpieza automática de generaciones antiguas de NixOS.

## Compatibilidad

Korunix está configurado para la ASUS ROG Ally original (RC71L) con AMD Ryzen Z1 Extreme.

Para instalarlo en otro equipo es necesario adaptar la configuración del hardware, las particiones y el usuario. El repositorio incluye una plantilla para ello.

No es una distribución independiente, sino una configuración de NixOS.

## Documentación

- [Instalación y mantenimiento](docs/configuracion-y-mantenimiento.md)
- [Estructura y personalización](docs/personalizacion.md)
- [EmuDeck](docs/emudeck.md)

## Créditos

- [NixOS y Nixpkgs](https://github.com/NixOS/nixpkgs) y [NixOS Hardware](https://github.com/NixOS/nixos-hardware).
- [Umbriel](https://github.com/noctalia-dev/umbriel), [Noctalia](https://github.com/noctalia-dev/noctalia), [Luciole](https://github.com/noctalia-dev/luciole) y [GNOME](https://www.gnome.org/).
- [Hatter](https://github.com/Mibea/Hatter), [Bibata](https://github.com/ful1e5/Bibata_Cursor) y [ChromaLeon](https://github.com/Fabito02/ChromaLeon).
- [Handheld Daemon](https://github.com/hhd-dev/hhd), [Gamescope](https://github.com/ValveSoftware/gamescope), [Decky Loader](https://github.com/SteamDeckHomebrew/decky-loader) y [EmuDeck](https://github.com/dragoonDorise/EmuDeck).
- [CSS Loader](https://github.com/DeckThemes/SDH-CssLoader) y los autores de los [temas de DeckThemes](https://deckthemes.com/).

La receta de Decky está adaptada de [Jovian-NixOS](https://github.com/Jovian-Experiments/Jovian-NixOS) y varios temas provienen de las [plantillas de la comunidad de Noctalia](https://github.com/noctalia-dev/community-templates).

## Licencia

Comparto mi configuración y su documentación bajo la [licencia MIT](LICENSE). Los fondos, las capturas y el material de terceros quedan fuera de esta licencia. Las licencias y atribuciones de terceros se conservan en el mismo archivo.

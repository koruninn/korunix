# EmuDeck

Utilizo EmuDeck para instalar y configurar los emuladores de la Ally. Korunix prepara las dependencias de Linux y el acceso a la microSD; EmuDeck se encarga de los emuladores, los mandos y Steam ROM Manager.

## Instalar o recuperar EmuDeck

La tarjeta que utilizo tiene formato ext4 y la etiqueta EmuDeck. Su carpeta es:

```text
/run/media/koru/EmuDeck
```

Si ya tienes juegos o partidas en la tarjeta, consérvala tal cual. No hace falta formatearla ni cambiar su UUID. NixOS puede arrancar sin ella.

Después de instalar Korunix, conecta la tarjeta y abre EmuDeck desde el menú de aplicaciones en GNOME o Umbriel. También puedes abrirlo desde la terminal:

```sh
emudeck
```

Si aún no está descargado, el comando obtiene la versión 2.7.1 de la publicación oficial. Si ya existe en `~/Applications/EmuDeck.AppImage`, utiliza ese archivo. EmuDeck puede actualizarse desde su propia interfaz.

En la primera instalación, selecciona la microSD como destino y los emuladores que utilizas. Si estás recuperando una instalación, selecciona la misma tarjeta y la carpeta Emulation existente. Revisa los avisos antes de confirmar cualquier restablecimiento.

Los emuladores se instalan y actualizan desde EmuDeck. No hace falta instalarlos otra vez desde Nixpkgs ni configurar sus mandos uno por uno.

## Juegos y partidas

EmuDeck crea una carpeta `Emulation` dentro de la tarjeta. Allí se encuentran, entre otras, las carpetas `roms`, `bios`, `saves` y `tools`.

Copia tus juegos a la carpeta de cada consola y los archivos de sistema que requiera el emulador a la ubicación que indica EmuDeck. EmuDeck no incluye juegos, BIOS ni claves.

Algunos emuladores guardan datos fuera de la microSD. Para conservar tu instalación, haz una copia de la tarjeta completa, de tu carpeta personal (incluidos los archivos ocultos) y de los datos de Steam. Conserva los enlaces simbólicos al copiar. No guardes partidas ni datos de cuenta en el repositorio.

## Añadir los juegos a Steam

Abre Steam al menos una vez e inicia sesión. En Ajustes > Almacenamiento, añade la microSD si no aparece. Este paso solucionó el acceso a la tarjeta desde el modo consola en mi Ally. Una instalación nueva de Steam puede necesitarlo otra vez.

Cierra Steam por completo y abre Steam ROM Manager desde EmuDeck. También puedes hacerlo con:

```sh
emudeck srm
```

Selecciona las reglas de los emuladores que utilizas, revisa los juegos encontrados y guarda los accesos directos en Steam. Si incorporas más juegos, vuelve a ejecutar Steam ROM Manager.

Si Steam ROM Manager aparece con la interfaz clásica, sin las reglas o las rutas de EmuDeck, cierra tanto Steam como Steam ROM Manager y ejecuta:

```sh
emudeck reparar-srm
```

El comando usa el restablecimiento oficial de EmuDeck y abre su Steam ROM Manager. Solo úsalo si necesitas recuperar la configuración; reemplaza las reglas personalizadas de Steam ROM Manager. Antes comprueba que EmuDeck esté apuntando a la tarjeta correcta.

## Qué queda en Korunix

- [emulacion.nix](../aplicaciones/emulacion.nix) instala las dependencias y el acceso directo de EmuDeck.
- [emudeck.sh](../aplicaciones/emudeck.sh) abre el instalador oficial o el Steam ROM Manager de la tarjeta.
- [almacenamiento.nix](../servicios/almacenamiento.nix) conserva la ruta de la tarjeta.
- [ejecutables.nix](../modulos/base/ejecutables.nix) permite ejecutar los AppImages.

No hace falta repetir los comandos de compatibilidad de NixOS al reinstalar. La selección inicial de emuladores, la cuenta de Steam y la importación de juegos siguen siendo parte de las aplicaciones, no del repositorio.

Para más detalles, consulta el [manual de EmuDeck](https://manual.emudeck.com/).

# NixDeck

[Volver a Korunix](../README.md)

NixDeck es la parte de Korunix que se encarga de la emulación. Guarda los juegos, las partidas y las configuraciones de los emuladores en una microSD. Utiliza Steam ROM Manager para añadir los juegos a la biblioteca de Steam.

No es un programa independiente. Los emuladores y el comando `nixdeck` se instalan al aplicar Korunix. Escribe `nixdeck` sin ninguna acción para ver la ayuda.

## 1. Preparar la microSD

La tarjeta debe tener una partición en formato ext4, un sistema de archivos de Linux. Si ya está en ext4, no necesitas formatearla.

Si necesitas cambiar el formato, utiliza una herramienta de discos. Selecciona la microSD, no el disco interno de la Ally. **Formatear borra los archivos de la partición.** Haz una copia antes.

NixDeck reconoce la tarjeta por su etiqueta (el nombre del sistema de archivos), no por su UUID. Puedes utilizar otra tarjeta o volver a formatearla sin editar archivos Nix ni reconstruir el sistema.

### Configurar la tarjeta

Conecta la microSD, cierra Steam ROM Manager y los emuladores y abre una terminal:

```sh
nixdeck configurar
```

El comando muestra las particiones ext4 de dispositivos extraíbles. Selecciona la de tu tarjeta y escribe `NixDeck` para confirmar.

Si la tarjeta aún no tiene esa etiqueta, debe estar desmontada. Puedes desmontarla desde el explorador de archivos o una herramienta de discos y volver a ejecutar el comando, sin retirarla. Es posible que se solicite tu contraseña para cambiar la etiqueta o los permisos.

El comando no formatea la tarjeta: cambia su etiqueta si hace falta, comprueba el montaje, ajusta los permisos de la carpeta raíz si es necesario, prepara las carpetas y descarga DuckStation y Xenia.

Solo conecta una tarjeta con la etiqueta NixDeck a la vez. Si tu lector no se identifica como extraíble, puedes asignarle la etiqueta NixDeck manualmente desde una herramienta de discos. Comprueba que tengas permisos de escritura y ejecuta `nixdeck sincronizar` con Steam y los emuladores cerrados.

### Comprobar el montaje

Montar una tarjeta significa que sus archivos quedan disponibles en una carpeta del sistema. NixDeck monta la tarjeta automáticamente al acceder a su carpeta:

```text
/run/media/koru/NixDeck
```

Si usas otro nombre de usuario, sustituye `koru` en las rutas de esta guía.

Para comprobar la etiqueta, el formato y la ubicación de la tarjeta:

```sh
ls /run/media/koru/NixDeck
lsblk -f
```

Busca una partición ext4 con la etiqueta NixDeck. NixOS puede arrancar sin la tarjeta y los lanzadores comprueban que esté montada antes de abrir los emuladores.

## 2. Carpetas de la tarjeta

`nixdeck configurar` crea las carpetas y prepara las rutas de los emuladores. Si ya tienes partidas, haz una copia antes. NixDeck conserva las carpetas que contienen datos distintos y avisa si no puede unirlas sin sobrescribir archivos.

La tarjeta queda organizada de esta forma:

```text
NixDeck/
├── roms/           Juegos
├── bios/           BIOS, firmware y claves
├── saves/          Partidas y tarjetas de memoria
├── states/         Estados de guardado
├── storage/        Almacenamiento interno de las consolas y datos de Steam ROM Manager
├── emulators/      Configuraciones y datos de los emuladores
├── mods/           Mods y trucos
├── shaders/        Sombreadores
├── screenshots/    Capturas de pantalla
├── texture_packs/  Paquetes de texturas
└── appimages/      Emuladores descargados por separado
```

Las carpetas de cada emulador se crean cuando hacen falta.

## 3. Copiar los juegos

Una ROM o imagen de disco es una copia digital de un juego de consola. NixDeck no incluye juegos.

Abre la tarjeta desde el explorador de archivos y entra en la carpeta `roms`. Guarda tus copias de juegos en la carpeta correspondiente:

| Consola | Emulador | Carpeta dentro de roms |
| --- | --- | --- |
| PlayStation | DuckStation | psx |
| PlayStation 2 | PCSX2 | ps2 |
| PlayStation 3 | RPCS3 | ps3 |
| PSP | PPSSPP | psp |
| GameCube | Dolphin | gc |
| Wii | Dolphin | wii |
| Wii U | Cemu | wiiu |
| Nintendo Switch | Eden | switch |
| Nintendo 3DS | Azahar | n3ds |
| Nintendo DS | melonDS | nds |
| Game Boy Advance | mGBA | gba |
| Xbox 360 | Xenia | xbox360 |

Por ejemplo, un juego de Wii en formato RVZ va en `roms/wii`. Un juego de PS2 en formato ISO va en `roms/ps2`.

Algunos juegos, especialmente los de PS3 y Wii U, vienen en carpetas con varios archivos. Conserva su estructura original. No todos los emuladores aceptan los mismos formatos.

Las reglas de Steam ROM Manager reconocen, entre otros, archivos CHD y CUE para PS1, ISO y CHD para PS2, RVZ para GameCube y Wii, y WUA para Wii U. Para PS3 también buscan juegos con la estructura `PS3_GAME/USRDIR/EBOOT.BIN` y juegos instalados en el almacenamiento de RPCS3. Para Wii U reconocen archivos RPX de juegos extraídos.

Que Steam encuentre un archivo no garantiza que el emulador pueda ejecutarlo.

## 4. DuckStation y Xenia

Los demás emuladores se instalan desde Nixpkgs. DuckStation y Xenia se descargan automáticamente en formato AppImage cuando ejecutas `nixdeck configurar` o `nixdeck sincronizar`.

NixDeck utiliza las publicaciones de [DuckStation](https://github.com/stenzek/duckstation/releases) (versión estable para Linux x64) y [Xenia Canary](https://github.com/xenia-canary/xenia-canary/releases) (versión para Linux). Los archivos se guardan en `appimages` y reciben permiso de ejecución.

Si ya tienes una versión instalada, NixDeck comprueba si hay otra disponible. Solo reemplaza el archivo después de comprobar que la descarga coincide con el tamaño y el SHA-256 de la publicación. Conserva una copia de la versión anterior en `appimages/.anteriores`. Si falla la descarga, conserva la versión que ya funciona.

No hace falta descargarlos manualmente. Necesitas internet la primera vez; si ya están instalados, puedes jugar sin conexión. Cierra los emuladores antes de configurar o sincronizar la tarjeta.

Puedes abrir ambos desde el menú de aplicaciones. Los lanzadores utilizan las rutas de NixDeck, por lo que no debes abrir directamente los AppImages si quieres mantener los datos en la tarjeta.

## 5. Configurar las BIOS y el firmware

Algunos emuladores necesitan archivos del sistema de la consola original para funcionar. NixDeck prepara sus carpetas, pero no incluye las BIOS, el firmware ni las claves.

| Consola | Carpeta dentro de bios |
| --- | --- |
| PlayStation | psx |
| PlayStation 2 | ps2 |
| PlayStation 3 | ps3 |
| Nintendo Switch | switch/keys |
| Nintendo 3DS | n3ds/sysdata |
| Nintendo DS | nds |
| Nintendo DSi | dsi |
| Game Boy Advance | gba |
| Wii U | wiiu |

Coloca los archivos obtenidos legítimamente en las carpetas correspondientes. Cada emulador tiene sus propios requisitos; algunos necesitan que importes o selecciones los archivos desde sus ajustes.

### PlayStation 3

Descarga el firmware oficial de PS3 desde PlayStation y guarda el archivo como:

```text
bios/ps3/PS3UPDAT.PUP
```

Cierra RPCS3 y ejecuta:

```sh
nixdeck firmware
```

El comando abre RPCS3 para instalar el firmware. También puedes instalarlo desde la interfaz de RPCS3 si el comando falla.

### Nintendo Switch

Guarda las claves en `bios/switch/keys` e instala el firmware desde la interfaz de Eden.

### Nintendo DS y Game Boy Advance

NixDeck reconoce determinados nombres de archivos al preparar melonDS y mGBA. Para DS utiliza `bios7.bin`, `bios9.bin` y `firmware.bin` dentro de `bios/nds`. Para GBA reconoce `gba_bios.bin`, `gba.bin` o `bios.bin` dentro de `bios/gba`.

Después de añadir los archivos con los emuladores cerrados, vuelve a abrir el emulador correspondiente. NixDeck actualiza sus rutas al abrirlo. Todavía no comprueba que todas las BIOS sean válidas.

## 6. Añadir los juegos a Steam

Steam ROM Manager busca los juegos en la microSD y crea accesos directos en Steam. No copia los juegos ni los convierte en juegos comprados en Steam.

Antes de utilizarlo por primera vez, abre Steam e inicia sesión con tu cuenta. Después ciérralo por completo, no solo su ventana. Puedes hacerlo desde el menú Steam, seleccionando Salir.

Cierra también Steam ROM Manager si está abierto. Desde una sesión de escritorio, ejecuta:

```sh
nixdeck sincronizar
```

El comando prepara las reglas de búsqueda, comprueba las actualizaciones de DuckStation y Xenia y añade los accesos directos a Steam. Cada consola tiene una regla que indica dónde buscar los juegos y con qué emulador abrirlos. Steam ROM Manager llama parsers a estas reglas.

Cuando termine, vuelve a abrir Steam y busca los juegos en tu biblioteca. También deberían aparecer en el modo consola.

Si quieres revisar las reglas, las portadas o los accesos antes de guardarlos, abre la interfaz de Steam ROM Manager:

```sh
nixdeck steam
```

Puedes usar esa interfaz para revisar y guardar la importación con Steam cerrado.

Cuando añadas más juegos, vuelve a ejecutar la sincronización. Para comprobar un juego que no abre, prueba primero el emulador desde el escritorio y revisa si necesita archivos de sistema o ajustes adicionales.

La importación y ejecución de juegos de Wii se ha probado en Veskalia. Todavía falta probar el proceso completo con todas las consolas.

## 7. Usar el celular como mando

[PepoMote](https://github.com/PepoTechYT/PepoMote) permite utilizar un celular como mando para Wii, Wii U y Switch.

Descarga el receptor de Linux y la aplicación para tu celular desde el proyecto. Korunix configura las rutas de Dolphin, Cemu y Eden para que el receptor utilice los ajustes de NixDeck.

1. Si acabas de aplicar Korunix, cierra sesión y vuelve a entrar.
2. Conecta el celular y la Ally a la misma red.
3. Cierra los emuladores y abre el receptor.
4. Conecta el celular desde PepoMote y selecciona la consola.
5. Abre el emulador correspondiente.

## 8. Guardar una copia de tus partidas

Una partida normal se guarda dentro del juego. Un estado de guardado permite guardar el punto exacto en el que estás mediante el emulador.

No todas las partidas se encuentran en `saves`. Algunas consolas utilizan su propio almacenamiento virtual dentro de `storage`, y ciertos emuladores conservan datos en `emulators`.

Para respaldar NixDeck, cierra los emuladores y copia la tarjeta completa a otro disco, incluyendo los archivos ocultos y conservando los enlaces simbólicos (accesos a archivos o carpetas). Si solo copias `saves`, puedes dejar partidas fuera de la copia.

No retires la microSD mientras un emulador esté abierto o se estén escribiendo archivos. Expúlsala desde el explorador antes de retirarla.

## Si algo no funciona

| Problema | Qué revisar |
| --- | --- |
| La tarjeta no está montada | Comprueba que sea ext4, tenga la etiqueta NixDeck y que no haya otra tarjeta con la misma etiqueta. |
| No hay permiso de escritura | Ejecuta `nixdeck configurar` con la tarjeta conectada. |
| DuckStation no abre | Ejecuta `nixdeck sincronizar` con conexión a internet y revisa los avisos. |
| Xenia no abre | Comprueba que haya un solo AppImage de Xenia y ejecuta `nixdeck sincronizar`. |
| Un juego no aparece en Steam | Revisa su carpeta y formato, cierra Steam y vuelve a sincronizar. |
| Un juego aparece pero no abre | Prueba el emulador desde el escritorio y revisa su configuración, BIOS o firmware. |
| El organizador avisa de un conflicto | Conserva las dos ubicaciones y revisa su contenido antes de mover o borrar nada. |

NixDeck sigue en desarrollo. Faltan la comprobación de BIOS, las copias de seguridad automáticas y las pruebas de los perfiles de mandos, atajos y gráficos de todos los emuladores.

## Archivos de NixDeck

- [emulacion.nix](../aplicaciones/emulacion.nix): define los emuladores, el montaje de la microSD y los comandos.
- [nixdeck-tarjeta.py](../aplicaciones/nixdeck-tarjeta.py): identifica, etiqueta y comprueba la tarjeta.
- [appimages.nix](../aplicaciones/appimages.nix): fuentes de descarga de DuckStation y Xenia.
- [nixdeck-appimages.py](../aplicaciones/nixdeck-appimages.py): descarga y actualiza DuckStation y Xenia.
- [nixdeck-datos.sh](../aplicaciones/nixdeck-datos.sh): organiza las carpetas y vincula los datos de los emuladores.
- [nixdeck-config.py](../aplicaciones/nixdeck-config.py): ajusta las rutas de los emuladores y genera las reglas de Steam ROM Manager.

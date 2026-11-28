# Estructura y personalización

[Volver a Korunix](../README.md)

## Dónde cambiar cada cosa

Los archivos `.nix` son archivos de texto. Puedes abrirlos con cualquier editor, como GNOME Text Editor o Visual Studio Code. Los cambios se aplican al reconstruir el sistema.

| Archivo o carpeta | Qué puedes cambiar |
| --- | --- |
| [equipo.nix](../equipo.nix) | Sesión predeterminada, pantalla y rama de NixOS. |
| [configuracion.nix](../configuracion.nix) | Idioma, zona horaria y ajustes generales. |
| [usuario.nix](../usuario.nix) | Nombre y permisos del usuario. |
| [hardware.nix](../hardware.nix) | Particiones del disco y soporte de hardware. |
| [aplicaciones](../aplicaciones) | Programas instalados y su configuración. |
| [apariencia](../apariencia) | Escritorios, fondos, colores, iconos y cursor. |
| [servicios](../servicios) | Mandos, redes, arranque y otras funciones del sistema. |
| [modulos/base](../modulos/base) | Audio, energía y herramientas básicas. |
| [flake.nix](../flake.nix) | Dependencias y archivos que forman el sistema. |
| [flake.lock](../flake.lock) | Versiones exactas de las dependencias. Nix lo genera y actualiza. |
| [plantilla-equipo](../plantilla-equipo) | Archivos de ejemplo para configurar otro equipo. |

Si quieres adaptar Korunix a una instalación nueva, empieza por la [guía de instalación](configuracion-y-mantenimiento.md).

## Añadir o quitar programas

### Nixpkgs

Nixpkgs es el repositorio de paquetes que utiliza NixOS. Un paquete contiene lo necesario para instalar un programa.

Por ejemplo, para instalar Krita:

1. Busca `krita` en [NixOS Search](https://search.nixos.org/packages). Selecciona la rama que usas en `equipo.nix` (stable o unstable).
2. Abre [aplicaciones/paquetes.nix](../aplicaciones/paquetes.nix).
3. Busca la lista que empieza con `environment.systemPackages = with pkgs; [`.
4. Añade `krita` en una línea dentro de esa lista.

Esta es una parte de la lista, con Krita añadido:

```nix
    gimp
    heroic
    inkscape
    krita
```

No reemplaces la lista completa por este ejemplo. Guarda el archivo y aplica los cambios desde la carpeta de Korunix:

```sh
just switch
```

Para quitar un programa que se instala desde esa lista, elimina su línea y vuelve a aplicar la configuración. Algunos programas, como Firefox y Steam, tienen sus propios archivos. Revisa esos módulos si el programa no aparece en la lista.

### Flakes de otros proyectos

No todos los programas de Korunix vienen de Nixpkgs. Algunos proyectos tienen su propio `flake.nix` y ofrecen paquetes o módulos que puedes utilizar directamente desde su repositorio.

Un flake no es necesariamente un programa: también puede contener una configuración completa, módulos, herramientas o varios paquetes. Korunix, por ejemplo, es un flake que contiene una configuración de NixOS. Figma Linux Next y Luciole son proyectos que ofrecen sus aplicaciones mediante flakes.

Para saber qué ofrece un proyecto, revisa su README y su archivo `flake.nix`. También puedes consultar sus resultados con `nix flake show`, seguido de la dirección del flake. Por ejemplo:

```sh
nix flake show github:arximus88/figma-linux-next
```

Hay dos formas habituales de incorporar estos programas.

#### Cuando el flake ofrece un módulo de NixOS

Nix Flatpak es un ejemplo que ya está en Korunix.

1. Declara el proyecto en `inputs` de [flake.nix](../flake.nix):

   ```nix
   nix-flatpak.url = "github:gmodena/nix-flatpak?ref=latest";
   ```

2. Importa su módulo y activa Flatpak. Puedes ver cómo se hace en [aplicaciones/flatpak.nix](../aplicaciones/flatpak.nix):

   ```nix
   {inputs, ...}: {
     imports = [inputs.nix-flatpak.nixosModules.nix-flatpak];
     services.flatpak.enable = true;
   }
   ```

3. Si creas un módulo nuevo, añádelo a los imports de [aplicaciones/default.nix](../aplicaciones/default.nix) y a Git antes de reconstruir.

#### Cuando el flake ofrece un paquete

Si el proyecto no proporciona un módulo de NixOS, pero sí un paquete, puedes instalarlo mediante `environment.systemPackages`.

Luciole y Figma Linux Next funcionan de esta manera. Su dependencia está declarada en `flake.nix` y se utiliza desde [luciole.nix](../apariencia/escritorios/noctalia/luciole.nix). Esta es una versión reducida de ese método:

```nix
{inputs, pkgs, ...}: {
  environment.systemPackages = [
    inputs.luciole.packages.${pkgs.stdenv.hostPlatform.system}.default
  ];
}
```

- `inputs.luciole` accede al proyecto declarado en `flake.nix`.
- `packages` contiene los paquetes que ofrece ese proyecto.
- `${pkgs.stdenv.hostPlatform.system}` selecciona la arquitectura del equipo. En la Ally es `x86_64-linux`.
- `default` selecciona el paquete predeterminado. Algunos proyectos utilizan otros nombres.

No necesitas añadir este ejemplo: Luciole ya está configurado en Korunix. Para instalar otro programa de esta forma, declara su input, crea e importa su módulo y utiliza el nombre del paquete que realmente ofrece el proyecto.

#### Probar, actualizar y quitar un flake

Después de añadir un input y los archivos necesarios, ejecuta estos comandos desde la carpeta de Korunix, uno por uno:

```sh
nix flake lock
just revisar
just probar
just switch
```

El primero registra la dependencia en `flake.lock`. Si algún comando falla, no continúes hasta corregir el error.

Para actualizar únicamente un proyecto, utiliza su nombre en `inputs`. Por ejemplo:

```sh
nix flake update figma-linux-next
```

Después comprueba y aplica la configuración. Si la dirección del input incluye un commit concreto, como ocurre con Luciole, actualizar el lock no cambia ese commit. Tendrás que modificar la dirección si quieres pasar a otra revisión.

Para quitar un programa instalado mediante un flake, elimina su paquete o módulo de la configuración. Si ya no se utiliza ese input en ningún otro archivo, elimínalo también de `flake.nix` y ejecuta `nix flake lock` antes de reconstruir.

### Flatpak

Flatpak es otra forma de instalar aplicaciones. Las de Korunix se descargan de Flathub.

Busca el programa en [Flathub](https://flathub.org/) y copia su identificador, que suele tener un formato parecido a `org.nombre.Aplicacion`.

Abre [aplicaciones/flatpak.nix](../aplicaciones/flatpak.nix) y añade el identificador entre comillas dentro de `services.flatpak.packages`. Por ejemplo, para añadir Bottles:

```nix
    "com.usebottles.bottles"
```

Guarda el archivo y ejecuta `just switch`. Para dejar de gestionar una aplicación desde Korunix, quítala de la lista. Si sigue instalada, puedes desinstalarla desde la tienda de aplicaciones o con `flatpak uninstall` seguido de su identificador.

### AppImage

Un AppImage es un programa que viene en un solo archivo ejecutable. No hace falta añadirlo a la lista de paquetes.

Korunix incluye Gear Lever para gestionar estos archivos y soporte adicional en [modulos/base/ejecutables.nix](../modulos/base/ejecutables.nix). Si un AppImage no abre, comprueba que tenga permiso de ejecución. No todos los programas hechos para otras distribuciones son compatibles con NixOS.

#### Actualizaciones de AppImages

Gear Lever necesita saber de dónde descargar una versión nueva. Algunos AppImages ya incluyen esa información; otros necesitan una fuente de actualización.

Las fuentes de actualización de Gear Lever están en [aplicaciones/appimages.nix](../aplicaciones/appimages.nix). Los emuladores y sus AppImages se gestionan desde EmuDeck. No los integres de nuevo en Gear Lever.

Para añadir otro AppImage, primero integra el archivo desde Gear Lever. Puedes consultar su ubicación con:

```sh
gearlever --list-installed
```

Después añade una declaración dentro de la lista `gearlever` en `appimages.nix`. Este ejemplo muestra el formato; sustituye el nombre del archivo y la dirección por los de tu aplicación:

```nix
{
  archivo = "AppImages/Nombre-*.AppImage";
  url = "https://github.com/autor/proyecto/releases/download/*/Nombre-*.AppImage";
  gestor = "GithubUpdater";
}
```

La ruta `archivo` empieza dentro de tu carpeta personal. El asterisco permite encontrar el archivo aunque cambie el número de versión. Debe coincidir con un solo AppImage integrado.

La dirección `url` debe ser la fuente de actualización que admite Gear Lever. Para GitHub indica el repositorio y el patrón del archivo publicado. También puedes usar una URL HTTPS de descarga directa y omitir `gestor` para que Gear Lever detecte el tipo de fuente.

Aplica la configuración con `just switch` y vuelve a iniciar sesión. Korunix registra las fuentes declaradas en Gear Lever al iniciar el escritorio. Después puedes buscar e instalar actualizaciones desde Gear Lever.

Si ya tienes la sesión abierta y quieres registrar las fuentes sin cerrarla, ejecuta:

```sh
systemctl --user start korunix-gearlever-fuentes.service
```

Si un AppImage no tiene una fuente de actualización conocida, consulta la documentación de su proyecto. No todos admiten actualizaciones desde Gear Lever. La lista no instala ni elimina los archivos que integras manualmente.

## Cambiar los escritorios

Los ajustes generales están en [equipo.nix](../equipo.nix). Los valores `true` y `false` significan activado y desactivado.

Por ejemplo, para desactivar Umbriel:

```nix
habilitarUmbriel = false;
```

La sesión seleccionada al iniciar y el inicio automático de Steam también se cambian desde ese archivo. La configuración de Steam a pantalla completa está en [servicios/sesiones.nix](../servicios/sesiones.nix).

### Modo consola

Los complementos de Decky y los temas de CSS Loader están en [servicios/decky-complementos.nix](../servicios/decky-complementos.nix).

La lista `plugins` indica qué complementos instalar y la lista `temas` contiene los temas y sus ajustes iniciales. Para añadir otro tema, necesitas su identificador en [DeckThemes](https://deckthemes.com/). En sus ajustes, `active = true;` hace que esté activo desde la primera instalación.

Korunix descarga los complementos y temas que faltan al iniciar Decky. Necesitas internet para descargarlos, pero no para seguir utilizando los que ya tienes. Puedes seguir cambiando sus preferencias y actualizándolos desde Steam.

Si quieres dejar de recuperar un complemento o tema, elimina su entrada de la lista. Esto no lo desinstala: puedes quitarlo desde Decky o CSS Loader. Los datos y las cuentas personales de los complementos no se guardan en el repositorio.

### Aplicaciones fijadas

El dock es la barra donde aparecen las aplicaciones favoritas. Puedes cambiar su contenido en [apariencia/escritorios/dock.nix](../apariencia/escritorios/dock.nix).

La lista `comunes` contiene los identificadores de las aplicaciones, no necesariamente los nombres que aparecen en el menú. Por ejemplo, Firefox es `"firefox"` y Obsidian puede tener otro identificador.

Para ver los archivos de los lanzadores del sistema, ejecuta:

```sh
ls /run/current-system/sw/share/applications
```

Para añadir uno al dock, copia el nombre de su archivo sin la extensión `.desktop`. Añádelo entre comillas a `comunes`. Puedes cambiar el orden moviendo las líneas o quitar un favorito eliminando su línea.

GNOME y Noctalia utilizan esa misma lista, con algunas diferencias. Nautilus se mantiene en GNOME; Umbriel utiliza Luciole.

### Umbriel y Noctalia

Los atajos de teclado de Umbriel están en [umbriel.nix](../apariencia/escritorios/noctalia/umbriel.nix). Por ejemplo:

```nix
"Mod+E" = "spawn:luciole";
```

`Mod` corresponde a la tecla Super (normalmente la tecla con el logo de Windows). `spawn` abre un programa. En este caso, Super + E abre Luciole.

Los paneles, el dock y los widgets de Noctalia están en [ajustes.nix](../apariencia/escritorios/noctalia/ajustes.nix). Sus complementos están en [plugins.nix](../apariencia/escritorios/noctalia/plugins.nix).

### GNOME

Los ajustes de GNOME y sus extensiones están principalmente en [apariencia/default.nix](../apariencia/default.nix).

GSConnect se configura en ese mismo archivo mediante la opción `programs.kdeconnect`, con el paquete `pkgs.gnomeExtensions.gsconnect`. Para compartir archivos con el iPhone utilizo LocalSend.

Si cambias algo desde los ajustes del escritorio y vuelve a su valor anterior al reconstruir, busca esa preferencia en los archivos de Korunix. Algunos ajustes se vuelven a aplicar automáticamente.

### Fondos, colores e iconos

Los fondos del modo claro están en [fondos/claro](../apariencia/fondos/claro) y los del modo oscuro en [fondos/oscuro](../apariencia/fondos/oscuro).

Para añadir una imagen, cópiala a la carpeta correspondiente y añade ese archivo a Git antes de reconstruir. Por ejemplo, si la imagen se llama `paisaje.jpg`:

```sh
git add apariencia/fondos/claro/paisaje.jpg
just switch
```

Noctalia genera los colores del escritorio a partir del fondo. [apariencia/fondos.nix](../apariencia/fondos.nix) sincroniza los fondos y colores entre los escritorios, y [apariencia/modo-solar.py](../apariencia/modo-solar.py) calcula el amanecer y el anochecer para GNOME.

El tema de iconos se configura en [hatter.nix](../apariencia/hatter.nix) y el cursor en [cursor.nix](../apariencia/cursor.nix).

## Conectar el iPhone

Para compartir archivos utilizo LocalSend. Instálalo también en el iPhone, conecta ambos equipos a la misma red y abre la aplicación en los dos.

### Duplicar la pantalla del iPhone

[UxPlay](https://github.com/FDH2/UxPlay) permite recibir la pantalla y el sonido del iPhone mediante AirPlay.

1. Conecta ambos equipos a la misma red.
2. Abre UxPlay desde el menú de aplicaciones.
3. Abre el Centro de control del iPhone, pulsa Duplicar pantalla y elige Veskalia.

Para desconectarlo, cierra UxPlay con `Ctrl+C` desde su terminal.

No se inicia automáticamente y no pide contraseña. Úsalo en redes de confianza.

Su configuración está en [aplicaciones/uxplay.nix](../aplicaciones/uxplay.nix). La opción `-n Veskalia` cambia el nombre que aparece en el iPhone. `-avdec` utiliza el procesador para decodificar el vídeo y `-vsync no` desactiva la sincronización con las marcas de tiempo para reducir el retraso, aunque puede afectar al sonido.

## Cómo leer los archivos Nix

Un módulo es un archivo que define una parte de la configuración de NixOS. Por ejemplo, este instala Git y curl:

```nix
{pkgs, ...}: {
  environment.systemPackages = with pkgs; [
    git
    curl
  ];
}
```

Cada parte tiene una función:

- `{pkgs, ...}:` indica qué información recibe el módulo. `pkgs` contiene los paquetes y `...` permite recibir otros argumentos sin nombrarlos.
- `environment.systemPackages` es la opción de NixOS que define los paquetes instalados en todo el sistema.
- `with pkgs;` permite escribir `git` en vez de `pkgs.git`.
- Los corchetes `[ ]` contienen una lista. Sus elementos se separan con espacios o saltos de línea, no con comas.
- Las llaves `{ }` agrupan atributos. Las asignaciones terminan en punto y coma.

Puedes buscar las opciones de configuración de NixOS en [NixOS Search — Options](https://search.nixos.org/options).

### Qué recibe un módulo

| Argumento | Qué contiene |
| --- | --- |
| `pkgs` | Los paquetes de Nixpkgs. |
| `config` | Los valores de la configuración que NixOS reúne a partir de los módulos. |
| `lib` | Funciones que ayudan a trabajar con Nix y sus opciones. |
| `modulesPath` | La ubicación de los módulos que trae NixOS. |
| `inputs` | Las dependencias del flake de Korunix. |
| `equipo` | Los ajustes escritos en equipo.nix. |
| `canal` | La rama estable o unstable elegida en equipo.nix. |

Los primeros cuatro los proporciona NixOS. Los otros tres se pasan desde `flake.nix` mediante `specialArgs`.

## Cómo se conectan los archivos

### flake.nix

[flake.nix](../flake.nix) es el punto de partida de Korunix.

- `inputs` indica de dónde salen las dependencias, como Nixpkgs, Umbriel y Noctalia.
- `outputs` define lo que produce el flake: la configuración de Veskalia y el formateador de código.
- `lib.nixosSystem` reúne los módulos para crear una configuración de NixOS.
- `modules` contiene los archivos y carpetas que forman parte del sistema.

El archivo `flake.lock` guarda las versiones exactas de los inputs. No hace falta editarlo a mano.

### imports y default.nix

Dentro de las carpetas hay archivos llamados `default.nix` que reúnen otros módulos. Cuando Nix recibe la ruta de una carpeta como módulo, busca su `default.nix`.

Por ejemplo, [modulos/base/default.nix](../modulos/base/default.nix) contiene:

```nix
{
  imports = [
    ./audio.nix
    ./energia.nix
    ./herramientas.nix
    ./ejecutables.nix
  ];
}
```

`imports` indica qué módulos debe cargar NixOS. El prefijo `./` significa que el archivo está en la misma carpeta.

Si quieres separar la configuración de Krita en un archivo propio, en vez de añadirlo a `paquetes.nix`, crea `aplicaciones/krita.nix` con este contenido:

```nix
{pkgs, ...}: {
  environment.systemPackages = [pkgs.krita];
}
```

Después abre [aplicaciones/default.nix](../aplicaciones/default.nix) y añade `./krita.nix` dentro de `imports`.

Desde la carpeta del repositorio, ejecuta:

```sh
git add aplicaciones/krita.nix
```

Si no importas el módulo, NixOS no aplicará sus ajustes. Si no añades el archivo nuevo a Git, el flake no podrá encontrarlo.

### specialArgs

Los módulos necesitan a veces información que no viene incluida en NixOS, como las preferencias de `equipo.nix`.

En `flake.nix`, Korunix lee ese archivo:

```nix
equipo = import ./equipo.nix;
```

Después comparte su contenido con los módulos:

```nix
specialArgs = {
  inherit inputs equipo;
  canal = equipo.canal;
};
```

`specialArgs` permite pasar argumentos adicionales a los módulos. `inherit inputs equipo;` es una forma corta de escribir `inputs = inputs;` y `equipo = equipo;`.

Así, un módulo que empieza con `{equipo, ...}:` puede leer `equipo.habilitarUmbriel` sin tener que abrir de nuevo `equipo.nix`. Otro que recibe `inputs` puede acceder a las dependencias del flake.

### Variables y condiciones

`let ... in` permite guardar un valor con un nombre para utilizarlo más adelante:

```nix
{pkgs, ...}: let
  terminal = pkgs.ptyxis;
in {
  environment.systemPackages = [terminal];
}
```

Aquí `terminal` representa el paquete Ptyxis.

`lib.mkIf` permite aplicar ajustes solo cuando se cumple una condición:

```nix
{equipo, lib, pkgs, ...}: lib.mkIf (equipo.habilitarUmbriel or false) {
  environment.systemPackages = [pkgs.ptyxis];
}
```

Este ejemplo añade Ptyxis si Umbriel está habilitado. `or false` utiliza false si el atributo no existe. Es un ejemplo de sintaxis, no un módulo que tengas que añadir: Korunix ya incluye Ptyxis.

### Cuando varios módulos cambian lo mismo

NixOS combina los ajustes de todos los módulos. Por ejemplo, si dos archivos añaden programas a `environment.systemPackages`, Nix reúne ambas listas.

Si dos módulos asignan valores incompatibles a una opción, puede aparecer un error. Para estos casos existen:

- `lib.mkDefault`: define un valor que una asignación normal puede reemplazar.
- `lib.mkForce`: da prioridad a un valor frente a una asignación normal.
- `lib.mkBefore` y `lib.mkAfter`: cambian el orden al combinar listas.

En Korunix, [modulos/base/energia.nix](../modulos/base/energia.nix) activa Power Profiles Daemon, un servicio que gestiona los perfiles de energía. [servicios/mandos.nix](../servicios/mandos.nix) lo desactiva con `mkForce false` para evitar que interfiera con Handheld Daemon.

El botón de encendido lo gestiona el sistema. Antes de iniciar HHD, [hhd-encendido.py](../servicios/hhd-encendido.py) desactiva su gestor del botón sin cambiar los demás ajustes. [energia.nix](../modulos/base/energia.nix) y [apariencia/default.nix](../apariencia/default.nix) establecen la suspensión. El lector de huellas permanece desactivado porque interfería con esa función.

No uses `mkForce` para tapar cualquier error de configuración. Primero revisa qué módulos están definiendo la misma opción.

## Aplicar los cambios

Desde la carpeta del repositorio, ejecuta estos comandos uno por uno:

```sh
cd ~/.korunix
just revisar
just probar
just switch
```

El primero te lleva al repositorio. `just revisar` comprueba la configuración, `just probar` construye el sistema sin activarlo y `just switch` lo aplica. Si uno falla, no continúes hasta corregir el error.

Los archivos nuevos deben añadirse a Git antes de hacer estas comprobaciones. No es necesario hacer un commit para probar los cambios.

Korunix no guarda tus archivos personales, contraseñas ni sesiones iniciadas. Algunos ajustes de las aplicaciones sí se administran desde los módulos, pero una reconstrucción no equivale a una copia de seguridad de tu carpeta personal.

Para recuperar una versión anterior o actualizar las dependencias, consulta la [guía de instalación y mantenimiento](configuracion-y-mantenimiento.md).

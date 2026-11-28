# Instalación y mantenimiento

[Volver a Korunix](../README.md)

## 1. Instalar NixOS

Korunix no se instala como una distribución de Linux. Primero necesitas tener NixOS instalado y funcionando. Puedes descargarlo desde la [página de NixOS](https://nixos.org/download/).

Antes de cambiar la configuración, guarda tus archivos personales y una copia de `/etc/nixos`, donde normalmente está la configuración que se creó durante la instalación.

## 2. Descargar Korunix

Los flakes permiten que Nix lea la configuración de este repositorio. Si todavía no están habilitados, abre `/etc/nixos/configuration.nix` (o el archivo principal de tu configuración actual) y añade esta línea dentro del bloque de configuración:

```nix
nix.settings.experimental-features = ["nix-command" "flakes"];
```

Guarda el archivo y aplica el cambio. En una instalación normal de NixOS sin flakes se hace así:

```sh
sudo nixos-rebuild switch
```

### Git

Git es la herramienta que permite descargar y actualizar el repositorio. Como no viene instalado por defecto en NixOS, abre una terminal y ejecuta:

```sh
nix-shell -p git
```

Esto abre una terminal con Git disponible sin tener que instalarlo permanentemente. Desde esa misma terminal, descarga Korunix:

```sh
git clone https://github.com/koruninn/korunix.git ~/.korunix
cd ~/.korunix
```

El primer comando descarga el repositorio y el segundo te lleva a su carpeta. El símbolo `~` representa tu carpeta personal.

Mantén esta terminal abierta mientras sigues la guía. Korunix instala Git automáticamente al aplicar su configuración.

## 3. Adaptar la configuración

Los archivos de Korunix corresponden a una ASUS ROG Ally RC71L con AMD Ryzen Z1 Extreme. Aunque tengas el mismo modelo, no debes aplicar la configuración tal como viene: las particiones de tu disco y tu usuario pueden ser distintos.

### hardware.nix

Aquí están las particiones del disco, los controladores necesarios para arrancar y el perfil de hardware de la Ally.

NixOS puede detectar tu hardware y mostrarte su configuración sin modificar el sistema:

```sh
sudo nixos-generate-config --show-hardware-config > hardware-local.nix
```

Abre `hardware-local.nix` y compara su contenido con [hardware.nix](../hardware.nix). Lleva los ajustes de tu equipo a `hardware.nix`, incluyendo las particiones, sus sistemas de archivos y las opciones de montaje.

Cada partición tiene un identificador llamado UUID. Puedes ver los de tu equipo con:

```sh
lsblk -f
```

En `hardware.nix`, `fileSystems."/"` corresponde a la partición donde está instalado NixOS, `fileSystems."/boot"` a la de arranque y `swapDevices` a la memoria de intercambio, si existe. Usa las particiones que creó tu instalación, no las de Veskalia.

Si tienes la misma Ally, conserva el perfil `asus-ally-rc71l` de NixOS Hardware. Si tienes otro equipo, revisa también ese perfil y los servicios específicos de la Ally.

El archivo `hardware-local.nix` solo sirve como referencia. No lo añadas al repositorio.

### usuario.nix

En [usuario.nix](../usuario.nix) se define la cuenta de usuario y sus permisos. El usuario de Veskalia se llama `koru`.

Si tu cuenta tiene otro nombre, cambia también las referencias a `koru` en los módulos y scripts. Este comando te muestra dónde aparece:

```sh
git grep -n -w koru -- '*.nix' '*.py' '*.sh' justfile
```

No basta con cambiar el nombre en un solo archivo. Hay servicios y rutas que utilizan ese usuario. Si vas a crear una cuenta nueva, asegúrate de configurar su contraseña y conservar acceso de administrador antes de cerrar la sesión. No escribas contraseñas en el repositorio.

### equipo.nix

[equipo.nix](../equipo.nix) reúne varios ajustes que puedes cambiar sin entrar en los módulos de cada programa.

| Ajuste | Qué cambia |
| --- | --- |
| `canal` | `stable` para la rama estable de NixOS o `unstable` para la que recibe paquetes más recientes. |
| `arquitectura` | El tipo de procesador. La Ally usa `x86_64-linux`. |
| `habilitarUmbriel` | Activa o desactiva Umbriel. |
| `sesionPredeterminada` | El escritorio o sesión seleccionado al iniciar. |
| `inicioAutomaticoSteam` | Permite iniciar la sesión de Steam sin pasar por la pantalla de inicio de sesión. |
| `steamConMillennium` | Activa la integración de Millennium con Steam. |
| `navegadorPredeterminado` | El navegador que se abre por defecto. |
| `pantalla` | Nombre de la pantalla, resolución, frecuencia y escala. |
| `gamescopeForzarComposicion` | Permite usar la luz nocturna en el modo consola. |

### configuracion.nix

En [configuracion.nix](../configuracion.nix) puedes cambiar el idioma y la zona horaria.

También encontrarás `system.stateVersion`. Este valor indica la versión de NixOS con la que se creó originalmente la configuración del equipo. No es la versión actual del sistema. Conserva el valor de tu instalación inicial, incluso si actualizas NixOS.

### flake.nix

[flake.nix](../flake.nix) reúne los módulos y las dependencias. La configuración de este equipo se llama `veskalia`.

Si quieres cambiar el nombre, revisa `nixosConfigurations.veskalia`, `networking.hostName` y las demás referencias al nombre:

```sh
git grep -n -i veskalia -- '*.nix' '*.py' '*.sh' justfile
```

Si lo cambias, sustituye también `veskalia` en los comandos de esta guía y en el [justfile](../justfile).

### NixDeck

La microSD de los emuladores se reconoce por la etiqueta NixDeck. No hace falta cambiar su UUID en la configuración. Sigue la [guía de NixDeck](nixdeck.md) después de instalar Korunix.

### Conectar el iPhone

### Otros equipos

La carpeta [plantilla-equipo](../plantilla-equipo) contiene archivos de ejemplo para organizar la configuración de otro equipo. No se instala por sí sola: hay que incorporarla a `flake.nix` y revisar los módulos compartidos.

## 4. Aplicar la configuración

Una vez adaptados los archivos, comprueba qué has cambiado:

```sh
git status --short
```

Si creaste archivos nuevos que Nix necesita, añádelos con `git add ruta/del/archivo.nix`. Los flakes que están dentro de un repositorio Git no leen los archivos nuevos que Git todavía no conoce. No añadas archivos personales ni contraseñas.

Si falta una dependencia en `flake.lock`, puedes registrarla con `nix flake lock`. No hace falta actualizar todas las dependencias para instalar Korunix. Si mantienes tu propia copia del repositorio, guarda también el archivo `flake.lock` actualizado para que puedas volver a utilizar esas mismas versiones.

Primero comprueba la configuración:

```sh
nix flake check --no-build
```

Luego construye el sistema sin cambiar el que estás utilizando:

```sh
nix build .#nixosConfigurations.veskalia.config.system.build.toplevel --no-link
```

Si los dos comandos terminan sin errores, puedes aplicar la configuración:

```sh
sudo nixos-rebuild switch --flake .#veskalia
```

`nixos-rebuild` construye y activa NixOS. `--flake .#veskalia` indica que debe utilizar el flake de la carpeta actual y la configuración llamada `veskalia`.

## 5. Actualizar Korunix

Para descargar los cambios de GitHub, entra en la carpeta del repositorio y ejecuta:

```sh
cd ~/.korunix
git pull --ff-only origin main
```

Si Git termina sin errores, aplica los cambios:

```sh
just switch
```

`git pull` actualiza los archivos. `just switch` actualiza el sistema con esos archivos. Son dos cosas distintas.

Si Git avisa de que tus cambios locales serían sobrescritos, guárdalos antes de actualizar. Puedes usar una rama propia o guardarlos temporalmente con `git stash push -u`, hacer el pull y recuperarlos con `git stash apply`. Si aparecen conflictos, resuélvelos antes de reconstruir. No borres el stash hasta comprobar que recuperaste tus cambios.

### Actualizar los programas

Las versiones de Nixpkgs y los demás proyectos se guardan en `flake.lock`. Para actualizar las dependencias:

```sh
just actualizar
```

Este comando ejecuta `nix flake update`. Revisa los cambios en `flake.lock` y comprueba que el sistema se construye antes de activarlo.

Si solo quieres actualizar Nixpkgs de la rama unstable:

```sh
nix flake update nixpkgs
```

## 6. Comandos rápidos

Just es una herramienta que permite guardar comandos largos bajo nombres cortos. Korunix la instala junto con el sistema. Los comandos están definidos en el [justfile](../justfile) y se ejecutan desde la carpeta del repositorio.

| Comando | Qué hace |
| --- | --- |
| `just revisar` | Comprueba que Nix puede leer y evaluar la configuración. |
| `just probar` | Construye el sistema sin activarlo. |
| `just switch` | Construye y activa la configuración. |
| `just actualizar` | Actualiza las versiones de las dependencias. |
| `just formatear` | Ordena el formato de los archivos Nix. |
| `just volver` | Vuelve a la generación anterior del sistema. |
| `just limpiar` | Elimina generaciones de más de 30 días y optimiza el almacén de Nix. |

## 7. Si algo deja de funcionar

Cada vez que aplicas una configuración, NixOS guarda una versión del sistema llamada generación. Puedes volver a una anterior sin reinstalar NixOS.

Para probar un cambio sin convertirlo en la configuración predeterminada del próximo arranque:

```sh
sudo nixos-rebuild test --flake .#veskalia
```

Esto sí activa los cambios en la sesión actual. Si algo deja de funcionar, puedes volver a arrancar con la configuración anterior.

Si ya aplicaste los cambios con `just switch`, puedes volver a la generación anterior:

```sh
just volver
```

Si NixOS no arranca, elige una generación anterior desde el menú de arranque.

Korunix elimina automáticamente cada semana las generaciones de más de 30 días. Una vez eliminadas, ya no puedes seleccionarlas desde ese menú.

Las generaciones no recuperan documentos, partidas, contraseñas ni archivos personales. Tampoco revierten cambios de BIOS.

Para cambiar programas, servicios o la apariencia, sigue con [Estructura y personalización](personalizacion.md).

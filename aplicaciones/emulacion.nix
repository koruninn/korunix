{
  config,
  lib,
  pkgs,
  ...
}: let
  usuario = config.users.users.koru;
  montaje = "/run/media/${usuario.name}/NixDeck";
  raiz = montaje;
  fuentesAppImages = pkgs.writeText "nixdeck-appimages.json" (builtins.toJSON (import ./appimages.nix).nixdeck);

  comprobarTarjeta = pkgs.writeShellScript "korunix-nixdeck-comprobar-tarjeta" ''
    export PATH=${lib.makeBinPath [pkgs.util-linux]}:$PATH
    exec ${lib.getExe pkgs.python3} ${./nixdeck-tarjeta.py} comprobar ${lib.escapeShellArg montaje} ${lib.escapeShellArg usuario.name}
  '';

  actualizarAppImages = pkgs.writeShellScript "korunix-nixdeck-actualizar-appimages" ''
    set -eu
    ${comprobarTarjeta}
    export PATH=${lib.makeBinPath [pkgs.curl]}:$PATH
    exec ${lib.getExe pkgs.python3} ${./nixdeck-appimages.py} ${lib.escapeShellArg raiz} ${fuentesAppImages}
  '';

  organizarDatos = pkgs.writeShellScript "korunix-nixdeck-organizar-datos" ''
    export PATH=${lib.makeBinPath [pkgs.coreutils pkgs.util-linux pkgs.diffutils]}
    configurar() { ${lib.getExe pkgs.python3} ${./nixdeck-config.py} "$@"; }
    ${builtins.readFile ./nixdeck-datos.sh}
  '';

  preparar = pkgs.writeShellScript "korunix-emulacion-datos" ''
    set -eu
    montaje=${lib.escapeShellArg montaje}
    raiz=${lib.escapeShellArg raiz}
    export PATH=${lib.makeBinPath [pkgs.coreutils pkgs.util-linux]}:/run/current-system/sw/bin:$PATH

    ${comprobarTarjeta}
    if [ ! -w "$montaje" ]; then
      echo "Korunix: no tienes permisos de escritura en $montaje." >&2
      exit 1
    fi

    mkdir -p "$raiz"/{bios,saves,states,mods,storage,screenshots,shaders,texture_packs,appimages,emulators}
    mkdir -p "$raiz"/roms/{switch,n3ds,nds,gba,gc,wii,wiiu,psx,ps2,ps3,psp,xbox360}

    mkdir -p "$raiz"/bios/{psx,ps2,ps3,switch,nds,dsi,n3ds,wiiu,gc,wii,gba,psp}
    mkdir -p "$raiz"/saves/{psx,ps2,ps3,psp,switch,n3ds,nds,gba,gc,wii,wiiu,xbox360}
    mkdir -p "$raiz"/states/{psx,ps2,psp,n3ds,nds,gba,dolphin}

    if [ "$1" = srm ]; then
      shift
      exec ${lib.getExe pkgs.python3} ${./nixdeck-config.py} "$raiz" srm ${lib.escapeShellArg usuario.home}
    fi

    if [ "$1" = organizar ]; then
      ${organizarDatos} "$raiz" organizar
      ${lib.getExe pkgs.python3} ${./nixdeck-config.py} "$raiz" srm ${lib.escapeShellArg usuario.home}
      printf '%s\n' "$raiz"
      exit 0
    fi

    emulador="$1"
    shift
    case "$emulador" in
      azahar|cemu|dolphin|eden|melonds|mgba|pcsx2|ppsspp|rpcs3|xenia|duckstation) ;;
      *) echo "Emulador desconocido: $emulador" >&2; exit 1 ;;
    esac

    datos="$raiz/emulators/$emulador"
    mkdir -p "$datos"/{home,config,data,cache,state}
    ${organizarDatos} "$raiz" "$emulador"
    export HOME="$datos/home"
    export XDG_CONFIG_HOME="$datos/config"
    export XDG_DATA_HOME="$datos/data"
    export XDG_CACHE_HOME="$datos/cache"
    export XDG_STATE_HOME="$datos/state"
    exec "$@"
  '';

  emuladores = [
    { id = "azahar"; nombre = "Azahar"; paquete = pkgs.azahar; }
    { id = "cemu"; nombre = "Cemu"; paquete = pkgs.cemu; }
    { id = "dolphin"; nombre = "Dolphin"; paquete = pkgs.dolphin-emu; }
    { id = "eden"; nombre = "Eden"; paquete = pkgs.eden; }
    { id = "melonds"; nombre = "melonDS"; paquete = pkgs.melonds; }
    { id = "mgba"; nombre = "mGBA"; paquete = pkgs.mgba; }
    { id = "pcsx2"; nombre = "PCSX2"; paquete = pkgs.pcsx2; }
    { id = "ppsspp"; nombre = "PPSSPP"; paquete = pkgs.ppsspp; }
    { id = "rpcs3"; nombre = "RPCS3"; paquete = pkgs.rpcs3; }
  ];

  lanzador = emulador:
    pkgs.writeShellScriptBin "korunix-${emulador.id}" ''
      exec ${preparar} ${emulador.id} ${lib.getExe emulador.paquete} "$@"
    '';

  escritorio = emulador: let
    comando = pkgs.writeShellScript "korunix-escritorio-${emulador.id}" ''
      export PATH="${emulador.paquete}/bin:$PATH"
      exec ${preparar} ${emulador.id} "$@"
    '';
  in pkgs.runCommand "korunix-escritorio-${emulador.id}" {} ''
    mkdir -p "$out/share/applications"
    for dir in icons pixmaps; do
      if [ -d "${emulador.paquete}/share/$dir" ]; then
        ln -s "${emulador.paquete}/share/$dir" "$out/share/$dir"
      fi
    done

    for desktop in ${emulador.paquete}/share/applications/*.desktop; do
      [ -f "$desktop" ] || continue
      sed \
        -e 's|^Exec=|Exec=${comando} |' \
        -e 's|^TryExec=.*|TryExec=${comando}|' \
        -e 's|^DBusActivatable=true|DBusActivatable=false|' \
        "$desktop" > "$out/share/applications/$(basename "$desktop")"
    done
  '';

  # Xenia se descarga en la microSD.
  xenia = pkgs.writeShellScriptBin "korunix-xenia" ''
    set -eu
    raiz=${lib.escapeShellArg raiz}
    ${comprobarTarjeta}
    case "''${1:-}" in
      *.[Aa][Pp][Pp][Ii][Mm][Aa][Gg][Ee]) imagen="$1"; shift ;;
      *)
        shopt -s nullglob nocaseglob
        candidatos=("$raiz"/appimages/*xenia*.appimage)
        if [ "''${#candidatos[@]}" -ne 1 ]; then
          echo "NixDeck: deja un único AppImage de Xenia en $raiz/appimages o indica su ruta." >&2
          exit 1
        fi
        imagen="''${candidatos[0]}"
        ;;
    esac
    exec ${preparar} xenia ${lib.getExe pkgs.appimage-run} "$imagen" \
      --storage_root="$raiz/storage/xbox360" --content_root="$raiz/saves/xbox360" "$@"
  '';

  srm = pkgs.writeShellScriptBin "steam-rom-manager" ''
    set -eu
    ${preparar} srm
    export PORTABLE_EXECUTABLE_DIR=${lib.escapeShellArg "${raiz}/storage/steam-rom-manager"}
    exec ${lib.getExe pkgs.steam-rom-manager} "$@"
  '';

  duckstationSteamRun = (pkgs.steam.override {
    extraPkgs = p: lib.unique (
      (p.appimageTools.defaultFhsEnvArgs.targetPkgs p)
      ++ (p.appimageTools.defaultFhsEnvArgs.multiPkgs p)
      ++ [p.e2fsprogs.out (lib.getLib p.libgpg-error)]
    );
  }).run;

  # DuckStation necesita montar el AppImage sin extraerlo.
  duckstationRuntime = pkgs.writeShellScript "korunix-duckstation-runtime" ''
    set -eu
    imagen="$1"
    shift
    if [ ! -x "$imagen" ]; then
      echo "NixDeck: AppImage ausente o sin permiso de ejecución: $imagen" >&2
      exit 1
    fi
    export FUSERMOUNT_PROG=/run/wrappers/bin/fusermount3
    unset APPIMAGE_EXTRACT_AND_RUN
    temporal=$(${pkgs.coreutils}/bin/mktemp -d "$XDG_CACHE_HOME/duckstation-mount.XXXXXXXX")
    ruta="$temporal/mount-path"
    runtime="$temporal/runtime"
    appdir=""
    pid=""

    limpiar() {
      if [ -n "$pid" ]; then
        kill "$pid" 2>/dev/null || true
        wait "$pid" 2>/dev/null || true
      fi
      if [ -n "$appdir" ] && ${pkgs.util-linux}/bin/mountpoint -q "$appdir"; then
        /run/wrappers/bin/fusermount3 -u "$appdir" || true
      fi
      rm -f -- "$ruta" "$runtime"
      rmdir -- "$temporal" 2>/dev/null || true
    }
    trap limpiar EXIT
    trap 'exit 130' INT
    trap 'exit 143' TERM

    # Evitar que el lanzador de AppImages intervenga.
    if ! ${pkgs.coreutils}/bin/ln -- "$imagen" "$runtime" 2>/dev/null; then
      ${pkgs.coreutils}/bin/cp --reflink=auto --preserve=mode -- "$imagen" "$runtime"
    fi
    TMPDIR="$temporal" "$runtime" --appimage-mount > "$ruta" &
    pid=$!
    for intento in $(${pkgs.coreutils}/bin/seq 1 100); do
      if [ -s "$ruta" ]; then
        IFS= read -r appdir < "$ruta" || true
        if [ -n "$appdir" ] && [ -x "$appdir/AppRun" ] &&
          ${pkgs.util-linux}/bin/mountpoint -q "$appdir"; then
          break
        fi
      fi
      if ! kill -0 "$pid" 2>/dev/null; then
        echo "NixDeck: DuckStation no pudo montar el AppImage." >&2
        exit 1
      fi
      sleep 0.1
    done
    if [ -z "$appdir" ] || [ ! -x "$appdir/AppRun" ] ||
      ! ${pkgs.util-linux}/bin/mountpoint -q "$appdir"; then
      echo "NixDeck: no se encontró el montaje FUSE de DuckStation." >&2
      exit 1
    fi

    ${lib.getExe duckstationSteamRun} env -u LD_LIBRARY_PATH \
      APPIMAGE="$imagen" APPDIR="$appdir" "$appdir/AppRun" "$@"
  '';

  duckstation = pkgs.writeShellScriptBin "korunix-duckstation" ''
    imagen=${lib.escapeShellArg "${raiz}/appimages/DuckStation-x64.AppImage"}
    # Conservar los accesos directos existentes.
    case "''${1:-}" in
      *.[Aa][Pp][Pp][Ii][Mm][Aa][Gg][Ee]) imagen="$1"; shift ;;
    esac
    exec ${preparar} duckstation ${duckstationRuntime} "$imagen" "$@"
  '';

  xeniaDesktop = pkgs.makeDesktopItem {
    name = "korunix-xenia";
    desktopName = "Xenia";
    exec = "korunix-xenia %f";
    icon = "applications-games";
    categories = ["Game" "Emulator"];
    terminal = false;
  };

  duckstationDesktop = pkgs.makeDesktopItem {
    name = "org.duckstation.DuckStation";
    desktopName = "DuckStation";
    exec = "korunix-duckstation %f";
    icon = "org.duckstation.DuckStation";
    categories = ["Game" "Emulator"];
    terminal = false;
  };

  nixdeck = pkgs.writeShellScriptBin "nixdeck" ''
    set -eu

    ayuda() {
      cat <<'EOF'
NixDeck

  nixdeck configurar       Preparar la microSD.
  nixdeck sincronizar      Añadir los juegos a Steam.
  nixdeck steam            Abrir Steam ROM Manager.
  nixdeck firmware         Instalar el firmware de PS3.
EOF
    }

    if [ "$(id -u)" = 0 ]; then
      echo "NixDeck: ejecuta este comando sin sudo." >&2
      exit 1
    fi

    accion="''${1:-ayuda}"
    if [ "$#" -gt 0 ]; then shift; fi
    if [ "$#" -ne 0 ]; then
      ayuda >&2
      exit 1
    fi

    case "$accion" in
      configurar)
        export PATH=${lib.makeBinPath [pkgs.util-linux pkgs.e2fsprogs pkgs.systemd pkgs.coreutils]}:$PATH
        ${lib.getExe pkgs.python3} ${./nixdeck-tarjeta.py} configurar ${lib.escapeShellArg montaje} ${lib.escapeShellArg usuario.name}
        ${preparar} organizar
        exec ${actualizarAppImages}
        ;;
      sincronizar)
        if ${pkgs.procps}/bin/pgrep -u "$(id -u)" -x steam >/dev/null ||
           ${pkgs.procps}/bin/pgrep -u "$(id -u)" -x steam-rom-manager >/dev/null; then
          echo "NixDeck: cierra Steam y Steam ROM Manager antes de sincronizar." >&2
          exit 1
        fi
        ${preparar} organizar
        ${actualizarAppImages}
        exec ${srm}/bin/steam-rom-manager add
        ;;
      steam)
        exec ${srm}/bin/steam-rom-manager
        ;;
      firmware)
        ${comprobarTarjeta}
        firmware=${lib.escapeShellArg "${raiz}/bios/ps3/PS3UPDAT.PUP"}
        if [ ! -f "$firmware" ]; then
          echo "NixDeck: coloca el firmware oficial en $firmware" >&2
          exit 1
        fi
        exec ${preparar} rpcs3 ${lib.getExe pkgs.rpcs3} --installfw "$firmware"
        ;;
      ayuda|-h|--help)
        ayuda
        ;;
      *)
        echo "NixDeck: acción desconocida: $accion" >&2
        ayuda >&2
        exit 1
        ;;
    esac
  '';


in {
  # La Ally puede arrancar sin la microSD.
  fileSystems."${montaje}" = {
    device = "/dev/disk/by-label/NixDeck";
    fsType = "ext4";
    options = ["nofail" "x-systemd.automount" "x-systemd.device-timeout=5s"];
  };

  environment.sessionVariables = {
    PEPOMOTE_DOLPHIN_DIR = "${raiz}/emulators/dolphin/config/dolphin-emu";
    PEPOMOTE_CEMU_DIR = "${raiz}/emulators/cemu/config/Cemu";
    PEPOMOTE_EDEN_DIR = "${raiz}/emulators/eden/config/eden";
  };

  environment.systemPackages =
    [
      pkgs.steam-rom-manager
      (lib.hiPrio srm)
      nixdeck
      xenia
      xeniaDesktop
      duckstation
      duckstationDesktop
    ]
    ++ map lanzador emuladores
    ++ map escritorio emuladores;
}

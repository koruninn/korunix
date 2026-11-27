{equipo, lib, pkgs, ...}: let
  salirDeSteam = pkgs.writeShellScriptBin "steamos-session-select" ''
    estado="''${XDG_STATE_HOME:-$HOME/.local/state}"
    ${pkgs.coreutils}/bin/mkdir -p "$estado/korunix"
    exec >>"$estado/korunix/steam-session-select.log" 2>&1
    echo "$(${pkgs.coreutils}/bin/date -Is): selector solicitado ($*)"

    # No cerrar otros escritorios ni una ventana normal de Steam.
    if [ "''${KORUNIX_STEAM_SESSION:-}" != 1 ]; then
      echo "Korunix: el selector solo funciona en la sesión Steam de Gamescope."
      exit 1
    fi
    case "''${1:-desktop}" in
      # SteamOS llama plasma al modo escritorio, aunque no se use KDE.
      desktop|plasma) ;;
      *) echo "Korunix: modo no admitido: $1"; exit 1 ;;
    esac

    sesion="''${XDG_SESSION_ID:-self}"
    uid="$(${pkgs.coreutils}/bin/id -u)"
    propietario="$(${pkgs.systemd}/bin/loginctl show-session "$sesion" -p User --value)"
    tipo="$(${pkgs.systemd}/bin/loginctl show-session "$sesion" -p Type --value)"
    if [ "$propietario" != "$uid" ] || [ "$tipo" != wayland ]; then
      echo "Korunix: no se pudo verificar la sesión gráfica propia ($sesion)."
      exit 1
    fi

    sesion="$(${pkgs.systemd}/bin/loginctl show-session "$sesion" -p Id --value)"
    [ -n "$sesion" ] || exit 1
    echo "Korunix: solicitando fin de la sesión $sesion para regresar a GDM."
    # Volver a la pantalla de inicio de sesión.
    exec ${pkgs.systemd}/bin/loginctl --no-ask-password terminate-session "$sesion"
  '';
in {
  programs.steam = {
    gamescopeSession = {
      enable = true;
      # Permitir la luz nocturna en el modo consola.
      args = ["--xwayland-count" "2"] ++ lib.optional (equipo.gamescopeForzarComposicion or true) "--force-composition";
      steamArgs = ["-gamepadui" "-steamos3" "-steampal" "-steamdeck" "-pipewire-dmabuf"];
      env = {
        KORUNIX_STEAM_SESSION = "1";
        STEAM_GAMESCOPE_COLOR_MANAGED = "1";
        STEAM_GAMESCOPE_VIRTUAL_WHITE = "1";
        STEAM_MULTIPLE_XWAYLANDS = "1";
        STEAM_ENABLE_VOLUME_HANDLER = "1";
        STEAM_DISPLAY_REFRESH_LIMITS = "40,120";
      };
    };
    extraPackages = [salirDeSteam];
  };
  environment.systemPackages = [salirDeSteam];

  services.displayManager = {
    defaultSession = equipo.sesionPredeterminada or "steam";
    autoLogin = {
      enable = equipo.inicioAutomaticoSteam or false;
      user = "koru";
    };
    gdm.autoLogin.delay = 10;
  };
}

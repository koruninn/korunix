{equipo, inputs, lib, pkgs, ...}: let
  recuperarGOA = pkgs.writeShellScript "korunix-goa-after-umbriel" ''
    set -eu
    marca="$1/korunix-goa-after-umbriel"
    [ -f "$marca" ] || exit 0

    busctl=${pkgs.systemd}/bin/busctl
    propietario() {
      "$busctl" --user --timeout=2s call org.freedesktop.DBus /org/freedesktop/DBus org.freedesktop.DBus GetConnectionUnixProcessID s org.gnome.OnlineAccounts 2>/dev/null
    }

    # Esperar a que se desbloquee el llavero.
    for intento in {1..20}; do
      if [ "$("$busctl" --user --timeout=2s get-property org.freedesktop.secrets /org/freedesktop/secrets/collection/login org.freedesktop.Secret.Collection Locked 2>/dev/null || true)" = "b false" ]; then
        break
      fi
      if [ "$intento" = 20 ]; then
        echo "Korunix: el llavero Login aún no está disponible; se conserva la marca."
        exit 0
      fi
      ${pkgs.coreutils}/bin/sleep 1
    done

    respuesta="$(propietario || true)"
    if [ -n "$respuesta" ]; then
      read -r tipo pid <<< "$respuesta"
      if [ "$tipo" != u ] || ! [[ "$pid" =~ ^[0-9]+$ ]] || [ "$pid" -le 1 ]; then
        echo "Korunix: no se pudo verificar el PID de GOA."
        exit 0
      fi
      if [ "$(propietario || true)" = "$respuesta" ]; then
        kill -TERM "$pid" 2>/dev/null || true
      fi
      for intento in {1..20}; do
        if [ "$(propietario || true)" != "$respuesta" ]; then
          break
        fi
        if [ "$intento" = 20 ]; then
          echo "Korunix: GOA no terminó; se conserva la marca."
          exit 0
        fi
        ${pkgs.coreutils}/bin/sleep 0.25
      done
      if ! "$busctl" --user --timeout=10s call org.gnome.OnlineAccounts /org/gnome/OnlineAccounts org.freedesktop.DBus.Peer Ping; then
        exit 0
      fi
      echo "Korunix: GOA recuperado tras salir de Umbriel (PID anterior: $pid)."
    fi
    ${pkgs.coreutils}/bin/rm -f "$marca"
  '';
in {
  imports = [
    inputs.noctalia.nixosModules.default
    inputs.umbriel.nixosModules.default
    ../apariencia/escritorios/noctalia/umbriel.nix
    ../apariencia/escritorios/noctalia/noctalia.nix
    ../apariencia/escritorios/noctalia/luciole.nix
    ../apariencia/escritorios/noctalia/plantillas.nix
    ../apariencia/escritorios/noctalia/fondos-dia-noche.nix
    ../apariencia/escritorios/noctalia/qt.nix
    ../apariencia/escritorios/noctalia/portales.nix
  ];

  config = lib.mkIf (equipo.habilitarUmbriel or false) {
    programs.noctalia.systemd.enable = false;

    # Recuperar las cuentas en línea al volver a GNOME.
    systemd.user.services.umbriel.serviceConfig.ExecStopPost = [
      "${pkgs.coreutils}/bin/touch %t/korunix-goa-after-umbriel"
    ];
    systemd.user.services.korunix-goa-after-umbriel = {
      description = "Recuperar GNOME Online Accounts tras salir de Umbriel";
      wantedBy = ["gnome-session.target"];
      partOf = ["gnome-session.target"];
      unitConfig.ConditionPathExists = "%t/korunix-goa-after-umbriel";
      serviceConfig = {
        Type = "exec";
        ExecStart = "${recuperarGOA} %t";
      };
    };
    xdg.portal.extraPortals = [pkgs.xdg-desktop-portal-gtk];
    environment.systemPackages = with pkgs; [brightnessctl playerctl];
  };
}

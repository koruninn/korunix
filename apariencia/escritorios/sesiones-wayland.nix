{
  lib,
  pkgs,
  ...
}: let
  sesionWayland = {
    nombre,
    paquete,
    sesion,
  }:
    pkgs.runCommand nombre {
      passthru.providedSessions = [sesion];
    } ''
      origen=${paquete}/share/wayland-sessions/${sesion}.desktop

      if [ ! -f "$origen" ]; then
        echo "No existe la sesión Wayland ${sesion}.desktop en ${paquete}" >&2
        exit 1
      fi

      mkdir -p "$out/share/wayland-sessions"
      cp "$origen" "$out/share/wayland-sessions/${sesion}.desktop"
    '';

  plasma = sesionWayland {
    nombre = "korunix-plasma-wayland-session";
    paquete = pkgs.kdePackages.plasma-workspace.sessions;
    sesion = "plasma";
  };

  gnome = sesionWayland {
    nombre = "korunix-gnome-wayland-session";
    paquete = pkgs.gnome-session.sessions;
    sesion = "gnome";
  };
in {
  # Korunix expone únicamente sesiones Wayland. Niri y Umbriel ya publican una
  # sola sesión; Plasma y GNOME se filtran para evitar variantes adicionales.
  services.displayManager.sessionPackages = lib.mkForce [
    pkgs.niri
    pkgs.umbriel
    plasma
    gnome
  ];
}

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

  hyprland = sesionWayland {
    nombre = "korunix-hyprland-session";
    paquete = pkgs.hyprland;
    sesion = "hyprland";
  };

  plasma = sesionWayland {
    nombre = "korunix-plasma-wayland-session";
    paquete = pkgs.kdePackages.plasma-workspace.sessions;
    sesion = "plasma";
  };
in {
  # El selector de Korunix expone únicamente sesiones Wayland. Niri y Umbriel
  # ya publican una sola sesión; Hyprland y Plasma se filtran para ocultar sus
  # variantes UWSM/X11 sin quitar XWayland para aplicaciones antiguas.
  services.displayManager.sessionPackages = lib.mkForce [
    pkgs.niri
    pkgs.umbriel
    hyprland
    plasma
  ];
}

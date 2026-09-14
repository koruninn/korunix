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

  gnome = sesionWayland {
    nombre = "korunix-gnome-wayland-session";
    paquete = pkgs.gnome-session.sessions;
    sesion = "gnome";
  };
in {
  # Korunix expone únicamente Niri, Umbriel y GNOME en Wayland.
  services.displayManager.sessionPackages = lib.mkForce [
    pkgs.niri
    pkgs.umbriel
    gnome
  ];
}

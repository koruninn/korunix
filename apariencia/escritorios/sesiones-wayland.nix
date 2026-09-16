{
  inputs,
  lib,
  pkgs,
  ...
}: let
  system = pkgs.stdenv.hostPlatform.system;
  umbriel = inputs.umbriel.packages.${system}.default;

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
  # Decisión final de Korunix: solo Umbriel y GNOME como sesiones gráficas.
  services.displayManager.sessionPackages = lib.mkForce [
    umbriel
    gnome
  ];
}

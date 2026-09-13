{pkgs, ...}: let
  hyprlandSinUwsm = pkgs.hyprland.overrideAttrs (old: {
    postInstall = (old.postInstall or "") + ''
      rm -f "$out/share/wayland-sessions/hyprland-uwsm.desktop"
    '';

    passthru = (old.passthru or {}) // {
      providedSessions = ["hyprland"];
    };
  });
in {
  # Noctalia Greeter escanea directamente /run/current-system/sw/share/wayland-sessions.
  # Hyprland instala también hyprland-uwsm.desktop aunque Korunix tenga UWSM
  # desactivado. Eliminamos únicamente esa entrada del paquete usado por Korunix;
  # Hyprland normal, XWayland y el portal de Hyprland siguen intactos.
  programs.hyprland.package = hyprlandSinUwsm;
}

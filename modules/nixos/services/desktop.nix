{
  config,
  pkgs,
  lib,
  ...
}:

{
  # Servidor X (necesario para muchas apps aunque uses Wayland)
  services.xserver.enable = true;
  services.xserver.xkb = {
    layout = "es";
    variant = "deadtilde";
  };
  console.keyMap = "es";

  # --- GDM ---
  services.displayManager.gdm.enable = true;
  services.xserver.desktopManager.gnome.enable = true;

  # --- Niri Window Manager ---
  programs.niri.enable = true;
}

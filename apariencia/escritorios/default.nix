{ config, pkgs, lib, ... }:

{
  imports = [
    ./plasma.nix
    ./noctalia
  ];

  # Servidor X (necesario para compatibilidad de teclado/XWayland)
  services.xserver.enable = true;
  services.xserver.xkb = {
    layout = "es";
    variant = "deadtilde";
  };
  console.keyMap = "es";

  # Gestor de inicio de sesión SDDM
  services.displayManager.sddm = {
  enable = true;
  wayland.enable = true; # Ejecuta SDDM nativamente en Wayland
 };
}

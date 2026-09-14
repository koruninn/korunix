{
  imports = [
    ./plasma.nix
    ./sesiones-wayland.nix
    ./noctalia
  ];

  # Servidor X para compatibilidad con aplicaciones y XWayland.
  services.xserver.enable = true;
  services.xserver.xkb = {
    layout = "es";
    variant = "deadtilde";
  };
  console.keyMap = "es";
}

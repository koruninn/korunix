{
  imports = [
    ./gnome.nix
    ./gnome-blur.nix
    ./sesiones-wayland.nix
    ./noctalia
  ];

  # Activamos el soporte de X para que las aplicaciones antiguas y XWayland sigan funcionando aunque el escritorio use Wayland.
  services.xserver.enable = true;
  services.xserver.xkb = {
    layout = "es";
    variant = "deadtilde";
  };
  console.keyMap = "es";
}

{ pkgs, ... }:

{
  # Activar KDE Plasma 6 únicamente como sesión Wayland.
  services.desktopManager.plasma6.enable = true;

  # KWin X11 no es necesario para ejecutar aplicaciones X11 dentro de Wayland;
  # esa compatibilidad la proporciona XWayland.
  environment.plasma6.excludePackages = [
    pkgs.kdePackages.kwin-x11
  ];

  # Num Lock encendido al iniciar Plasma.
  environment.etc."xdg/kcminputrc".text = ''
[Keyboard]
NumLock=0
'';

  # Configuración del teclado nativo para Wayland en Plasma.
  environment.variables = {
    XKB_DEFAULT_LAYOUT = "es";
    XKB_DEFAULT_VARIANT = "deadtilde";
  };
}


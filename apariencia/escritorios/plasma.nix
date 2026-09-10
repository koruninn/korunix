{ pkgs, ... }:

{
  # Activar el entorno de escritorio KDE Plasma 6
  services.desktopManager.plasma6.enable = true;

  # Configuración del teclado nativo para Wayland en Plasma
  environment.variables = {
    XKB_DEFAULT_LAYOUT = "es";
    XKB_DEFAULT_VARIANT = "deadtilde";
  };
}


{ pkgs, ... }: {
  # 1. Asegura que el paquete de iconos esté instalado en el sistema
  environment.systemPackages = [
    pkgs.hatter-icon-theme
  ];

  # 2. Inyecta el tema de iconos por defecto en las rutas globales de configuración XDG
  environment.etc = {
    "gtk-3.0/settings.ini".text = ''
      [Settings]
      gtk-icon-theme-name=Hatter-Green
    '';
    "gtk-4.0/settings.ini".text = ''
      [Settings]
      gtk-icon-theme-name=Hatter-Green
    '';
  };
}

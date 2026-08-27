# Valores iniciales de producto para una instalación nueva de Korunix.
#
# Este archivo contiene únicamente decisiones de producto que deben compartir
# el motor, los módulos NixOS y la interfaz. Los hechos detectados del equipo
# —hardware, localización y stateVersion— no son defaults y viven en sus
# respectivas fuentes.
{
  schemaVersion = 1;

  desktop = {
    primary = "niri";
    additional = [];
  };

  applications = {
    # Herramientas que forman parte de la experiencia inicial en cualquier
    # arquitectura soportada.
    common = [
      "firefox"
      "xwayland-satellite"
      "git"
      "just"
      "tree"
      "wget"
    ];

    # La suite ofimática depende de la arquitectura. La aplicación sigue siendo
    # una decisión del producto, pero no forzamos un paquete incompatible.
    bySystem = {
      "x86_64-linux" = [
        "onlyoffice-desktopeditors"
      ];

      "aarch64-linux" = [
        "libreoffice"
      ];
    };
  };

  services = {
    # Descubrimiento local y SSH forman parte de la instalación inicial.
    avahi = true;
    ssh = true;

    # Estas capacidades necesitan hardware, periféricos o una decisión humana.
    bluetooth = false;
    sunshine = false;
    printing = false;
    virtualization = false;

    printingDriver = null;
  };
}

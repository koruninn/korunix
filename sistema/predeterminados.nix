# ARCHIVO INTERNO DE KORUNIX.
#
# ¿Qué es?
# Guarda las elecciones iniciales que Korunix propone para una instalación nueva.
#
# ¿Para qué sirve?
# Evita que el motor, Nix y la interfaz inventen valores iniciales distintos.
# Todos parten de la misma lista y una persona puede cambiarlos después desde
# la configuración de su computadora.
#
# ¿Debes editarlo para personalizar tu equipo?
# No. Este archivo define el punto de partida del producto, no tus preferencias.
#
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

  # Los roles describen intenciones humanas, no paquetes Nix ni archivos
  # .desktop. La traducción técnica vive en sistema/roles.nix.
  roles = {
    common = {
      terminal = "alacritty";
      shell = "fish";
      mail = "thunderbird";
      photoEditor = "gimp";
    };

    byDesktop = {
      niri = {
        fileManager = "nautilus";
        imageViewer = "loupe";
        pdfViewer = "papers";
        textEditor = "gnome-text-editor";
        videoPlayer = "showtime";
        musicPlayer = "gnome-music";
        calendar = "gnome-calendar";
        maps = "gnome-maps";
        camera = "snapshot";
        calculator = "gnome-calculator";
        archiveManager = "file-roller";
      };

      hyprland = {
        fileManager = "nautilus";
        imageViewer = "loupe";
        pdfViewer = "papers";
        textEditor = "gnome-text-editor";
        videoPlayer = "showtime";
        musicPlayer = "gnome-music";
        calendar = "gnome-calendar";
        maps = "gnome-maps";
        camera = "snapshot";
        calculator = "gnome-calculator";
        archiveManager = "file-roller";
      };

      cinnamon = {
        fileManager = "nemo";
        imageViewer = "xviewer";
        pdfViewer = "xreader";
        textEditor = "xed";
        videoPlayer = "celluloid";
        musicPlayer = "rhythmbox";
        calendar = "gnome-calendar";
        maps = "gnome-maps";
        camera = "snapshot";
        calculator = "gnome-calculator";
        archiveManager = "file-roller";
      };

      plasma = {
        fileManager = "dolphin";
        imageViewer = "gwenview";
        pdfViewer = "okular";
        textEditor = null;
        videoPlayer = "haruna";
        musicPlayer = "elisa";
        calendar = "merkuro";
        maps = "marble";
        camera = "kamoso";
        calculator = "kcalc";
        archiveManager = "ark";
      };
    };

    choices = {
      browser = [
        "firefox"
        "google-chrome"
      ];

      plasmaTextEditor = [
        "kwrite"
        "kate"
      ];
    };
  };

  services = {
    # El descubrimiento local forma parte de la instalación inicial. SSH no
    # aparece aquí porque ya no es una preferencia: Korunix lo mantiene activo.
    avahi = true;

    # Estas capacidades necesitan hardware, periféricos o una decisión humana.
    bluetooth = false;
    sunshine = false;
    printing = false;
    virtualization = false;

    printingDriver = null;
  };
}

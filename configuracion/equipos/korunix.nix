# ESTE ARCHIVO SE PUEDE CAMBIAR.
#
# ¿Qué es?
# Aquí guardas cómo quieres que funcione esta computadora.
#
# ¿Qué puedes elegir aquí?
# Entre otras cosas: idioma, teclado, escritorio, programas, servicios y canal
# de actualizaciones. Algunas elecciones son controles principales y pueden
# preparar varias piezas relacionadas al mismo tiempo.
#
# ¿Qué NO vive aquí?
# El hardware que Korunix descubre por sí solo está en generado/equipos/.
# Las reglas internas que hacen funcionar esas elecciones están en sistema/.
#
# Si una opción cambia varias cosas, el comentario cercano debe decir qué
# cambia, qué no cambia y qué elección suele convenir.
#
{
  # Este archivo contiene decisiones humanas y el contexto estructural mínimo que
  # flake necesita antes de evaluar NixOS. Los hechos físicos viven en generado/equipos/.
  # La plataforma se detecta al adoptar el equipo, pero flake la necesita antes
  # de evaluar los módulos; por eso permanece como contexto estructural.
  system = "x86_64-linux";

  # Un host puede tener varias personas y una misma persona puede aparecer en
  # varios hosts. Cada identificador apunta a configuracion/personas/<identificador>.nix.
  # El host es dueño de la pertenencia y de las decisiones locales. La identidad,
  # preferencias y capacidades portables siguen viviendo en configuracion/personas/koru.nix.
  users = {
    koru = {
      homeDirectory = "/home/koru";
      administrator = true;
      deferredCapabilities = [];
      deferredInputMethods = [];
      preservedGroups = [];
      githubSshIdentityFile = ".ssh/blep";
    };
  };

  korunix = {
    appearance = {
      style = "default";
      mode = "auto";
    };

    enable = true;

    # El canal es una decisión de actualización de este equipo. Este host ya
    # utilizaba nixos-unstable antes de que Korunix modelara esta elección, así
    # que declararlo como inestable conserva exactamente su comportamiento.
    channel = "unstable";

    # hostId identifica este archivo dentro de Korunix. hostName es el nombre que
    # el sistema publica en la red y puede cambiar sin renombrar la estructura.
    hostName = "korunix";

    # Este valor conserva la compatibilidad histórica de NixOS. No representa el
    # canal estable/inestable y no debe actualizarse automáticamente.
    stateVersion = "26.05";

    localization = {
      # El idioma del sistema, los formatos y la región son estado de este host.
      # La preferencia de idioma de cada persona sigue viviendo en configuracion/personas/.
      systemLanguage = "es";
      region = "PE";

      formats = {
        language = "es";
        region = "PE";
      };

      timeZone = "America/Lima";

      keyboard = {
        layout = "es";
        variant = "deadtilde";
        displayNames = ["Español — España" "Español — Latinoamérica"];
        additionalLayouts = ["latam"];
        additionalVariants = [""];
        switchOption = "grp:alt_shift_toggle";
        console = "es";
      };
    };

    # Esta unidad interna se identificó por UUID para que el nombre /dev/sdX
    # pueda cambiar sin romper el acceso. Korunix detectó también los IDs locales
    # de la cuenta que debe poder escribir; no son datos que la interfaz pregunte.
    storage = {
      dataVolumes = [
        {
          id = "datos";
          uuid = "036F8E656FF00FB2";
          fileSystem = "ntfs";
          path = "/mnt/datos";
          ownerUid = 1000;
          ownerGid = 100;
          availableAtLogin = true;
        }
      ];
    };

    desktop = {
      primary = "niri";

      monitor = {
        output = "DP-1";
        mode = "1920x1080";
        refreshRate = 120;
      };

      additional = [
        "hyprland"
        "plasma"
        "cinnamon"
      ];
    };

    # Aquí viven las aplicaciones generales elegidas para este equipo.
    # Las suites propias de Noctalia, Plasma y Cinnamon pertenecen a sus
    # respectivos escritorios y no se duplican en este catálogo.
    applications = [
      "android-tools"
      "birdfont"
      "cohesion"
      "darktable"
      "figma-linux-next"
      "firefox"
      "fontforge"
      "genshin-impact"
      "gimp"
      "git"
      "google-chrome"
      "heroic"
      "honkai-star-rail"
      "inkscape"
      "just"
      "kate"
      "kdenlive"
      "krita"
      "localsend"
      "lutris"
      "obs-studio"
      "obsidian"
      "onlyoffice-desktopeditors"
      "peazip"
      "polyglot"
      "prismlauncher"
      "protonplus"
      "pywalfox-native"
      "rapidraw"
      "rar"
      "scrcpy"
      "spotdl"
      "spotify"
      "steam"
      "thunderbird"
      "tree"
      "unrar"
      "vesktop"
      "vlc"
      "vscode"
      "wget"
      "nixpkgs:legacyPackages.x86_64-linux.blender"
    ];

    services = {
      avahi = true;
      bluetooth = true;
      sunshine = true;
      printing = true;
      virtualization = true;

      # El controlador actual se conserva como decisión de este equipo. Más
      # adelante Korunix podrá proponerlo a partir de la impresora detectada.
      printingDriver = "epson-201207w";
    };
  };
}

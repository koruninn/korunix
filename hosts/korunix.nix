{
  # Este archivo contiene decisiones humanas y el contexto estructural mínimo que
  # flake necesita antes de evaluar NixOS. Los hechos físicos viven en hardware/.
  # La plataforma se detecta al adoptar el equipo, pero flake la necesita antes
  # de evaluar los módulos; por eso permanece como contexto estructural.
  system = "x86_64-linux";

  # Un host puede tener varias personas y una misma persona puede aparecer en
  # varios hosts. Cada identificador apunta a users/<identificador>.nix.
  # El host es dueño de la pertenencia y de las decisiones locales. La identidad,
  # preferencias y capacidades portables siguen viviendo en users/koru.nix.
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
    enable = true;

    # hostId identifica este archivo dentro de Korunix. hostName es el nombre que
    # el sistema publica en la red y puede cambiar sin renombrar la estructura.
    hostName = "korunix";

    # Este valor conserva la compatibilidad histórica de NixOS. No representa el
    # canal estable/inestable y no debe actualizarse automáticamente.
    stateVersion = "26.05";

    localization = {
      # El idioma del sistema, los formatos y la región son estado de este host.
      # La preferencia de idioma de cada persona sigue viviendo en users/.
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
      "figma-linux"
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
    ];

    services = {
      avahi = true;
      bluetooth = true;
      ssh = true;
      sunshine = true;
      printing = true;
      virtualization = true;

      # El controlador actual se conserva como decisión de este equipo. Más
      # adelante Korunix podrá proponerlo a partir de la impresora detectada.
      printingDriver = "epson-201207w";
    };
  };
}

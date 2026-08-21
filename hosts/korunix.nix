{
  # Este archivo contiene decisiones humanas y el contexto estructural mínimo que
  # flake necesita antes de evaluar NixOS. Los hechos físicos viven en hardware/.
  # La plataforma se detecta al adoptar el equipo, pero flake la necesita antes
  # de evaluar los módulos; por eso permanece como contexto estructural.
  system = "x86_64-linux";

  # Un host puede tener varias personas y una misma persona puede aparecer en
  # varios hosts. Cada identificador apunta a users/<identificador>.nix.
  users = [
    "koru"
  ];

  korunix = {
    enable = true;

    # hostId identifica este archivo dentro de Korunix. hostName es el nombre que
    # el sistema publica en la red y puede cambiar sin renombrar la estructura.
    hostName = "korunix";

    # Este valor conserva la compatibilidad histórica de NixOS. No representa el
    # canal estable/inestable y no debe actualizarse automáticamente.
    stateVersion = "26.05";


    localization = {
      language = "es";
      region = "PE";
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

      additional = [
        "hyprland"
        "gnome"
      ];
    };

    # Estas son aplicaciones elegidas por la persona. Alacritty, Fish, Fetch,
    # Nautilus y otras piezas que Korunix usa como parte de una experiencia base
    # se derivan en los módulos correspondientes y no se repiten aquí.
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
      "valent"
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

{
  config,
  inputs,
  lib,
  pkgs,
  ...
}: let
  cfg = config.korunix.applications;

  # Esta tabla resuelve elecciones humanas a paquetes. Las piezas que forman
  # parte de un rol fijo de Korunix, como Alacritty o Fetch, no aparecen aquí.
  packageMap = {
    "android-tools" = pkgs.android-tools;
    birdfont = pkgs.birdfont;
    darktable = pkgs.darktable;
    "figma-linux" = pkgs.figma-linux;
    fontforge = pkgs.fontforge;
    git = pkgs.git;
    gimp = pkgs.gimp;
    "google-chrome" = pkgs.google-chrome;
    heroic = pkgs.heroic;
    inkscape = pkgs.inkscape;
    just = pkgs.just;
    kate = pkgs.kdePackages.kate;
    kdenlive = pkgs.kdePackages.kdenlive;
    krita = pkgs.krita;
    lutris = pkgs.lutris;
    obsidian = pkgs.obsidian;
    "onlyoffice-desktopeditors" = pkgs.onlyoffice-desktopeditors;
    peazip = pkgs.peazip;
    prismlauncher = pkgs.prismlauncher;
    protonplus = pkgs.protonplus;
    "pywalfox-native" = pkgs.pywalfox-native;
    rapidraw = pkgs.rapidraw;
    rar = pkgs.rar;
    scrcpy = pkgs.scrcpy;
    spotdl = pkgs.spotdl;
    thunderbird = pkgs.thunderbird;
    tree = pkgs.tree;
    unrar = pkgs.unrar;
    valent = pkgs.valent;
    vesktop = pkgs.vesktop;
    vlc = pkgs.vlc;
    vscode = pkgs.vscode;
    wget = pkgs.wget;
  };

  specialApplications = [
    "firefox"
    "localsend"
    "obs-studio"
    "spotify"
    "steam"
    "genshin-impact"
    "honkai-star-rail"
    "polyglot"
    "cohesion"
  ];

  knownApplications = (builtins.attrNames packageMap) ++ specialApplications;

  unknownApplications =
    lib.filter (
      name: !(lib.elem name knownApplications)
    )
    cfg;

  ordinaryApplications =
    lib.filter (
      name: builtins.hasAttr name packageMap
    )
    cfg;

  selectedPackages =
    map (
      name: packageMap.${name}
    )
    ordinaryApplications;

  flatpakPackages =
    lib.optionals (lib.elem "polyglot" cfg) [
      "io.github.DraqueT.PolyGlot"
    ]
    ++ lib.optionals (lib.elem "cohesion" cfg) [
      "io.github.brunofin.Cohesion"
    ];

  spicePkgs = inputs.spicetify-nix.legacyPackages.${pkgs.stdenv.hostPlatform.system};
in {
  imports = [
    inputs.nix-flatpak.nixosModules.nix-flatpak
  ];

  options.korunix.applications = lib.mkOption {
    type = lib.types.listOf lib.types.str;
    default = [];
    description = "Aplicaciones que la persona quiere tener disponibles.";
  };

  config = lib.mkIf config.korunix.enable {
    assertions = [
      {
        assertion = unknownApplications == [];
        message =
          "Korunix todavía no conoce estas aplicaciones: "
          + lib.concatStringsSep ", " unknownApplications;
      }
    ];

    environment.systemPackages = selectedPackages;

    programs.firefox.enable = lib.elem "firefox" cfg;

    programs.localsend = lib.mkIf (lib.elem "localsend" cfg) {
      enable = true;
      openFirewall = true;
    };

    programs.obs-studio = lib.mkIf (lib.elem "obs-studio" cfg) {
      enable = true;

      package = pkgs.obs-studio.override {
        cudaSupport = false;
      };

      plugins = with pkgs.obs-studio-plugins; [
        wlrobs
        obs-backgroundremoval
        obs-pipewire-audio-capture
        obs-vaapi
        obs-gstreamer
        obs-vkcapture
      ];

      enableVirtualCamera = true;
    };

    boot.extraModulePackages = lib.optionals (lib.elem "obs-studio" cfg) [
      config.boot.kernelPackages.v4l2loopback
    ];

    programs.steam = lib.mkIf (lib.elem "steam" cfg) {
      enable = true;
      remotePlay.openFirewall = true;
      dedicatedServer.openFirewall = true;
      package = pkgs.millennium-steam;
    };

    programs.gamemode.enable = lib.elem "steam" cfg;

    programs.anime-game-launcher.enable = lib.elem "genshin-impact" cfg;
    programs.honkers-railway-launcher.enable = lib.elem "honkai-star-rail" cfg;

    # Spicetify tiene módulo NixOS propio, así que ya no necesita Home Manager.
    # Seleccionar Spotify instala Spotify ya parcheado con las extensiones actuales.
    programs.spicetify = lib.mkIf (lib.elem "spotify" cfg) {
      enable = true;

      # Spotify funciona de forma nativa en Wayland. Además de evitar XWayland,
      # las decoraciones Wayland permiten que Niri gestione correctamente la
      # ventana sin la barra de título blanca del cliente de Spotify.
      wayland = true;
      spotifyLaunchFlags = "--enable-features=WaylandWindowDecorations,UseOzonePlatform";

      enabledExtensions = with spicePkgs.extensions; [
        adblock
        spicyLyrics
        oneko
      ];

      theme = spicePkgs.themes.defaultDynamic;
    };

    # Flatpak es una capacidad del sistema, no una pregunta técnica. Puede quedar
    # habilitado aunque en este momento no haya ninguna aplicación Flatpak elegida.
    services.flatpak.enable = true;
    services.flatpak.packages = flatpakPackages;

    services.flatpak.update.auto = lib.mkIf (flatpakPackages != []) {
      enable = true;
      onCalendar = "weekly";
    };
  };
}

{
  config,
  inputs,
  lib,
  pkgs,
  ...
}: let
  cfg = config.korunix.applications;

  # Una instalación nueva y el bootstrap consumen la misma fuente de decisiones
  # de producto. La arquitectura solo selecciona la suite ofimática apropiada.
  productDefaults = import ../predeterminados.nix;
  hostSystem = pkgs.stdenv.hostPlatform.system;

  defaultApplications =
    productDefaults.applications.common
    ++ (productDefaults.applications.bySystem.${hostSystem} or []);

  # Esta tabla resuelve elecciones humanas a paquetes. Las piezas que forman
  # parte de un rol fijo de Korunix, como Alacritty o Fetch, no aparecen aquí.
  packageMap = {
    "android-tools" = pkgs.android-tools;
    "xwayland-satellite" = pkgs.xwayland-satellite;

    # Aplicaciones GNOME que Korunix conserva como aplicaciones independientes.
    baobab = pkgs.baobab;
    "gnome-calculator" = pkgs.gnome-calculator;
    "gnome-calendar" = pkgs.gnome-calendar;
    "gnome-characters" = pkgs.gnome-characters;
    "gnome-clocks" = pkgs.gnome-clocks;
    "gnome-disk-utility" = pkgs.gnome-disk-utility;
    "gnome-font-viewer" = pkgs.gnome-font-viewer;
    "gnome-maps" = pkgs.gnome-maps;
    "gnome-text-editor" = pkgs.gnome-text-editor;
    "gnome-weather" = pkgs.gnome-weather;
    loupe = pkgs.loupe;
    nautilus = pkgs.nautilus;
    papers = pkgs.papers;
    "simple-scan" = pkgs.simple-scan;
    snapshot = pkgs.snapshot;
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
    libreoffice = pkgs.libreoffice;
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

  spotifySelected = lib.elem "spotify" cfg;

  desktopChoices =
    [config.korunix.desktop.primary]
    ++ config.korunix.desktop.additional;

  noctaliaDesktopAvailable =
    lib.any (
      desktop: lib.elem desktop ["niri" "hyprland"]
    )
    desktopChoices;

  # Comfy llega desde la revisión fijada de spicetify-nix. Korunix sustituye
  # únicamente sus cargadores remotos por los archivos que esa misma revisión
  # ya contiene, de modo que Spotify conserve el tema también sin conexión.
  comfyThemeSource = pkgs.runCommand "korunix-spicetify-comfy" {} ''
    set -eu

    mkdir -p "$out"
    cp -R ${spicePkgs.themes.comfy.src}/. "$out/"
    chmod -R u+w "$out"

    cp "$out/app.css" "$out/user.css"
    cp "$out/theme.script.js" "$out/theme.js"
  '';

  comfyTheme =
    spicePkgs.themes.comfy
    // {
      src = comfyThemeSource;
      extraCommands = "";
      injectThemeJs = false;
      requiredExtensions = [
        {
          src = comfyThemeSource;
          name = "theme.js";
        }
      ];
    };

  # Spotify ya contiene las modificaciones estructurales de spicetify-nix. Esta
  # extensión solo relee colors.css para que una nueva paleta de Noctalia pueda
  # verse en la instancia abierta sin volver a parchear la aplicación.
  noctaliaPaletteExtension = pkgs.writeTextDir "noctalia-palette.js" ''
    (() => {
      "use strict";

      const styleId = "korunix-noctalia-palette";
      let previous = "";

      async function refreshKorunixPalette() {
        try {
          const response = await fetch(
            "colors.css?korunix=" + Date.now(),
            { cache: "no-store" },
          );

          if (!response.ok) return;

          const css = await response.text();
          if (!css || css === previous) return;

          let style = document.getElementById(styleId);
          if (!style) {
            style = document.createElement("style");
            style.id = styleId;
            document.head.appendChild(style);
          }

          style.textContent = css;
          previous = css;
        } catch (_) {
          // La paleta empotrada por spicetify-nix sigue siendo el fallback.
        }
      }

      refreshKorunixPalette();
      window.setInterval(refreshKorunixPalette, 2000);
      document.addEventListener("visibilitychange", () => {
        if (!document.hidden) refreshKorunixPalette();
      });
    })();
  '';

  # El store de Nix es inmutable. Para una sesión Noctalia se prepara una copia
  # de ejecución derivada exactamente del Spotify construido por spicetify-nix.
  # No contiene configuración humana y se renueva solo cuando cambia la
  # derivación de origen.
  prepareSpicetifyRuntime = pkgs.writeShellScript "korunix-spicetify-runtime-prepare" ''
    set -eu

    case ":''${XDG_CURRENT_DESKTOP:-}:''${XDG_SESSION_DESKTOP:-}:''${DESKTOP_SESSION:-}:" in
      *:niri:*|*:Niri:*|*:Hyprland:*|*:hyprland:*|*:hyprland-uwsm:*)
        ;;
      *)
        exit 0
        ;;
    esac

    state_home="''${XDG_STATE_HOME:-$HOME/.local/state}"
    runtime_root="$state_home/korunix/spicetify"
    target="$runtime_root/spotify"
    marker="$runtime_root/source"
    source=${lib.escapeShellArg (toString config.programs.spicetify.spicedSpotify)}

    mkdir -p "$runtime_root"

    if [ -x "$target/spotify" ] \
        && [ -f "$marker" ] \
        && [ "$(cat "$marker")" = "$source" ]
    then
      exit 0
    fi

    temporary=""
    previous="$runtime_root/.spotify.previous"

    hacer_extraible() {
      path="$1"

      [ -e "$path" ] || return 0

      # cp -a conserva el modo de solo lectura del almacén de Nix. Para
      # borrar una copia incompleta solo hace falta devolver escritura y
      # recorrido a sus directorios; los archivos continúan inmutables.
      ${pkgs.findutils}/bin/find "$path" \
        -type d \
        -exec chmod u+rwx {} +
    }
    cleanup() {
      if [ -n "''${temporary:-}" ] && [ -d "$temporary" ]; then
        hacer_extraible "$temporary" || true
        rm -rf -- "$temporary"
      fi
    }

    trap cleanup EXIT HUP INT TERM

    # Una interrupción anterior puede dejar una copia de trabajo incompleta.
    # Siempre se elimina antes de preparar la siguiente; no contiene datos
    # personales y puede reconstruirse desde spicetify-nix.
    for stale in "$runtime_root"/.spotify.new.*; do
      [ -e "$stale" ] || continue
      hacer_extraible "$stale"
      rm -rf -- "$stale"
    done

    temporary="$(${pkgs.coreutils}/bin/mktemp -d "$runtime_root/.spotify.new.XXXXXX")"
    cp -a --reflink=auto "$source/share/spotify/." "$temporary/"

    hacer_extraible "$temporary"
    chmod u+w \
      "$temporary/spotify" \
      "$temporary/Apps/xpui/colors.css"
    # El wrapper construido por Nix contiene la ruta absoluta de su propia
    # derivación. Solo sustituimos el directorio de recursos para que el
    # ejecutable copiado lea el xpui mutable; las bibliotecas siguen fijadas.
    ${pkgs.python3}/bin/python3 - \
      "$temporary/spotify" \
      "$source/share/spotify" \
      "$target" \
      <<'PY'
    import sys
    from pathlib import Path

    wrapper = Path(sys.argv[1])
    source = sys.argv[2]
    target = sys.argv[3]
    content = wrapper.read_text(encoding="utf-8")

    if source not in content:
        raise SystemExit("Korunix: el lanzador de Spotify no contiene la ruta esperada.")

    wrapper.write_text(content.replace(source, target), encoding="utf-8")
    PY

    hacer_extraible "$previous"
    rm -rf -- "$previous"

    if [ -e "$target" ]; then
      mv "$target" "$previous"
    fi

    mv "$temporary" "$target"
    temporary=""

    printf '%s\n' "$source" > "$marker.new"
    mv "$marker.new" "$marker"
    hacer_extraible "$previous"
    rm -rf -- "$previous"
  '';

  # Noctalia entrega un INI con roles Material 3. El adaptador solo convierte
  # esos colores al formato CSS que el Spotify ya construido por spicetify-nix
  # consume; no instala ni vuelve a ejecutar Spicetify.
  syncNoctaliaSpotify = pkgs.writeShellApplication {
    name = "korunix-spotify-theme-sync";
    runtimeInputs = [pkgs.python3];
    text = ''
      set -eu

      state_home="''${XDG_STATE_HOME:-$HOME/.local/state}"
      config_home="''${XDG_CONFIG_HOME:-$HOME/.config}"
      palette="$state_home/korunix/spicetify/noctalia.ini"
      previous_palette="$config_home/spicetify/Themes/Comfy/color.ini"
      colors="$state_home/korunix/spicetify/spotify/Apps/xpui/colors.css"

      # La primera activación puede ocurrir antes de que Noctalia haya procesado
      # la plantilla nueva. Su último color.ini se acepta solo como fuente de
      # lectura durante esa transición y nunca se modifica.
      if [ ! -f "$palette" ] && [ -f "$previous_palette" ]; then
        palette="$previous_palette"
      fi

      if [ ! -f "$palette" ] || [ ! -f "$colors" ]; then
        exit 0
      fi

      python3 - "$palette" "$colors" <<'PY'
      import configparser
      import os
      import re
      import sys
      from pathlib import Path

      palette_path = Path(sys.argv[1])
      colors_path = Path(sys.argv[2])

      color_names = [
          "text",
          "subtext",
          "main",
          "main-elevated",
          "main-transition",
          "highlight",
          "highlight-elevated",
          "sidebar",
          "player",
          "card",
          "shadow",
          "selected-row",
          "button",
          "button-active",
          "button-disabled",
          "tab-active",
          "notification",
          "notification-error",
          "misc",
          "play-button",
          "play-button-active",
          "progress-fg",
          "progress-bg",
          "heart",
          "pagelink-active",
          "radio-btn-active",
      ]

      parser = configparser.ConfigParser(interpolation=None)
      parser.read(palette_path, encoding="utf-8")

      if "Comfy" not in parser:
          raise SystemExit("Korunix: la paleta de Noctalia no contiene la sección Comfy.")

      scheme = parser["Comfy"]
      values = {}

      for name in color_names:
          value = scheme.get(name, "").strip().lstrip("#")
          if not re.fullmatch(r"[0-9A-Fa-f]{6}", value):
              raise SystemExit(f"Korunix: el color {name} no es un valor hexadecimal válido.")
          values[name] = value.lower()

      lines = [":root {"]
      for name in color_names:
          value = values[name]
          rgb = ", ".join(str(int(value[index:index + 2], 16)) for index in (0, 2, 4))
          lines.append(f"  --spice-{name}: #{value} !important;")
          lines.append(f"  --spice-rgb-{name}: {rgb} !important;")
      lines.append("}")
      lines.append("")

      temporary = colors_path.with_name(colors_path.name + ".new")
      temporary.write_text("\n".join(lines), encoding="utf-8")
      os.replace(temporary, colors_path)
      PY
    '';
  };

  # El lanzador general continúa siendo Spotify. Solo Niri y Hyprland usan la
  # copia capaz de recibir la paleta viva; Cinnamon y Plasma ejecutan directamente
  # el paquete inmutable producido por spicetify-nix.
  spotifySessionWrapper = pkgs.writeShellApplication {
    name = "spotify";
    runtimeInputs = [pkgs.systemd];
    text = ''
      set -eu

      case ":''${XDG_CURRENT_DESKTOP:-}:''${XDG_SESSION_DESKTOP:-}:''${DESKTOP_SESSION:-}:" in
        *:niri:*|*:Niri:*|*:Hyprland:*|*:hyprland:*|*:hyprland-uwsm:*)
          if systemctl --user start korunix-spicetify-runtime.service; then
            state_home="''${XDG_STATE_HOME:-$HOME/.local/state}"
            runtime="$state_home/korunix/spicetify/spotify/spotify"

            if [ -x "$runtime" ]; then
              exec "$runtime" "$@"
            fi
          fi
          ;;
      esac

      exec ${config.programs.spicetify.spicedSpotify}/bin/spotify "$@"
    '';
  };
in {
  imports = [
    inputs.nix-flatpak.nixosModules.nix-flatpak
  ];

  options.korunix.applications = lib.mkOption {
    type = lib.types.listOf lib.types.str;
    default = defaultApplications;
    description = ''
      Aplicaciones que la persona quiere tener disponibles. Cuando un host no
      declara una selección propia, parte del conjunto inicial de Korunix.
    '';
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

    environment.systemPackages =
      selectedPackages
      ++ lib.optionals spotifySelected [
        (lib.hiPrio spotifySessionWrapper)
        syncNoctaliaSpotify
      ];

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
    programs.spicetify = lib.mkIf spotifySelected {
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
        {
          src = noctaliaPaletteExtension;
          name = "noctalia-palette.js";
        }
      ];

      theme = comfyTheme;
      colorScheme = "Comfy";
    };

    # El servicio prepara únicamente estado derivado y no modifica la copia
    # declarativa, la configuración externa de ~/.config/spicetify ni Git.
    systemd.user.services.korunix-spicetify-runtime = lib.mkIf (spotifySelected && noctaliaDesktopAvailable) {
      description = "Prepara Spotify Comfy para la paleta de Noctalia";

      serviceConfig = {
        Type = "oneshot";
        ExecStart = prepareSpicetifyRuntime;
        ExecStartPost = lib.getExe syncNoctaliaSpotify;
      };
    };

    systemd.user.services.korunix-spicetify-palette-sync = lib.mkIf (spotifySelected && noctaliaDesktopAvailable) {
      description = "Sincroniza Spotify Comfy con la paleta de Noctalia";

      serviceConfig = {
        Type = "oneshot";
        ExecStart = lib.getExe syncNoctaliaSpotify;
      };
    };

    systemd.user.paths.korunix-spicetify-palette-sync = lib.mkIf (spotifySelected && noctaliaDesktopAvailable) {
      description = "Observa la paleta de Spotify generada por Noctalia";
      wantedBy = ["graphical-session.target"];

      pathConfig = {
        PathChanged = [
          "%h/.local/state/korunix/spicetify/noctalia.ini"
          "%h/.config/spicetify/Themes/Comfy/color.ini"
        ];
        Unit = "korunix-spicetify-palette-sync.service";
      };
    };

    # La plantilla se inserta en la configuración personal solo cuando Spotify
    # está seleccionado. Así Noctalia no muestra ni ejecuta una integración para
    # una aplicación ausente.
    environment.etc."korunix/noctalia/spicetify-template.toml" = lib.mkIf (spotifySelected && noctaliaDesktopAvailable) {
      text = ''
        # Esta plantilla pertenece a Korunix. Noctalia genera la paleta y el
        # adaptador actualiza la copia de ejecución creada desde spicetify-nix.
        [theme.templates.user.spicetify]
        input_path = "/etc/korunix/noctalia/themes/spicetify/color.ini"
        output_path = "~/.local/state/korunix/spicetify/noctalia.ini"
        post_hook = "/run/current-system/sw/bin/korunix-spotify-theme-sync"
      '';
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

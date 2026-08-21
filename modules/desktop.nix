{
  config,
  inputs,
  lib,
  pkgs,
  ...
}: let
  cfg = config.korunix.desktop;
  localization = config.korunix.localization;

  keyboardLayouts =
    [localization.keyboard.layout]
    ++ localization.keyboard.additionalLayouts;

  keyboardVariants =
    [localization.keyboard.variant]
    ++ localization.keyboard.additionalVariants;

  keyboardLabelsFile =
    pkgs.writeText
      "korunix-keyboard-labels.json"
      (builtins.toJSON localization.keyboard.displayNames);

  prepareKeyboardLabels = pkgs.writeShellScript "korunix-noctalia-keyboard-labels" ''
    set -e

    dir="$HOME/.config/noctalia"
    target="$dir/20-korunix-keyboard-labels.toml"
    mkdir -p "$dir"

    # Residuo de una implementación antigua de Korunix.
    rm -f "$dir/10-korunix-integrations.toml"

    labels="$(cat ${keyboardLabelsFile})"

    if [ "$labels" = "[]" ]; then
      rm -f "$target"
      exit 0
    fi

    if [ -z "$NIRI_SOCKET" ]; then
      socket="$(ls -t "$XDG_RUNTIME_DIR"/niri*.sock 2>/dev/null | head -n1 || true)"

      if [ -n "$socket" ]; then
        export NIRI_SOCKET="$socket"
      else
        # Hyprland tiene su propio backend de teclado en Noctalia. No borramos
        # aquí las etiquetas ya generadas por una sesión Niri.
        exit 0
      fi
    fi

    data=""

    for _ in $(seq 1 80); do
      data="$(${lib.getExe pkgs.niri} msg --json keyboard-layouts 2>/dev/null || true)"

      if printf "%s" "$data" | ${lib.getExe pkgs.jq} -e '.names' >/dev/null 2>&1; then
        break
      fi

      sleep 0.1
    done

    names_count="$(printf "%s" "$data" | ${lib.getExe pkgs.jq} '.names | length' 2>/dev/null || echo 0)"
    labels_count="$(printf "%s" "$labels" | ${lib.getExe pkgs.jq} 'length')"

    if [ "$names_count" != "$labels_count" ]; then
      echo "Korunix: Niri y el modelo de teclado no coinciden." >&2
      rm -f "$target"
      exit 0
    fi

    {
      echo "# Archivo generado por Korunix."
      echo "# Edita korunix.localization, no este archivo."
      echo
      echo "[shell.keyboard_layout.custom_labels]"

      printf "%s" "$data" |
        ${lib.getExe pkgs.jq} -r           --argjson labels "$labels"           '.names as $names
           | range(0; ($names | length)) as $i
           | "\($names[$i] | tojson) = \($labels[$i] | tojson)"'
    } > "$target.tmp"

    mv "$target.tmp" "$target"
  '';

  # El modelo reconoce los cinco escritorios objetivo. Este bloque implementa
  # Niri, Hyprland y GNOME; Cinnamon y Plasma conservan la protección que evita
  # activar una sesión incompleta antes de tener su implementación dedicada.
  desktopType = lib.types.enum [
    "niri"
    "hyprland"
    "gnome"
    "cinnamon"
    "plasma"
  ];

  implementedDesktops = [
    "niri"
    "hyprland"
    "gnome"
  ];

  enabledDesktops = lib.unique ([cfg.primary] ++ cfg.additional);
  unimplementedDesktops =
    lib.filter (
      desktop: !(lib.elem desktop implementedDesktops)
    )
    enabledDesktops;

  niriEnabled = lib.elem "niri" enabledDesktops;
  hyprlandEnabled = lib.elem "hyprland" enabledDesktops;
  gnomeEnabled = lib.elem "gnome" enabledDesktops;
  noctaliaEnabled = niriEnabled || hyprlandEnabled;

  # GNOME habilita IBus como método de entrada del sistema. Su autostart
  # genérico excluye GNOME y KDE, pero de otro modo también se ejecutaría en
  # Niri y Hyprland. Esas sesiones ya usan directamente el modelo XKB de
  # Korunix, así que no necesitan arrancar ese daemon adicional.
  ibusEnabled =
    config.i18n.inputMethod.enable
    && config.i18n.inputMethod.type == "ibus";

  ibusPackage =
    pkgs.ibus-with-plugins.override {
      plugins = config.i18n.inputMethod.ibus.engines;
    };

  ibusPanel = config.i18n.inputMethod.ibus.panel;

  ibusPanelArgument =
    lib.optionalString
      (ibusPanel != null)
      "--panel=${toString ibusPanel}";

  # Hyprland 0.55+ usa Lua. El archivo humano permanece en config/ y estos
  # marcadores reciben la misma fuente de verdad de teclado que Niri.
  hyprlandConfig =
    pkgs.writeText "korunix-hyprland.lua" (
      builtins.replaceStrings
        [
          "@KORUNIX_KEYBOARD_LAYOUTS@"
          "@KORUNIX_KEYBOARD_VARIANTS@"
          "@KORUNIX_KEYBOARD_OPTIONS@"
        ]
        [
          (lib.concatStringsSep "," keyboardLayouts)
          (lib.concatStringsSep "," keyboardVariants)
          localization.keyboard.switchOption
        ]
        (builtins.readFile ../config/hyprland.lua)
    );
in {
  options.korunix.desktop = {
    primary = lib.mkOption {
      type = desktopType;
      default = "niri";
      description = "Escritorio que Korunix presenta como sesión principal.";
    };

    additional = lib.mkOption {
      type = lib.types.listOf desktopType;
      default = [];
      description = "Otros escritorios disponibles para elegir al iniciar sesión.";
    };
  };

  config = lib.mkIf config.korunix.enable {
    assertions = [
      {
        assertion = !(lib.elem cfg.primary cfg.additional);
        message = "El escritorio principal no puede repetirse como escritorio adicional.";
      }
      {
        assertion = unimplementedDesktops == [];
        message =
          "Estos escritorios todavía no tienen una implementación completa en este bloque: "
          + lib.concatStringsSep ", " unimplementedDesktops;
      }
    ];

    # GDM es el punto de inicio de sesión común. Cuando haya varios escritorios,
    # la persona podrá escoger cualquiera de las sesiones disponibles.
    services.xserver.enable = true;

    services.displayManager = {
      gdm.enable = true;
      defaultSession = cfg.primary;
    };

    programs.niri.enable = niriEnabled;

    # El módulo NixOS instala Hyprland, XWayland, su portal y la sesión que GDM
    # presenta. UWSM mantiene correctamente los targets systemd de la sesión.
    programs.hyprland = lib.mkIf hyprlandEnabled {
      enable = true;
      withUWSM = true;
      xwayland.enable = true;
    };

    services.desktopManager.gnome.enable = gnomeEnabled;

    # La copia en /etc/xdg tiene prioridad sobre el autostart aportado por el
    # paquete de IBus. Conservamos su comportamiento original y añadimos solo
    # las sesiones que Korunix administra directamente mediante XKB.
    environment.etc."xdg/autostart/ibus-daemon.desktop" =
      lib.mkIf (ibusEnabled && (niriEnabled || hyprlandEnabled)) {
        text = ''
          [Desktop Entry]
          Name=IBus
          Type=Application
          Exec=${ibusPackage}/bin/ibus-daemon --daemonize --xim ${ibusPanelArgument}
          # GNOME inicia IBus mediante systemd.
          # KDE lo integra desde su propio escritorio.
          # Niri y Hyprland usan el teclado XKB administrado por Korunix.
          NotShowIn=GNOME;KDE;niri;Hyprland;hyprland;
        '';
      };

    # Niri y Hyprland utilizan la misma base GTK acordada. Xwayland Satellite
    # solo pertenece a Niri; Hyprland utiliza su integración XWayland propia.
    environment.systemPackages =
      lib.optionals (niriEnabled || hyprlandEnabled) [
        pkgs.nautilus
        pkgs.eog
        pkgs.papers
      ]
      ++ lib.optionals niriEnabled [
        pkgs.xwayland-satellite
      ];

    # «Abrir en terminal» usa Alacritty en Nautilus, sin depender del escritorio
    # desde el que se haya abierto el gestor de archivos.
    programs.nautilus-open-any-terminal =
      lib.mkIf (niriEnabled || hyprlandEnabled || gnomeEnabled) {
      enable = true;
      terminal = "alacritty";
    };

    # El teclado concreto del equipo se genera desde korunix.localization.
    environment.etc."korunix/niri-input.kdl" = lib.mkIf niriEnabled {
      text = ''
        // Archivo generado por Korunix.
        // Edita korunix.localization, no este archivo.

        input {
          keyboard {
            xkb {
              layout ${builtins.toJSON (lib.concatStringsSep "," keyboardLayouts)}
              variant ${builtins.toJSON (lib.concatStringsSep "," keyboardVariants)}
              options ${builtins.toJSON localization.keyboard.switchOption}
            }

            numlock
          }
        }
      '';
    };

    # La experiencia común de Niri permanece como archivo humano del repositorio.
    environment.etc."niri/config.kdl" = lib.mkIf niriEnabled {
      source = ../config/niri.kdl;
    };

    # Cada compositor recibe explícitamente su configuración Korunix. De esta
    # manera no gana una configuración autogenerada o histórica del home.
    environment.sessionVariables = lib.mkMerge [
      (lib.mkIf niriEnabled {
        NIRI_CONFIG = "/etc/niri/config.kdl";
      })

      (lib.mkIf hyprlandEnabled {
        HYPRLAND_CONFIG = "/etc/hypr/hyprland.lua";
      })
    ];

    environment.etc."hypr/hyprland.lua" = lib.mkIf hyprlandEnabled {
      source = hyprlandConfig;
    };

    # Noctalia utiliza su módulo NixOS oficial. Su servicio de usuario arranca
    # después de que Korunix haya preparado la configuración de esa persona.
    programs.noctalia = lib.mkIf noctaliaEnabled {
      enable = true;
      package = inputs.noctalia.packages.${pkgs.stdenv.hostPlatform.system}.default;
      systemd.enable = true;
      recommendedServices.enable = false;
    };

    systemd.user.services.korunix-noctalia-keyboard-labels =
      lib.mkIf noctaliaEnabled {
        description = "Prepara los nombres humanos del teclado para Noctalia";

        after = ["korunix-user-prepare.service"];
        requires = ["korunix-user-prepare.service"];
        before = ["noctalia.service"];

        serviceConfig = {
          Type = "oneshot";
          ExecStart = prepareKeyboardLabels;
        };
      };

    systemd.user.services.noctalia = lib.mkIf noctaliaEnabled {
      after = [
        "korunix-user-prepare.service"
        "korunix-noctalia-keyboard-labels.service"
      ];

      requires = [
        "korunix-user-prepare.service"
        "korunix-noctalia-keyboard-labels.service"
      ];
    };

    # El TOML contiene valores comunes. El servicio korunix-user-prepare sustituye
    # únicamente datos que dependen de la persona, como avatar y ruta XDG de fotos.
    environment.etc."korunix/noctalia/config.toml" = lib.mkIf noctaliaEnabled {
      source = ../config/noctalia/config.toml;
    };

    environment.etc."korunix/noctalia/wallpapers" = lib.mkIf noctaliaEnabled {
      source = ../config/noctalia/wallpapers;
    };

    environment.etc."korunix/noctalia/themes" = lib.mkIf noctaliaEnabled {
      source = ../config/noctalia/themes;
    };
  };
}

{
  apariencia,
  aparienciaNoctalia,
  config,
  escritorios,
  hatterSource,
  lib,
  noctaliaPackage,
  pkgs,
  ...
}: let
  niriActivo = builtins.elem "niri" escritorios;
  hyprlandActivo = builtins.elem "hyprland" escritorios;
  cinnamonActivo = builtins.elem "cinnamon" escritorios;
  plasmaActivo = builtins.elem "plasma" escritorios;
  noctaliaActivo = niriActivo || hyprlandActivo;

  # Cinnamon instala los esquemas que necesita, pero también publica defaults
  # visuales de Linux Mint. Cinnamon conserva esos defaults en su propia sesión.
  cinnamonGSettingsOverrides = pkgs.cinnamon-gsettings-overrides.override {
    extraGSettingsOverridePackages =
      config.services.xserver.desktopManager.cinnamon.extraGSettingsOverridePackages;
    extraGSettingsOverrides =
      config.services.xserver.desktopManager.cinnamon.extraGSettingsOverrides;
  };

  cinnamonGSettingsSchemaDir = "${cinnamonGSettingsOverrides}/share/gsettings-schemas/nixos-gsettings-overrides/glib-2.0/schemas";

  # Fuera de Cinnamon se conservan los mismos esquemas, pero no Mint-Y.
  neutralGSettingsOverrides = pkgs.runCommand "korunix-gsettings-neutral" {} ''
    set -eu

    destino="$out/share/gsettings-schemas/korunix-gsettings-overrides/glib-2.0/schemas"
    mkdir -p "$destino"
    cp -a ${cinnamonGSettingsSchemaDir}/. "$destino/"
    chmod -R u+w "$destino"

    rm -f \
      "$destino/gschemas.compiled" \
      "$destino/mint-artwork.gschema.override"

    ${pkgs.glib.dev}/bin/glib-compile-schemas --strict "$destino"
  '';

  neutralGSettingsSchemaDir = "${neutralGSettingsOverrides}/share/gsettings-schemas/korunix-gsettings-overrides/glib-2.0/schemas";

  hatterIconos = pkgs.runCommand "korunix-hatter-iconos" {} ''
    set -eu
    mkdir -p "$out/share/icons"
    cp -R ${hatterSource}/Hatter "$out/share/icons/Hatter"
    cp -R ${hatterSource}/Hatter-Green "$out/share/icons/Hatter-Green"
    cp -R ${hatterSource}/Hatter-Slate "$out/share/icons/Hatter-Slate"
  '';

  perfilNoctalia = pkgs.writeText "korunix-noctalia-dconf-profile" ''
    user-db:noctalia
  '';

  perfilPlasma = pkgs.writeText "korunix-plasma-dconf-profile" ''
    user-db:plasma
  '';

  esSesionNoctalia = pkgs.writeShellScript "korunix-es-sesion-noctalia-visual" ''
    case "''${XDG_CURRENT_DESKTOP:-}" in
      niri|Niri|Hyprland|hyprland) exit 0 ;;
    esac

    case "''${XDG_SESSION_DESKTOP:-}" in
      niri|Niri|Hyprland|hyprland|hyprland-uwsm) exit 0 ;;
    esac

    case "''${DESKTOP_SESSION:-}" in
      niri|hyprland|hyprland-uwsm) exit 0 ;;
    esac

    exit 1
  '';

  sincronizarApariencia = pkgs.writeShellScript "korunix-noctalia-portal-settings" ''
    set -eu

    modo="''${1:---default}"
    seleccion=""
    modo_visual=""

    case "$modo" in
      --default)
        modo_visual=${lib.escapeShellArg aparienciaNoctalia.mode}
        ;;
      --resolved)
        intento=0

        while [ "$intento" -lt 20 ]; do
          seleccion="$(${lib.getExe noctaliaPackage} msg color-scheme-get 2>/dev/null || true)"
          modo_visual="$(${lib.getExe noctaliaPackage} msg theme-mode-get 2>/dev/null || true)"

          if [ -n "$seleccion" ] && [ -n "$modo_visual" ]; then
            break
          fi

          intento=$((intento + 1))
          ${pkgs.coreutils}/bin/sleep 0.05
        done
        ;;
      *)
        printf '%s\n' "Korunix: modo de sincronización visual no válido: $modo" >&2
        exit 1
        ;;
    esac

    seleccion="$(
      printf '%s\n' "$seleccion" |
        ${pkgs.coreutils}/bin/head -n 1 |
        ${pkgs.coreutils}/bin/tr '[:upper:]' '[:lower:]'
    )"

    modo_visual="$(
      printf '%s\n' "$modo_visual" |
        ${pkgs.coreutils}/bin/head -n 1 |
        ${pkgs.coreutils}/bin/tr '[:upper:]' '[:lower:]'
    )"

    case "$seleccion" in
      "community everforest") tema_iconos="Hatter-Green" ;;
      *) tema_iconos="Hatter-Slate" ;;
    esac

    DCONF_PROFILE=noctalia \
      ${lib.getExe' pkgs.glib "gsettings"} set \
      org.gnome.desktop.interface icon-theme "$tema_iconos"

    case "$modo_visual" in
      light) esquema="prefer-light" ;;
      dark) esquema="prefer-dark" ;;
      *) esquema="" ;;
    esac

    if [ -n "$esquema" ]; then
      DCONF_PROFILE=noctalia \
        ${lib.getExe' pkgs.glib "gsettings"} set \
        org.gnome.desktop.interface color-scheme "$esquema"
    fi
  '';

  envolverPortal = nombre: ejecutable:
    pkgs.writeShellScript nombre ''
      if ${config.systemd.package}/bin/systemctl --user --quiet is-active noctalia.service 2>/dev/null; then
        export DCONF_PROFILE=noctalia
        ${lib.optionalString cinnamonActivo ''
        export NIX_GSETTINGS_OVERRIDES_DIR=${lib.escapeShellArg neutralGSettingsSchemaDir}
      ''}
      else
        case ":''${XDG_CURRENT_DESKTOP:-}:''${XDG_SESSION_DESKTOP:-}:''${DESKTOP_SESSION:-}:" in
          *:niri:*|*:Niri:*|*:Hyprland:*|*:hyprland:*|*:hyprland-uwsm:*)
            export DCONF_PROFILE=noctalia
            ${lib.optionalString cinnamonActivo ''
        export NIX_GSETTINGS_OVERRIDES_DIR=${lib.escapeShellArg neutralGSettingsSchemaDir}
      ''}
            ;;
          *:X-Cinnamon:*|*:cinnamon:*|*:cinnamon-wayland:*)
            export DCONF_PROFILE=user
            ${lib.optionalString cinnamonActivo ''
        export NIX_GSETTINGS_OVERRIDES_DIR=${lib.escapeShellArg cinnamonGSettingsSchemaDir}
      ''}
            ;;
          *:KDE:*|*:plasma:*)
            export DCONF_PROFILE=plasma
            ${lib.optionalString cinnamonActivo ''
        export NIX_GSETTINGS_OVERRIDES_DIR=${lib.escapeShellArg neutralGSettingsSchemaDir}
      ''}
            ;;
          *)
            export DCONF_PROFILE=user
            ${lib.optionalString cinnamonActivo ''
        export NIX_GSETTINGS_OVERRIDES_DIR=${lib.escapeShellArg neutralGSettingsSchemaDir}
      ''}
            ;;
        esac
      fi

      exec ${ejecutable}
    '';

  portalGtk =
    envolverPortal
    "korunix-xdg-desktop-portal-gtk"
    "${pkgs.xdg-desktop-portal-gtk}/libexec/xdg-desktop-portal-gtk";

  portalGnome =
    envolverPortal
    "korunix-xdg-desktop-portal-gnome"
    "${pkgs.xdg-desktop-portal-gnome}/libexec/xdg-desktop-portal-gnome";

  # El entorno global queda neutral. La sesión Cinnamon repone únicamente allí
  # sus defaults Mint y al salir devuelve el entorno neutral.
  sesionCinnamon = pkgs.writeShellScript "korunix-cinnamon-session" ''
    set -u

    export DCONF_PROFILE=user
    export NIX_GSETTINGS_OVERRIDES_DIR=${lib.escapeShellArg cinnamonGSettingsSchemaDir}

    ${config.systemd.package}/bin/systemctl --user set-environment \
      DCONF_PROFILE=user \
      NIX_GSETTINGS_OVERRIDES_DIR=${lib.escapeShellArg cinnamonGSettingsSchemaDir} ||
      true

    ${pkgs.dbus}/bin/dbus-update-activation-environment \
      --systemd \
      DCONF_PROFILE \
      NIX_GSETTINGS_OVERRIDES_DIR ||
      true

    restaurar() {
      export DCONF_PROFILE=user
      export NIX_GSETTINGS_OVERRIDES_DIR=${lib.escapeShellArg neutralGSettingsSchemaDir}

      ${config.systemd.package}/bin/systemctl --user set-environment \
        DCONF_PROFILE=user \
        NIX_GSETTINGS_OVERRIDES_DIR=${lib.escapeShellArg neutralGSettingsSchemaDir} ||
        true

      ${pkgs.dbus}/bin/dbus-update-activation-environment \
        --systemd \
        DCONF_PROFILE \
        NIX_GSETTINGS_OVERRIDES_DIR ||
        true
    }

    trap restaurar EXIT
    estado=0
    ${pkgs.cinnamon-session}/bin/cinnamon-session-cinnamon --wayland "$@" || estado=$?
    restaurar
    trap - EXIT
    exit "$estado"
  '';

  nombresSesion = {
    niri = "niri";
    hyprland = "hyprland-uwsm";
    plasma = "plasma";
    cinnamon = "cinnamon-wayland";
  };

  sesiones =
    pkgs.runCommand "korunix-sesiones-wayland" {
      passthru.providedSessions = map (nombre: nombresSesion.${nombre}) escritorios;
    } ''
      set -eu
      mkdir -p "$out/share/wayland-sessions" "$out/share/xsessions"

      ${lib.optionalString niriActivo ''
        cp \
          ${config.programs.niri.package}/share/wayland-sessions/niri.desktop \
          "$out/share/wayland-sessions/niri.desktop"
      ''}

      ${lib.optionalString hyprlandActivo ''
              cat > "$out/share/wayland-sessions/hyprland-uwsm.desktop" <<EOF
        [Desktop Entry]
        Name=Hyprland (uwsm-managed)
        Comment=Hyprland Wayland administrado por UWSM
        Exec=${lib.getExe config.programs.uwsm.package} start -- ${config.programs.hyprland.package}/share/wayland-sessions/hyprland.desktop
        Type=Application
        DesktopNames=Hyprland
        EOF
      ''}

      ${lib.optionalString plasmaActivo ''
        cp \
          ${pkgs.kdePackages.plasma-workspace.sessions}/share/wayland-sessions/plasma.desktop \
          "$out/share/wayland-sessions/plasma.desktop"
      ''}

      ${lib.optionalString cinnamonActivo ''
        cp \
          ${pkgs.cinnamon}/share/wayland-sessions/cinnamon-wayland.desktop \
          "$out/share/wayland-sessions/cinnamon-wayland.desktop"

        ${pkgs.gnused}/bin/sed -i \
          "s|^Exec=.*|Exec=${sesionCinnamon}|" \
          "$out/share/wayland-sessions/cinnamon-wayland.desktop"

        if ${pkgs.gnugrep}/bin/grep -q '^DesktopNames=' \
          "$out/share/wayland-sessions/cinnamon-wayland.desktop"
        then
          ${pkgs.gnused}/bin/sed -i \
            's/^DesktopNames=.*/DesktopNames=X-Cinnamon/' \
            "$out/share/wayland-sessions/cinnamon-wayland.desktop"
        else
          ${pkgs.gnused}/bin/sed -i \
            '/^\[Desktop Entry\]$/a DesktopNames=X-Cinnamon' \
            "$out/share/wayland-sessions/cinnamon-wayland.desktop"
        fi
      ''}
    '';

  noctaliaBase =
    builtins.replaceStrings
    [
      "@KORUNIX_THEME_SOURCE@"
      "@KORUNIX_THEME_MODE@"
    ]
    [
      aparienciaNoctalia.source
      aparienciaNoctalia.mode
    ]
    (builtins.readFile ./config/noctalia/config.toml);
in {
  environment.sessionVariables = lib.mkIf cinnamonActivo {
    NIX_GSETTINGS_OVERRIDES_DIR = lib.mkForce neutralGSettingsSchemaDir;
  };

  services.displayManager.sessionPackages = lib.mkForce [sesiones];

  environment.systemPackages =
    [hatterIconos (lib.hiPrio sesiones)]
    ++ lib.optionals noctaliaActivo [pkgs.adw-gtk3];

  xdg.portal.extraPortals = lib.mkIf noctaliaActivo [
    pkgs.xdg-desktop-portal-gtk
  ];

  programs.dconf.profiles.noctalia =
    lib.mkIf noctaliaActivo perfilNoctalia;

  programs.dconf.profiles.plasma =
    lib.mkIf plasmaActivo perfilPlasma;

  systemd.user.services.korunix-noctalia-icon-theme-default = lib.mkIf noctaliaActivo {
    description = "Prepara Hatter Slate para la sesión Noctalia";
    wantedBy = ["graphical-session-pre.target"];
    before = [
      "graphical-session.target"
      "noctalia.service"
      "xdg-desktop-portal-gtk.service"
      "xdg-desktop-portal-gnome.service"
    ];

    serviceConfig = {
      Type = "oneshot";
      ExecCondition = esSesionNoctalia;
      ExecStart = "${sincronizarApariencia} --default";
    };
  };

  systemd.user.services.korunix-noctalia-icon-theme-sync = lib.mkIf noctaliaActivo {
    description = "Sincroniza GTK con la apariencia efectiva de Noctalia";
    wantedBy = ["graphical-session.target"];
    after = ["noctalia.service"];
    wants = ["noctalia.service"];

    serviceConfig = {
      Type = "oneshot";
      ExecCondition = esSesionNoctalia;
      ExecStart = "${sincronizarApariencia} --resolved";
    };
  };

  systemd.user.paths.korunix-noctalia-icon-theme-sync = lib.mkIf noctaliaActivo {
    description = "Observa la apariencia activa de Noctalia";
    wantedBy = ["graphical-session.target"];

    pathConfig = {
      PathChanged = [
        "%h/.config/noctalia/config.toml"
        "%h/.local/state/noctalia/settings.toml"
      ];
      Unit = "korunix-noctalia-icon-theme-sync.service";
    };
  };

  systemd.user.services.xdg-desktop-portal-gtk = lib.mkIf noctaliaActivo {
    after = ["korunix-noctalia-icon-theme-sync.service"];
    wants = ["korunix-noctalia-icon-theme-sync.service"];

    serviceConfig.ExecStart = [
      ""
      "${portalGtk}"
    ];
  };

  systemd.user.services.xdg-desktop-portal-gnome = lib.mkIf niriActivo {
    after = ["korunix-noctalia-icon-theme-sync.service"];
    wants = ["korunix-noctalia-icon-theme-sync.service"];

    serviceConfig.ExecStart = [
      ""
      "${portalGnome}"
    ];
  };

  systemd.user.services.noctalia = lib.mkIf noctaliaActivo {
    after = lib.mkAfter ["korunix-noctalia-icon-theme-default.service"];
    wants = lib.mkAfter ["korunix-noctalia-icon-theme-default.service"];

    environment =
      {
        DCONF_PROFILE = "noctalia";
      }
      // lib.optionalAttrs cinnamonActivo {
        NIX_GSETTINGS_OVERRIDES_DIR = neutralGSettingsSchemaDir;
      };
  };

  environment.etc."korunix/noctalia.toml" = lib.mkIf noctaliaActivo {
    text = noctaliaBase;
  };

  environment.etc."korunix/noctalia/gtk4-live.css" = lib.mkIf noctaliaActivo {
    source = ./config/noctalia/gtk4-live.css;
  };

  environment.etc."korunix/noctalia/gtk4-live.toml" = lib.mkIf noctaliaActivo {
    source = ./config/noctalia/gtk4-live.toml;
  };

  environment.etc."korunix/noctalia/gtk4-live-apply.sh" = lib.mkIf noctaliaActivo {
    source = ./config/noctalia/gtk4-live-apply.sh;
    mode = "0755";
  };

  environment.etc."korunix/noctalia/wallpapers" = lib.mkIf noctaliaActivo {
    source = ./config/noctalia/wallpapers;
  };

  environment.etc."korunix/noctalia/themes" = lib.mkIf noctaliaActivo {
    source = ./config/noctalia/themes;
  };
}

{
  config,
  inputs,
  lib,
  pkgs,
  ...
}: let
  productDefaults = import ./predeterminados.nix;
  roleModel = import ./roles.nix {inherit lib pkgs;};
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

  # Korunix mantiene cuatro escritorios soportados. Cada uno conserva su
  # integración nativa; Noctalia pertenece únicamente a Niri y Hyprland.
  desktopType = lib.types.enum [
    "niri"
    "hyprland"
    "cinnamon"
    "plasma"
  ];

  implementedDesktops = [
    "niri"
    "hyprland"
    "cinnamon"
    "plasma"
  ];

  enabledDesktops = lib.unique ([cfg.primary] ++ cfg.additional);
  unimplementedDesktops =
    lib.filter (
      desktop: !(lib.elem desktop implementedDesktops)
    )
    enabledDesktops;

  niriEnabled = lib.elem "niri" enabledDesktops;
  hyprlandEnabled = lib.elem "hyprland" enabledDesktops;
  cinnamonEnabled = lib.elem "cinnamon" enabledDesktops;
  plasmaEnabled = lib.elem "plasma" enabledDesktops;

  fixedRolePackages = roleModel.packagesFor enabledDesktops;

  roleBrowserLauncher = pkgs.writeShellApplication {
    name = "korunix-open-browser";
    runtimeInputs = [
      pkgs.gtk3
      pkgs.xdg-utils
    ];

    text = ''
      set -eu

      desktop_id="$(xdg-settings get default-web-browser 2>/dev/null || true)"

      if [ -z "$desktop_id" ]; then
        printf '%s\n' \
          "Korunix todavía no tiene un navegador predeterminado efectivo." \
          >&2
        exit 2
      fi

      application="''${desktop_id%.desktop}"

      exec gtk-launch "$application"
    '';
  };

  # El módulo de Cinnamon reúne en un único directorio tanto los esquemas
  # necesarios como los valores predeterminados visuales de Linux Mint.
  # Conservamos ese conjunto intacto para la sesión Cinnamon.
  cinnamonGSettingsOverrides = pkgs.cinnamon-gsettings-overrides.override {
    extraGSettingsOverridePackages =
      config.services.xserver.desktopManager.cinnamon.extraGSettingsOverridePackages;
    extraGSettingsOverrides =
      config.services.xserver.desktopManager.cinnamon.extraGSettingsOverrides;
  };

  cinnamonGSettingsSchemaDir = "${cinnamonGSettingsOverrides}/share/gsettings-schemas/nixos-gsettings-overrides/glib-2.0/schemas";

  # Cinnamon exporta sus overrides de GSettings para todas las sesiones.
  # Korunix conserva exactamente los mismos esquemas, pero recompila una copia
  # sin mint-artwork.gschema.override para que los defaults visuales de Linux
  # Mint no alcancen Niri, Hyprland ni Plasma.
  neutralGSettingsOverrides = pkgs.runCommand "korunix-gsettings-overrides-neutral" {} ''
    set -eu

    schema_dir="$out/share/gsettings-schemas/korunix-gsettings-overrides/glib-2.0/schemas"

    mkdir -p "$schema_dir"
    cp -a ${cinnamonGSettingsSchemaDir}/. "$schema_dir/"
    chmod -R u+w "$schema_dir"

    rm -f \
      "$schema_dir/gschemas.compiled" \
      "$schema_dir/mint-artwork.gschema.override"

    ${pkgs.glib.dev}/bin/glib-compile-schemas --strict "$schema_dir"
  '';

  neutralGSettingsSchemaDir = "${neutralGSettingsOverrides}/share/gsettings-schemas/korunix-gsettings-overrides/glib-2.0/schemas";

  monitorConfigured =
    cfg.monitor.output
    != null
    && cfg.monitor.mode != null;

  monitorMode =
    lib.optionalString monitorConfigured
    "${cfg.monitor.mode}@${toString cfg.monitor.refreshRate}.000";

  noctaliaEnabled = niriEnabled || hyprlandEnabled;

  noctaliaPackage =
    inputs.noctalia.packages.${pkgs.stdenv.hostPlatform.system}.default;

  # La integración de dispositivos sigue el escritorio activo: Valent pertenece
  # a los escritorios GTK/Noctalia y KDE Connect a Plasma.
  valentDesktopEnabled = niriEnabled || hyprlandEnabled || cinnamonEnabled;
  kdeConnectDesktopEnabled = plasmaEnabled;
  deviceConnectEnabled = valentDesktopEnabled || kdeConnectDesktopEnabled;

  # Korunix publica una sola sesión por escritorio y todas son Wayland.
  desktopSessionNames = {
    niri = "niri";
    hyprland = "hyprland-uwsm";
    plasma = "plasma";
    cinnamon = "cinnamon-wayland";
  };

  waylandSessionNames =
    map (
      desktop: desktopSessionNames.${desktop}
    )
    enabledDesktops;

  primarySession = desktopSessionNames.${cfg.primary};

  # Niri y Hyprland comparten la experiencia Noctalia. Estas aplicaciones
  # forman parte de esa experiencia y no del catálogo general.
  noctaliaOnlyApplications = with pkgs; [
    baobab
    gnome-characters
    gnome-clocks
    gnome-font-viewer
    gnome-maps
    gnome-text-editor
    gnome-weather
    loupe
    nautilus
    papers
    simple-scan
    snapshot
  ];

  # Cinnamon utiliza también estas tres aplicaciones GNOME de forma nativa.
  noctaliaCinnamonApplications = with pkgs; [
    gnome-calculator
    gnome-calendar
    gnome-disk-utility
  ];

  noctaliaApplications =
    noctaliaOnlyApplications
    ++ noctaliaCinnamonApplications;

  # Aplicaciones y utilidades visibles como parte de Plasma. Kate no se
  # incluye porque está elegida explícitamente como aplicación general.
  plasmaMenuApplications =
    (with pkgs.kdePackages; [
      ark
      dolphin
      elisa
      gwenview
      khelpcenter
      kinfocenter
      kmenuedit
      krdp
      kwalletmanager
      okular
      plasma-systemmonitor
      spectacle
      systemsettings
    ])
    ++ lib.optional
    config.networking.networkmanager.enable
    pkgs.kdePackages.qrca
    ++ lib.optional
    (config.services.flatpak.enable || config.services.fwupd.enable)
    pkgs.kdePackages.discover
    ++ lib.optional
    config.services.printing.enable
    pkgs.kdePackages.print-manager
    ++ lib.optional
    config.hardware.sane.enable
    pkgs.kdePackages.skanpage
    ++ lib.optional
    config.services.colord.enable
    pkgs.kdePackages.colord-kde
    ++ lib.optional
    config.services.hardware.bolt.enable
    pkgs.kdePackages.plasma-thunderbolt
    ++ lib.optional
    config.services.flatpak.enable
    pkgs.kdePackages.flatpak-kcm;

  # Suite nativa de Cinnamon. Las tres aplicaciones compartidas con Noctalia
  # se administran en noctaliaCinnamonApplications para no duplicar reglas.
  cinnamonMenuApplications = with pkgs; [
    bulky
    warpinator
    xviewer
    xreader
    xed-editor
    pix
    celluloid
    gnome-screenshot
    file-roller
    gucharmap
    nemo-with-extensions
    cinnamon-control-center
    cinnamon-screensaver
    gnome-online-accounts-gtk
    onboard
  ];

  # Algunos módulos enlazan también sus sesiones originales dentro del perfil
  # general de NixOS. Estas máscaras XDG impiden que GDM vuelva a descubrir las
  # variantes que Korunix no ofrece, sin modificar ni recortar los paquetes.
  hiddenUnselectedSessions = pkgs.runCommand "korunix-hidden-unselected-sessions" {} ''
    set -eu

    mask_session() {
      directory="$1"
      filename="$2"

      mkdir -p "$out/share/$directory"

      {
        echo "[Desktop Entry]"
        echo "Type=Application"
        echo "Name=Hidden by Korunix"
        echo "Exec=/run/current-system/sw/bin/false"
        echo "Hidden=true"
        echo "NoDisplay=true"
      } > "$out/share/$directory/$filename"
    }

    # Hyprland se ofrece únicamente mediante UWSM.
    mask_session "wayland-sessions" "hyprland.desktop"

    # Cinnamon se ofrece únicamente mediante su sesión Wayland.
    mask_session "xsessions" "cinnamon.desktop"
    mask_session "xsessions" "cinnamon2d.desktop"

    # Plasma se ofrece únicamente mediante Wayland.
    mask_session "xsessions" "plasmax11.desktop"
  '';

  # Genera copias de los lanzadores con OnlyShowIn. Los ejecutables siguen
  # instalados: únicamente se separa lo que muestra cada menú/launcher.
  desktopVisibilityOverlay = pkgs.runCommand "korunix-desktop-visibility" {} ''
    set -eu

    mkdir -p "$out/share/applications" "$out/etc/xdg/autostart"

    patch_desktop_file() {
      source="$1"
      target="$2"
      desktops="$3"

      mkdir -p "$(dirname "$target")"

      ${pkgs.gawk}/bin/awk \
        -v desktops="$desktops" \
        '
          /^\[Desktop Entry\]$/ {
            print
            print "OnlyShowIn=" desktops ";"
            in_desktop = 1
            next
          }

          in_desktop && /^(OnlyShowIn|NotShowIn)=/ {
            next
          }

          /^\[/ {
            in_desktop = 0
          }

          {
            print
          }
        ' \
        "$source" > "$target"
    }

    patch_package() {
      package="$1"
      desktops="$2"
      directory="$package/share/applications"

      [ -d "$directory" ] || return 0

      for source in "$directory"/*.desktop; do
        [ -f "$source" ] || continue

        target="$out/share/applications/$(basename "$source")"
        patch_desktop_file "$source" "$target" "$desktops"
      done
    }

    patch_autostart_package() {
      package="$1"
      desktops="$2"
      directory="$package/etc/xdg/autostart"

      [ -d "$directory" ] || return 0

      for source in "$directory"/*.desktop; do
        [ -f "$source" ] || continue

        target="$out/etc/xdg/autostart/$(basename "$source")"
        patch_desktop_file "$source" "$target" "$desktops"
      done
    }

    for package in ${lib.escapeShellArgs (
      map toString (
        lib.optionals noctaliaEnabled noctaliaOnlyApplications
      )
    )}; do
      patch_package "$package" "niri;Hyprland"
    done

    for package in ${lib.escapeShellArgs (
      map toString (
        lib.optionals
        (noctaliaEnabled || cinnamonEnabled)
        noctaliaCinnamonApplications
      )
    )}; do
      patch_package "$package" "niri;Hyprland;X-Cinnamon"
    done

    for package in ${lib.escapeShellArgs (
      map toString (
        lib.optionals plasmaEnabled plasmaMenuApplications
      )
    )}; do
      patch_package "$package" "KDE"
    done

    for package in ${lib.escapeShellArgs (
      map toString (
        lib.optionals cinnamonEnabled cinnamonMenuApplications
      )
    )}; do
      patch_package "$package" "X-Cinnamon"
    done

    # Cinnamon necesita estos componentes, pero Niri y Hyprland ya ofrecen red
    # y Bluetooth mediante Noctalia. Se conservan instalados y solo se limita
    # su autoinicio visual a la sesión Cinnamon.
    for package in ${lib.escapeShellArgs (
      map toString (
        lib.optionals cinnamonEnabled [
          pkgs.blueman
          pkgs.networkmanagerapplet
        ]
      )
    )}; do
      patch_autostart_package "$package" "X-Cinnamon"
    done

    # Valent solo pertenece a Niri, Hyprland y Cinnamon.
    for package in ${lib.escapeShellArgs (
      map toString (
        lib.optionals valentDesktopEnabled [pkgs.valent]
      )
    )}; do
      patch_package "$package" "niri;Hyprland;X-Cinnamon"

      patch_desktop_file \
        "$package/etc/xdg/autostart/ca.andyholmes.Valent-autostart.desktop" \
        "$out/etc/xdg/autostart/ca.andyholmes.Valent-autostart.desktop" \
        "niri;Hyprland;X-Cinnamon"
    done

    # Plasma utiliza KDE Connect y no arranca Valent.
    for package in ${lib.escapeShellArgs (
      map toString (
        lib.optionals kdeConnectDesktopEnabled [
          pkgs.kdePackages.kdeconnect-kde
        ]
      )
    )}; do
      patch_package "$package" "KDE"

      patch_desktop_file \
        "$package/etc/xdg/autostart/org.kde.kdeconnect.daemon.desktop" \
        "$out/etc/xdg/autostart/org.kde.kdeconnect.daemon.desktop" \
        "KDE"

      # El indicador non-Plasma no se utiliza: fuera de KDE usamos Valent.
      printf '%s\n' \
        '[Desktop Entry]' \
        'Type=Application' \
        'Name=KDE Connect Indicator' \
        'Exec=/run/current-system/sw/bin/false' \
        'Hidden=true' \
        'NoDisplay=true' \
        > "$out/share/applications/org.kde.kdeconnect.nonplasma.desktop"
    done
  '';

  # El servicio de Noctalia existe porque Niri/Hyprland están instalados,
  # pero solo arranca cuando systemd pertenece realmente a una de esas sesiones.
  # Usamos el entorno importado de la sesión y no buscamos procesos: así se evita
  # una carrera durante el inicio de Hyprland.
  noctaliaSessionCheck = pkgs.writeShellScript "korunix-noctalia-session-check" ''
    case "''${XDG_CURRENT_DESKTOP:-}" in
      niri|Hyprland)
        exit 0
        ;;
    esac

    case "''${XDG_SESSION_DESKTOP:-}" in
      niri|Hyprland)
        exit 0
        ;;
    esac

    case "''${DESKTOP_SESSION:-}" in
      niri|hyprland-uwsm)
        exit 0
        ;;
    esac

    exit 1
  '';

  # DrKonqi pertenece a Plasma. El socket puede existir en cualquier sesión,
  # pero el lanzador solo debe procesar fallos mientras Plasma esté activo.
  # Consultamos el target de Plasma directamente porque el gestor systemd --user
  # no conserva de forma fiable las variables XDG del escritorio.
  plasmaSessionCheck = pkgs.writeShellScript "korunix-plasma-session-check" ''
    exec ${config.systemd.package}/bin/systemctl --user --quiet is-active plasma-workspace.target
  '';

  # El perfil visual de Noctalia usa una base de usuario distinta de la base
  # normal. Así puede cambiar con la paleta sin escribir la preferencia que
  # Cinnamon y Plasma leen al iniciar sus propias sesiones.
  noctaliaDconfProfile = pkgs.writeText "korunix-noctalia-dconf-profile" ''
    user-db:noctalia
  '';

  # Plasma mantiene una base dconf propia para que las preferencias GTK
  # persistidas por Cinnamon no crucen la frontera de sesión.
  # Plasma registra los atajos globales de lanzamiento a partir de las entradas
  # .desktop. Al eliminar Konsole, su Ctrl+Alt+T desaparece; publicamos la entrada
  # oficial de Alacritty con el mismo atajo KDE, sin escribir estado personal.
  plasmaAlacrittyDesktop = pkgs.runCommand "korunix-plasma-alacritty-desktop" {} ''
    install -Dm644 \
      ${pkgs.alacritty}/share/applications/Alacritty.desktop \
      "$out/share/applications/Alacritty.desktop"

    if ${pkgs.gnugrep}/bin/grep -q '^X-KDE-Shortcuts=' \
      "$out/share/applications/Alacritty.desktop"; then
      ${pkgs.gnused}/bin/sed -i \
        's/^X-KDE-Shortcuts=.*/X-KDE-Shortcuts=Ctrl+Alt+T/' \
        "$out/share/applications/Alacritty.desktop"
    else
      ${pkgs.gnused}/bin/sed -i \
        '/^\[Desktop Entry\]$/a X-KDE-Shortcuts=Ctrl+Alt+T' \
        "$out/share/applications/Alacritty.desktop"
    fi
  '';

  plasmaDconfProfile = pkgs.writeText "korunix-plasma-dconf-profile" ''
    user-db:plasma
  '';

  # Hatter Slate acompaña la apariencia predeterminada y las paletas generadas
  # desde el fondo. Hatter Green solo corresponde a Everforest cuando esa es la
  # selección efectiva de Noctalia, incluidos los cambios guardados por su GUI.
  applyNoctaliaIconTheme = pkgs.writeShellScript "korunix-noctalia-icon-theme" ''
    set -eu

    mode="''${1:---default}"
    selection=""

    case "$mode" in
      --default)
        # Antes de iniciar Noctalia no inferimos preferencias desde archivos
        # parciales: Slate es siempre el valor seguro y predeterminado.
        ;;
      --resolved)
        attempt=0

        # La GUI guarda primero su estado y después lo publica por IPC. Esta
        # espera breve evita leer la selección anterior durante ese relevo.
        while [ "$attempt" -lt 40 ]; do
          selection="$(${lib.getExe noctaliaPackage} msg color-scheme-get 2>/dev/null || true)"

          if [ -n "$selection" ]; then
            break
          fi

          attempt=$((attempt + 1))
          ${pkgs.coreutils}/bin/sleep 0.1
        done
        ;;
      *)
        echo "Korunix: modo de sincronización no válido: $mode" >&2
        exit 1
        ;;
    esac

    selection="$(
      printf '%s\n' "$selection" |
        ${pkgs.coreutils}/bin/head -n 1 |
        ${pkgs.coreutils}/bin/tr '[:upper:]' '[:lower:]'
    )"

    case "$selection" in
      "community everforest")
        theme="Hatter-Green"
        ;;
      *)
        theme="Hatter-Slate"
        ;;
    esac

    case "$theme" in
      Hatter-Slate|Hatter-Green)
        ;;
      *)
        echo "Korunix: variante de iconos no válida: $theme" >&2
        exit 1
        ;;
    esac

    DCONF_PROFILE=noctalia \
      ${lib.getExe' pkgs.glib "gsettings"} set \
      org.gnome.desktop.interface \
      icon-theme \
      "$theme"

    if [ -n "$selection" ]; then
      echo "Korunix: Noctalia usa $theme para '$selection'."
    else
      echo "Korunix: Noctalia usa Hatter-Slate como variante predeterminada."
    fi
  '';

  # GTK4 consulta el tema de iconos mediante el portal y no directamente desde
  # el proceso de la aplicación. Los portales comparten unidades entre todos los
  # escritorios, así que el perfil debe elegirse al arrancar cada sesión.
  portalSessionWrapper = name: executable:
    pkgs.writeShellScript name ''
      case ":''${XDG_CURRENT_DESKTOP:-}:''${XDG_SESSION_DESKTOP:-}:''${DESKTOP_SESSION:-}:" in
        *:niri:*|*:Niri:*|*:Hyprland:*|*:hyprland:*|*:hyprland-uwsm:*)
          export DCONF_PROFILE=noctalia
          ${lib.optionalString cinnamonEnabled ''
        export NIX_GSETTINGS_OVERRIDES_DIR=${lib.escapeShellArg neutralGSettingsSchemaDir}
      ''}
          ;;
        *:X-Cinnamon:*|*:cinnamon:*|*:cinnamon-wayland:*)
          export DCONF_PROFILE=user
          ${lib.optionalString cinnamonEnabled ''
        export NIX_GSETTINGS_OVERRIDES_DIR=${lib.escapeShellArg cinnamonGSettingsSchemaDir}
      ''}
          ;;
        *:KDE:*|*:plasma:*)
          export DCONF_PROFILE=plasma
          ${lib.optionalString cinnamonEnabled ''
        export NIX_GSETTINGS_OVERRIDES_DIR=${lib.escapeShellArg neutralGSettingsSchemaDir}
      ''}
          ;;
        *)
          export DCONF_PROFILE=user
          ${lib.optionalString cinnamonEnabled ''
        export NIX_GSETTINGS_OVERRIDES_DIR=${lib.escapeShellArg neutralGSettingsSchemaDir}
      ''}
          ;;
      esac

      exec ${executable}
    '';

  gtkPortalSession =
    portalSessionWrapper
    "korunix-xdg-desktop-portal-gtk"
    "${pkgs.xdg-desktop-portal-gtk}/libexec/xdg-desktop-portal-gtk";

  gnomePortalSession =
    portalSessionWrapper
    "korunix-xdg-desktop-portal-gnome"
    "${pkgs.xdg-desktop-portal-gnome}/libexec/xdg-desktop-portal-gnome";

  # Cinnamon conserva sus defaults de Linux Mint únicamente durante su propia
  # sesión. También actualizamos el entorno de activación de D-Bus y systemd
  # para que los servicios iniciados posteriormente reciban el mismo conjunto.
  cinnamonSessionWrapper = pkgs.writeShellScript "korunix-cinnamon-session" ''
    set -u

    # Cinnamon usa la base dconf normal de la persona y los overrides visuales
    # originales del módulo Cinnamon. Nunca debe heredar el perfil de Noctalia.
    export DCONF_PROFILE=user
    export NIX_GSETTINGS_OVERRIDES_DIR=${lib.escapeShellArg cinnamonGSettingsSchemaDir}

    if ! ${config.systemd.package}/bin/systemctl --user set-environment \
      DCONF_PROFILE=user \
      NIX_GSETTINGS_OVERRIDES_DIR=${lib.escapeShellArg cinnamonGSettingsSchemaDir}
    then
      echo "Korunix: no se pudo publicar el entorno de Cinnamon en systemd." >&2
    fi

    if ! ${pkgs.dbus}/bin/dbus-update-activation-environment \
      --systemd \
      DCONF_PROFILE \
      NIX_GSETTINGS_OVERRIDES_DIR
    then
      echo "Korunix: no se pudo publicar el entorno de Cinnamon en D-Bus." >&2
    fi

    restore_neutral_gsettings() {
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

    trap restore_neutral_gsettings EXIT

    status=0
    ${pkgs.cinnamon-session}/bin/cinnamon-session-cinnamon --wayland "$@" ||
      status=$?

    restore_neutral_gsettings
    trap - EXIT

    exit "$status"
  '';

  # Cinnamon Wayland 6.6 no restaura por sí mismo numlock-state al iniciar.
  # Este ayudante solo puede encender NumLock: comprueba primero los LED reales
  # y, si siguen apagados, inyecta una única pulsación KEY_NUMLOCK mediante uinput.
  cinnamonSessionCheck = pkgs.writeShellScript "korunix-cinnamon-session-check" ''
    case ":''${XDG_CURRENT_DESKTOP:-}:''${XDG_SESSION_DESKTOP:-}:''${DESKTOP_SESSION:-}:" in
      *:X-Cinnamon:*|*:cinnamon:*|*:cinnamon-wayland:*|*:korunix-cinnamon-wayland:*)
        exit 0
        ;;
      *)
        exit 1
        ;;
    esac
  '';

  numlockOnHelper = pkgs.runCommandCC "korunix-numlock-on" {} ''
    mkdir -p "$out/bin"

    cat > numlock-on.c <<'EOF'
    #include <fcntl.h>
    #include <glob.h>
    #include <linux/input.h>
    #include <linux/uinput.h>
    #include <stdio.h>
    #include <string.h>
    #include <sys/ioctl.h>
    #include <unistd.h>

    #define KORUNIX_BITS_PER_LONG (sizeof(unsigned long) * 8)
    #define KORUNIX_NBITS(max) (((max) / KORUNIX_BITS_PER_LONG) + 1)

    static int bit_is_set(const unsigned long *bits, unsigned int bit) {
      return (bits[bit / KORUNIX_BITS_PER_LONG] >>
              (bit % KORUNIX_BITS_PER_LONG)) & 1UL;
    }

    /*
     * Devuelve 1 si NumLock está encendido, 0 si está apagado y -1 si ningún
     * teclado físico expone el estado LED mediante evdev.
     */
    static int numlock_state(void) {
      glob_t paths;
      size_t i;
      int saw_keyboard = 0;

      memset(&paths, 0, sizeof(paths));

      if (glob("/dev/input/event*", 0, NULL, &paths) != 0) {
        return -1;
      }

      for (i = 0; i < paths.gl_pathc; ++i) {
        unsigned long key_bits[KORUNIX_NBITS(KEY_MAX)];
        unsigned long led_bits[KORUNIX_NBITS(LED_MAX)];
        unsigned long led_state[KORUNIX_NBITS(LED_MAX)];
        int fd = open(paths.gl_pathv[i], O_RDONLY | O_NONBLOCK);

        if (fd < 0) {
          continue;
        }

        memset(key_bits, 0, sizeof(key_bits));
        memset(led_bits, 0, sizeof(led_bits));
        memset(led_state, 0, sizeof(led_state));

        if (ioctl(fd, EVIOCGBIT(EV_KEY, sizeof(key_bits)), key_bits) < 0 ||
            ioctl(fd, EVIOCGBIT(EV_LED, sizeof(led_bits)), led_bits) < 0 ||
            !bit_is_set(key_bits, KEY_NUMLOCK) ||
            !bit_is_set(led_bits, LED_NUML)) {
          close(fd);
          continue;
        }

        saw_keyboard = 1;

        if (ioctl(fd, EVIOCGLED(sizeof(led_state)), led_state) >= 0 &&
            bit_is_set(led_state, LED_NUML)) {
          close(fd);
          globfree(&paths);
          return 1;
        }

        close(fd);
      }

      globfree(&paths);
      return saw_keyboard ? 0 : -1;
    }

    static int emit_event(int fd, unsigned short type, unsigned short code, int value) {
      struct input_event event;

      memset(&event, 0, sizeof(event));
      event.type = type;
      event.code = code;
      event.value = value;

      return write(fd, &event, sizeof(event)) == sizeof(event) ? 0 : -1;
    }

    int main(int argc, char **argv) {
      struct uinput_setup setup;
      int fd;

      usleep(1000000);

      int state = numlock_state();

      if (argc > 1 && strcmp(argv[1], "--status") == 0) {
        if (state > 0) {
          puts("on");
          return 0;
        }

        if (state == 0) {
          puts("off");
          return 0;
        }

        puts("unknown");
        return 1;
      }

      if (state > 0) {
        return 0;
      }

      if (state < 0) {
        fprintf(
          stderr,
          "Korunix: no se pudo determinar de forma segura el estado de NumLock.\n"
        );
        return 1;
      }

      fd = open("/dev/uinput", O_WRONLY | O_NONBLOCK);
      if (fd < 0) {
        perror("Korunix: no se pudo abrir /dev/uinput");
        return 1;
      }

      if (ioctl(fd, UI_SET_EVBIT, EV_KEY) < 0 ||
          ioctl(fd, UI_SET_KEYBIT, KEY_NUMLOCK) < 0 ||
          ioctl(fd, UI_SET_EVBIT, EV_SYN) < 0) {
        perror("Korunix: no se pudo preparar uinput");
        close(fd);
        return 1;
      }

      memset(&setup, 0, sizeof(setup));
      setup.id.bustype = BUS_USB;
      setup.id.vendor = 0x4b4f;
      setup.id.product = 0x5255;
      snprintf(setup.name, UINPUT_MAX_NAME_SIZE, "Korunix NumLock");

      if (ioctl(fd, UI_DEV_SETUP, &setup) < 0 ||
          ioctl(fd, UI_DEV_CREATE) < 0) {
        perror("Korunix: no se pudo crear el teclado virtual");
        close(fd);
        return 1;
      }

      usleep(700000);

      if (emit_event(fd, EV_KEY, KEY_NUMLOCK, 1) < 0 ||
          emit_event(fd, EV_SYN, SYN_REPORT, 0) < 0 ||
          emit_event(fd, EV_KEY, KEY_NUMLOCK, 0) < 0 ||
          emit_event(fd, EV_SYN, SYN_REPORT, 0) < 0) {
        perror("Korunix: no se pudo enviar NumLock");
        ioctl(fd, UI_DEV_DESTROY);
        close(fd);
        return 1;
      }

      usleep(150000);
      ioctl(fd, UI_DEV_DESTROY);
      close(fd);
      return 0;
    }
    EOF

    "$CC" -O2 -Wall -Wextra -Werror \
      -o "$out/bin/korunix-numlock-on" \
      numlock-on.c
  '';

  # Los módulos pueden traer varias sesiones, pero Korunix publica una sola
  # sesión Wayland por escritorio. El propio paquete declara exactamente esos
  # nombres a services.displayManager.
  waylandSessions =
    pkgs.runCommand "korunix-wayland-sessions" {
      passthru.providedSessions = waylandSessionNames;
    } ''
      set -eu

      mkdir -p "$out/share/wayland-sessions"
      mkdir -p "$out/share/xsessions"

      ${lib.optionalString niriEnabled ''
        cp \
          ${config.programs.niri.package}/share/wayland-sessions/niri.desktop \
          "$out/share/wayland-sessions/niri.desktop"
      ''}

      ${lib.optionalString hyprlandEnabled ''
              # No reutilizamos hyprland-uwsm.desktop del paquete porque esa entrada
              # vuelve a resolver hyprland.desktop. Korunix oculta esa sesión directa
              # en GDM, por lo que la sesión UWSM debe ser autosuficiente.
              cat > "$out/share/wayland-sessions/hyprland-uwsm.desktop" <<EOF
        [Desktop Entry]
        Name=Hyprland (uwsm-managed)
        Comment=Hyprland Wayland administrado por UWSM
        Exec=${lib.getExe config.programs.uwsm.package} start -- ${config.programs.hyprland.package}/share/wayland-sessions/hyprland.desktop
        Type=Application
        DesktopNames=Hyprland
        EOF
      ''}

      ${lib.optionalString plasmaEnabled ''
        cp \
          ${pkgs.kdePackages.plasma-workspace.sessions}/share/wayland-sessions/plasma.desktop \
          "$out/share/wayland-sessions/plasma.desktop"
      ''}

      ${lib.optionalString cinnamonEnabled ''
        cp \
          ${pkgs.cinnamon}/share/wayland-sessions/cinnamon-wayland.desktop \
          "$out/share/wayland-sessions/cinnamon-wayland.desktop"

        # Cinnamon entra con sus propios overrides en lugar de heredar el
        # conjunto neutral utilizado por el resto de escritorios.
        ${pkgs.gnused}/bin/sed -i \
          "s|^Exec=.*|Exec=${cinnamonSessionWrapper}|" \
          "$out/share/wayland-sessions/cinnamon-wayland.desktop"

        # Cinnamon no declara DesktopNames upstream. Korunix lo hace explícito
        # para poder separar después sus aplicaciones mediante OnlyShowIn.
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

  # El autostart genérico de IBus no debe competir con las sesiones que
  # Korunix administra directamente mediante su modelo XKB.
  ibusEnabled =
    config.i18n.inputMethod.enable
    && config.i18n.inputMethod.type == "ibus";

  ibusPackage = pkgs.ibus-with-plugins.override {
    plugins = config.i18n.inputMethod.ibus.engines;
  };

  ibusPanel = config.i18n.inputMethod.ibus.panel;

  ibusPanelArgument =
    lib.optionalString
    (ibusPanel != null)
    "--panel=${toString ibusPanel}";

  hyprlandMonitorRule = lib.optionalString monitorConfigured ''
    hl.monitor({
        output = ${builtins.toJSON cfg.monitor.output},
        mode = ${builtins.toJSON monitorMode},
    })

  '';

  # Hyprland 0.55+ usa Lua. El archivo humano permanece en config/ y estos
  # marcadores reciben las decisiones específicas del equipo.
  hyprlandConfig = pkgs.writeText "korunix-hyprland.lua" (
    builtins.replaceStrings
    [
      "@KORUNIX_MONITOR_RULE@"
      "@KORUNIX_KEYBOARD_LAYOUTS@"
      "@KORUNIX_KEYBOARD_VARIANTS@"
      "@KORUNIX_KEYBOARD_OPTIONS@"
    ]
    [
      hyprlandMonitorRule
      (lib.concatStringsSep "," keyboardLayouts)
      (lib.concatStringsSep "," keyboardVariants)
      localization.keyboard.switchOption
    ]
    (builtins.readFile ../config/hyprland.lua)
  );
in {
  options.korunix.internal.desktopCatalog = lib.mkOption {
    type = lib.types.listOf lib.types.str;
    readOnly = true;
    internal = true;
    default = implementedDesktops;
    description = "Escritorios que el modelo Nix de Korunix puede implementar.";
  };

  options.korunix.appearance = {
    style = lib.mkOption {
      type = lib.types.enum [
        "default"
        "everforest"
      ];
      default = "default";
      description = ''
        Estilo visual administrado por Korunix. "default" conserva la apariencia
        natural del escritorio; "everforest" expresa la decisión global de usar
        la identidad Everforest donde exista integración soportada.
      '';
    };

    mode = lib.mkOption {
      type = lib.types.enum [
        "light"
        "dark"
        "auto"
      ];
      default = "auto";
      description = ''
        Variante clara, oscura o automática de la apariencia. El modo automático
        sigue la fuente de estado del sistema que Korunix pueda observar de forma
        fiable.
      '';
    };
  };

  options.korunix.desktop = {
    primary = lib.mkOption {
      type = desktopType;
      default = productDefaults.desktop.primary;
      description = "Escritorio que Korunix presenta como sesión principal.";
    };

    additional = lib.mkOption {
      type = lib.types.listOf desktopType;
      default = productDefaults.desktop.additional;
      description = "Otros escritorios disponibles para elegir al iniciar sesión.";
    };

    monitor = {
      output = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        example = "DP-1";
        description = "Salida de vídeo que Korunix configura explícitamente.";
      };

      mode = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        example = "1920x1080";
        description = "Resolución explícita del monitor, sin frecuencia.";
      };

      refreshRate = lib.mkOption {
        type = lib.types.int;
        default = 60;
        description = "Frecuencia del monitor en Hz; Korunix usa 60 Hz por defecto.";
      };
    };
  };

  config = lib.mkIf config.korunix.enable {
    assertions = [
      {
        assertion =
          (cfg.monitor.output == null)
          == (cfg.monitor.mode == null);
        message = "korunix.desktop.monitor.output y mode deben declararse juntos.";
      }
      {
        assertion = cfg.monitor.refreshRate > 0;
        message = "korunix.desktop.monitor.refreshRate debe ser mayor que cero.";
      }
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
      defaultSession = primarySession;

      # Las sesiones aportadas por los módulos se sustituyen por el catálogo
      # Wayland curado de Korunix.
      sessionPackages = lib.mkForce [
        waylandSessions
      ];
    };

    programs.niri.enable = niriEnabled;

    # El módulo NixOS instala Hyprland, XWayland, su portal y la sesión que GDM
    # presenta. UWSM mantiene correctamente los targets systemd de la sesión.
    programs.hyprland = lib.mkIf hyprlandEnabled {
      enable = true;
      withUWSM = true;
      xwayland.enable = true;
    };

    # El backend GTK aporta Settings a las aplicaciones de la experiencia
    # Noctalia incluso cuando Hyprland es el único compositor elegido.
    xdg.portal.extraPortals = lib.mkIf noctaliaEnabled [
      pkgs.xdg-desktop-portal-gtk
    ];

    # Cinnamon conserva tanto el escritorio como su conjunto nativo de
    # aplicaciones. Cada escritorio completo mantiene su propia experiencia.
    services.xserver.desktopManager.cinnamon.enable = cinnamonEnabled;

    # La política de Korunix es NumLock encendido al entrar en cualquier
    # escritorio. Niri, Hyprland y Plasma usan sus mecanismos nativos; Cinnamon
    # Wayland 6.6 recibe el mismo resultado mediante un ayudante mínimo y acotado.
    security.wrappers."korunix-numlock-on" = lib.mkIf cinnamonEnabled {
      source = "${numlockOnHelper}/bin/korunix-numlock-on";
      owner = "root";
      group = "root";
      setuid = true;
      permissions = "u+rx,g+x,o+x";
    };

    systemd.user.services.korunix-cinnamon-numlock = lib.mkIf cinnamonEnabled {
      description = "Enciende NumLock al iniciar Cinnamon";

      wantedBy = ["graphical-session.target"];

      serviceConfig = {
        Type = "oneshot";
        ExecCondition = cinnamonSessionCheck;
        ExecStart = "/run/wrappers/bin/korunix-numlock-on";
      };
    };

    # Cinnamon utiliza Alacritty como terminal predeterminada. GNOME Terminal
    # se excluye del conjunto nativo para no duplicar terminales.
    environment.cinnamon.excludePackages = lib.optionals cinnamonEnabled [
      pkgs.gnome-terminal
    ];

    services.xserver.desktopManager.cinnamon.extraGSettingsOverrides = lib.mkIf cinnamonEnabled (lib.mkAfter ''
      [org.cinnamon.desktop.default-applications.terminal]
      exec='alacritty'
      exec-arg='-e'

      [org.cinnamon.desktop.keybindings.media-keys]
      terminal=['<Primary><Alt>t']
    '');

    # Plasma conserva igualmente su escritorio y su conjunto nativo completo.
    # GDM continúa siendo el gestor de inicio común de Korunix.
    services.desktopManager.plasma6.enable = plasmaEnabled;

    # Korunix usa Alacritty como única terminal. Konsole no se instala aunque
    # forme parte del conjunto opcional predeterminado del módulo Plasma 6.
    environment.plasma6.excludePackages = lib.optionals plasmaEnabled [
      pkgs.kdePackages.konsole
    ];

    # Plasma instala DrKonqi como manejador de coredumps de usuario. Su socket
    # pertenece a sockets.target y, sin este límite, también intenta mostrar
    # fallos dentro de Niri, Hyprland y Cinnamon. Conservamos la unidad original
    # y añadimos únicamente una condición específica de la sesión Plasma.
    systemd.user.services."drkonqi-coredump-launcher@" = lib.mkIf plasmaEnabled {
      overrideStrategy = "asDropin";
      serviceConfig.ExecCondition = plasmaSessionCheck;
    };

    # Plasma carga estos scripts antes de iniciar el escritorio. Esta es la
    # frontera nativa para variables que deben pertenecer únicamente a Plasma.
    environment.etc."xdg/plasma-workspace/env/20-korunix-session.sh" = lib.mkIf plasmaEnabled {
      mode = "0755";
      text = ''
        export DCONF_PROFILE=plasma
        export NIX_GSETTINGS_OVERRIDES_DIR=${lib.escapeShellArg neutralGSettingsSchemaDir}
      '';
    };

    # Al salir de Plasma retiramos su perfil del gestor systemd de usuario. Los
    # demás escritorios publican después su propio entorno al iniciar.
    environment.etc."xdg/plasma-workspace/shutdown/20-korunix-session.sh" = lib.mkIf plasmaEnabled {
      mode = "0755";
      text = ''
        ${config.systemd.package}/bin/systemctl --user set-environment \
          DCONF_PROFILE=user \
          NIX_GSETTINGS_OVERRIDES_DIR=${lib.escapeShellArg neutralGSettingsSchemaDir} ||
          true

        DCONF_PROFILE=user \
        NIX_GSETTINGS_OVERRIDES_DIR=${lib.escapeShellArg neutralGSettingsSchemaDir} \
          ${pkgs.dbus}/bin/dbus-update-activation-environment \
            --systemd \
            DCONF_PROFILE \
            NIX_GSETTINGS_OVERRIDES_DIR ||
          true
      '';
    };

    # Plasma y Dolphin usan Alacritty como terminal predeterminada. La
    # entrada desktop oficial se llama Alacritty.desktop.
    environment.etc."xdg/kdeglobals" = lib.mkIf plasmaEnabled {
      text = ''
        [General]
        TerminalApplication=alacritty
        TerminalService=Alacritty.desktop
      '';
    };

    # Plasma usa kcminputrc para decidir el estado inicial del teclado numérico.
    # NumLock=0 significa «encendido al iniciar Plasma».
    environment.etc."xdg/kcminputrc" = lib.mkIf plasmaEnabled {
      text = ''
        [Keyboard]
        NumLock=0
      '';
    };

    # La copia en /etc/xdg tiene prioridad sobre el autostart aportado por el
    # paquete de IBus. Conservamos su comportamiento original y añadimos solo
    # las sesiones que Korunix administra directamente mediante XKB.
    environment.etc."xdg/autostart/ibus-daemon.desktop" = lib.mkIf (ibusEnabled && (niriEnabled || hyprlandEnabled)) {
      text = ''
        [Desktop Entry]
        Name=IBus
        Type=Application
        Exec=${ibusPackage}/bin/ibus-daemon --daemonize --xim ${ibusPanelArgument}
        # KDE lo integra desde su propio escritorio.
        # Niri y Hyprland usan el teclado XKB administrado por Korunix.
        NotShowIn=KDE;niri;Hyprland;hyprland;
      '';
    };

    # El overlay tiene prioridad sobre los .desktop originales y separa los
    # menús sin desinstalar componentes que cada escritorio necesite.
    environment.systemPackages =
      [
        # También se publica con prioridad en /run/current-system/sw porque
        # Cinnamon enlaza /share completo y, de otro modo, reaparece su .desktop
        # upstream con Exec=cinnamon-session-cinnamon.
        (lib.hiPrio waylandSessions)
        (lib.hiPrio desktopVisibilityOverlay)
        (lib.hiPrio hiddenUnselectedSessions)
      ]
      ++ fixedRolePackages
      ++ [
        roleBrowserLauncher
      ]
      ++ lib.optionals niriEnabled [
        pkgs.xwayland-satellite
      ]
      ++ lib.optionals noctaliaEnabled [
        pkgs.adw-gtk3
      ]
      ++ lib.optionals cinnamonEnabled [
        pkgs.alacritty
      ]
      ++ lib.optionals plasmaEnabled [
        # Prioridad sobre la entrada upstream para que Plasma registre Ctrl+Alt+T.
        (lib.hiPrio plasmaAlacrittyDesktop)
      ]
      ++ lib.optionals noctaliaEnabled noctaliaApplications
      # Si Plasma también está instalado, programs.kdeconnect aporta KDE Connect;
      # Valent se añade aparte para Niri, Hyprland y Cinnamon.
      ++ lib.optionals (plasmaEnabled && valentDesktopEnabled) [
        pkgs.valent
      ];

    # Ambos implementan el mismo protocolo y comparten los puertos 1714-1764.
    # Plasma fija KDE Connect; sin Plasma, Valent es la implementación por defecto.
    programs.kdeconnect = {
      enable = deviceConnectEnabled;
      package = lib.mkDefault (
        if plasmaEnabled
        then pkgs.kdePackages.kdeconnect-kde
        else pkgs.valent
      );
    };

    # Nautilus pertenece a la experiencia Noctalia.
    programs.nautilus-open-any-terminal = lib.mkIf noctaliaEnabled {
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

    # La salida específica del equipo vive separada de la experiencia común.
    environment.etc."korunix/niri-output.kdl" = lib.mkIf (niriEnabled && monitorConfigured) {
      text = ''
        // Archivo generado por Korunix.
        // Edita korunix.desktop.monitor, no este archivo.

        output ${builtins.toJSON cfg.monitor.output} {
          mode ${builtins.toJSON monitorMode}
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

      # El módulo de Cinnamon exporta sus esquemas para todas las sesiones.
      # Korunix conserva los esquemas, pero elimina de este contexto global
      # únicamente los defaults visuales aportados por mint-artwork.
      (lib.mkIf cinnamonEnabled {
        NIX_GSETTINGS_OVERRIDES_DIR =
          lib.mkForce neutralGSettingsSchemaDir;
      })
    ];

    environment.etc."hypr/hyprland.lua" = lib.mkIf hyprlandEnabled {
      source = hyprlandConfig;
    };

    # La base de Noctalia es independiente de user-db:user. El servicio siguiente
    # escribe aquí la variante que corresponde y nunca toca el tema de Cinnamon.
    programs.dconf.profiles.noctalia =
      lib.mkIf noctaliaEnabled noctaliaDconfProfile;

    programs.dconf.profiles.plasma =
      lib.mkIf plasmaEnabled plasmaDconfProfile;

    # Slate se fija antes de que arranquen Noctalia y los portales. Esto evita
    # reutilizar la variante de una sesión anterior mientras el IPC aún no está
    # disponible.
    systemd.user.services.korunix-noctalia-icon-theme-default = lib.mkIf noctaliaEnabled {
      description = "Prepara Hatter Slate para la sesión Noctalia";

      wantedBy = ["graphical-session-pre.target"];
      after = ["korunix-user-prepare.service"];
      requires = ["korunix-user-prepare.service"];
      before = [
        "graphical-session.target"
        "noctalia.service"
        "xdg-desktop-portal-gtk.service"
        "xdg-desktop-portal-gnome.service"
      ];

      serviceConfig = {
        Type = "oneshot";
        ExecCondition = noctaliaSessionCheck;
        ExecStart = "${applyNoctaliaIconTheme} --default";
      };
    };

    # Después de iniciar Noctalia, su IPC informa la selección efectiva. Así las
    # preferencias guardadas por la GUI tienen prioridad sobre el archivo base.
    systemd.user.services.korunix-noctalia-icon-theme-sync = lib.mkIf noctaliaEnabled {
      description = "Sincroniza Hatter con la paleta efectiva de Noctalia";

      wantedBy = ["graphical-session.target"];
      after = ["noctalia.service"];
      wants = ["noctalia.service"];

      serviceConfig = {
        Type = "oneshot";
        ExecCondition = noctaliaSessionCheck;
        ExecStart = "${applyNoctaliaIconTheme} --resolved";
      };
    };

    # Noctalia conserva la configuración humana en config.toml y los cambios de
    # su interfaz en settings.toml. Cualquiera de los dos vuelve a consultar el
    # estado efectivo, sin deducirlo directamente del contenido de esos archivos.
    systemd.user.paths.korunix-noctalia-icon-theme-sync = lib.mkIf noctaliaEnabled {
      description = "Observa la paleta activa de Noctalia";
      wantedBy = ["graphical-session.target"];

      pathConfig = {
        PathChanged = [
          "%h/.config/noctalia/config.toml"
          "%h/.config/noctalia/settings.toml"
        ];
        Unit = "korunix-noctalia-icon-theme-sync.service";
      };
    };

    # En Wayland, GTK4 recibe el tema mediante org.freedesktop.portal.Settings.
    # Estos drop-ins conservan las unidades originales y sustituyen únicamente
    # su ejecutable por un selector que aplica la variante Hatter elegida para
    # Niri/Hyprland y deja el perfil nativo intacto en Cinnamon y Plasma.
    systemd.user.services.xdg-desktop-portal-gtk = lib.mkIf noctaliaEnabled {
      after = ["korunix-noctalia-icon-theme-sync.service"];
      wants = ["korunix-noctalia-icon-theme-sync.service"];

      serviceConfig.ExecStart = [
        ""
        "${gtkPortalSession}"
      ];
    };

    systemd.user.services.xdg-desktop-portal-gnome = lib.mkIf niriEnabled {
      after = ["korunix-noctalia-icon-theme-sync.service"];
      wants = ["korunix-noctalia-icon-theme-sync.service"];

      serviceConfig.ExecStart = [
        ""
        "${gnomePortalSession}"
      ];
    };

    # Noctalia utiliza su módulo NixOS oficial. Su servicio de usuario arranca
    # después de que Korunix haya preparado la configuración de esa persona.
    programs.noctalia = lib.mkIf noctaliaEnabled {
      enable = true;
      package = noctaliaPackage;
      systemd.enable = true;
      recommendedServices.enable = false;
    };

    systemd.user.services.korunix-noctalia-keyboard-labels = lib.mkIf noctaliaEnabled {
      description = "Prepara los nombres humanos del teclado para Noctalia";

      after = ["korunix-user-prepare.service"];
      requires = ["korunix-user-prepare.service"];
      before = ["noctalia.service"];

      serviceConfig = {
        Type = "oneshot";
        ExecCondition = noctaliaSessionCheck;
        ExecStart = prepareKeyboardLabels;
      };
    };

    systemd.user.services.noctalia = lib.mkIf noctaliaEnabled {
      after = [
        "korunix-user-prepare.service"
        "korunix-noctalia-keyboard-labels.service"
        "korunix-noctalia-icon-theme-default.service"
      ];

      requires = [
        "korunix-user-prepare.service"
        "korunix-noctalia-keyboard-labels.service"
      ];

      wants = ["korunix-noctalia-icon-theme-default.service"];

      environment =
        {
          DCONF_PROFILE = "noctalia";
        }
        // lib.optionalAttrs cinnamonEnabled {
          NIX_GSETTINGS_OVERRIDES_DIR = neutralGSettingsSchemaDir;
        };

      serviceConfig.ExecCondition = noctaliaSessionCheck;
    };

    # El TOML contiene valores comunes. El servicio korunix-user-prepare sustituye
    # únicamente datos que dependen de la persona, como avatar y ruta XDG de fotos.
    # Noctalia conserva su plantilla GTK4 integrada. Esta plantilla adicional
    # contiene simultáneamente las variantes clara y oscura de la misma paleta,
    # permitiendo que GTK4 cambie en vivo mediante prefers-color-scheme.
    environment.etc."korunix/noctalia/gtk4-live.css" = lib.mkIf noctaliaEnabled {
      source = ../config/noctalia/gtk4-live.css;
    };

    environment.etc."korunix/noctalia/gtk4-live.toml" = lib.mkIf noctaliaEnabled {
      source = ../config/noctalia/gtk4-live.toml;
    };

    environment.etc."korunix/noctalia/gtk4-live-apply.sh" = lib.mkIf noctaliaEnabled {
      source = ../config/noctalia/gtk4-live-apply.sh;
      mode = "0755";
    };

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

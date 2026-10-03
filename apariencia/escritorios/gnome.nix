{
  config,
  lib,
  pkgs,
  ...
}: let
  uuid = "user-accent-colors@fabito02";
  fondosClaros = ./noctalia/fondos/claro;
  fondosOscuros = ./noctalia/fondos/oscuro;
  dashToDock = pkgs.gnomeExtensions.dash-to-dock;
  appIndicator = pkgs.gnomeExtensions.appindicator;
  roundedUuid = "rounded-windows@marcosgt.github.io";
  alphabeticalUuid = "AlphabeticalAppGrid@stuarthayhurst";

  imageFiles = directory: let
    entries = builtins.readDir directory;
  in
    lib.sort builtins.lessThan (
      lib.filter
      (name:
        entries.${name}
        == "regular"
        && lib.any (suffix: lib.hasSuffix suffix name) [
          ".jpg"
          ".jpeg"
          ".png"
          ".webp"
          ".avif"
        ])
      (builtins.attrNames entries)
    );

  prettyName = name:
    builtins.replaceStrings
    ["-" "_"]
    [" " " "]
    (builtins.replaceStrings
      [".jpeg" ".jpg" ".png" ".webp" ".avif"]
      ["" "" "" "" ""]
      name);

  backgroundEntry = collection: directory: name: ''
    <wallpaper deleted="false">
      <name>${lib.escapeXML "Korunix · ${collection} · ${prettyName name}"}</name>
      <filename>${directory}/${name}</filename>
      <filename-dark>${directory}/${name}</filename-dark>
      <options>zoom</options>
      <shade_type>solid</shade_type>
      <pcolor>#000000</pcolor>
      <scolor>#000000</scolor>
    </wallpaper>
  '';

  gnomeBackgrounds = pkgs.writeTextFile {
    name = "korunix-gnome-backgrounds";
    destination = "/share/gnome-background-properties/korunix.xml";
    text = ''
      <?xml version="1.0"?>
      <!DOCTYPE wallpapers SYSTEM "gnome-wp-list.dtd">
      <wallpapers>
        ${lib.concatMapStrings (backgroundEntry "Claro" fondosClaros) (imageFiles fondosClaros)}
        ${lib.concatMapStrings (backgroundEntry "Oscuro" fondosOscuros) (imageFiles fondosOscuros)}
      </wallpapers>
    '';
  };

  chromaleonSource = builtins.fetchGit {
    url = "https://github.com/Fabito02/ChromaLeon.git";
    rev = "f64d1135f749ed7b6f82777f6c13e30a1cb95e6b";
  };

  chromaleon = pkgs.stdenvNoCC.mkDerivation {
    pname = "gnome-shell-extension-chromaleon";
    version = "2.3.1-korunix";
    src = chromaleonSource;

    nativeBuildInputs = [
      pkgs.glib
      pkgs.python3
    ];
    dontConfigure = true;
    dontBuild = true;

    # ChromaLeon se usa solamente para cambiar la apariencia del Shell de GNOME. Su código
    # original también modifica el tema de GTK y los iconos, pero Korunix necesita evitar
    # que esos cambios compitan con Noctalia y con la ajustes general de GTK.
    # Por eso GTK, Hatter y Noctalia deben partir de una sola ajustes compartida.
    postPatch = ''
            python3 - <<'PY'
      from pathlib import Path
      import re

      path = Path("extension.js")
      text = path.read_text()

      text, icon_count = re.subn(
          r"  async _updateIconPack\(cancellable\) \{.*?\n  \}\n\n  async _updateStyles",
          """  async _updateIconPack(cancellable) {
          throwIfCancelled(cancellable);
        }

        async _updateStyles""",
          text,
          count=1,
          flags=re.S,
      )

      text, app_count = re.subn(
          r"  async _updateAppStyles\(cancellable\) \{.*?\n  \}\n\n  _shouldUseLightShell",
          """  async _updateAppStyles(cancellable) {
          throwIfCancelled(cancellable);
        }

        _shouldUseLightShell""",
          text,
          count=1,
          flags=re.S,
      )

      if icon_count != 1 or app_count != 1:
          raise SystemExit(
              f"No se pudo limitar ChromaLeon al Shell: iconos={icon_count} GTK={app_count}"
          )

      path.write_text(text)
      PY
    '';

    installPhase = ''
      extension=$out/share/gnome-shell/extensions/${uuid}
      mkdir -p "$extension"
      cp -r . "$extension/"
      glib-compile-schemas "$extension/schemas"
    '';
  };

  roundedWindowsSource = builtins.fetchGit {
    url = "https://github.com/Nathanaelrc/rounded-windows.git";
    rev = "9d9eb77013b24e45ae75fc92a85a9b6d82e052f6";
  };

  # Steam y Visual Studio Code necesitan que GNOME les aplique el redondeo de ventanas, porque
  # sus ventanas no siempre reciben las mismas esquinas que las aplicaciones normales de GNOME.
  roundedWindows = pkgs.stdenvNoCC.mkDerivation {
    pname = "gnome-shell-extension-rounded-windows-korunix";
    version = "2.2.0";
    src = roundedWindowsSource;
    nativeBuildInputs = [pkgs.glib];
    dontConfigure = true;
    dontBuild = true;

    installPhase = ''
      extension=$out/share/gnome-shell/extensions/${roundedUuid}
      mkdir -p "$extension"
      cp extension.js effect.js prefs.js metadata.json stylesheet.css "$extension/"
      cp -r schemas "$extension/"
      glib-compile-schemas "$extension/schemas"
    '';
  };

  alphabeticalGridSource = builtins.fetchGit {
    url = "https://github.com/stuarthayhurst/alphabetical-grid-extension.git";
    rev = "bdd0bd07469c9df5dd1fbaca4a59f841da619549";
  };

  alphabeticalGrid = pkgs.stdenvNoCC.mkDerivation {
    pname = "gnome-shell-extension-alphabetical-app-grid-korunix";
    version = "46";
    src = alphabeticalGridSource;
    nativeBuildInputs = [pkgs.glib];
    dontConfigure = true;
    dontBuild = true;

    installPhase = ''
      extension=$out/share/gnome-shell/extensions/${alphabeticalUuid}
      mkdir -p "$extension"
      cp -r extension/. "$extension/"
      glib-compile-schemas "$extension/schemas"
    '';
  };

  gnomeShellPalette = pkgs.writeShellApplication {
    name = "korunix-gnome-shell-palette";
    runtimeInputs = [
      pkgs.coreutils
      pkgs.dconf
      pkgs.glib
      pkgs.gnome-shell
      pkgs.gnused
      pkgs.jq
    ];
    text = ''
      extension_dir="${chromaleon}/share/gnome-shell/extensions/${uuid}"
      config_home="''${XDG_CONFIG_HOME:-$HOME/.config}"
      palette="''${1:-}"
      mode="''${2:-dark}"

      export GSETTINGS_SCHEMA_DIR="$extension_dir/schemas"

      # Este bloque elimina una vez los archivos CSS que una versión anterior de
      # ChromaLeon pudo haber dejado dentro de GTK. En las versiones actuales la
      # extensión ya no vuelve a crearlos.
      for version in 3 4; do
        gtk_dir="$config_home/gtk-$version.0"
        gtk_css="$gtk_dir/gtk.css"
        if [ -f "$gtk_css" ]; then
          sed -i \
            '/\/\* CustomAccentExtension Start \*\//,/\/\* CustomAccentExtension End \*\//d' \
            "$gtk_css"
        fi
        rm -f "$gtk_dir/custom-accent.css"
      done

      # ChromaLeon solo modifica el Shell. GTK, Qt y las aplicaciones que usan las
      # plantillas de Korunix reciben directamente la misma paleta de colores.
      gsettings set org.gnome.shell.extensions.chromaleon gnome-colors false
      gsettings set org.gnome.shell.extensions.chromaleon custom-color true
      gsettings set org.gnome.shell.extensions.chromaleon tint-shell true
      gsettings set org.gnome.shell.extensions.chromaleon tint-panel true
      gsettings set org.gnome.shell.extensions.chromaleon tint-apps false
      gsettings set org.gnome.shell.extensions.chromaleon tint-gtk3 false
      gsettings set org.gnome.shell.extensions.chromaleon persistent-choices false
      gsettings set org.gnome.shell.extensions.chromaleon recolor-folders false
      gsettings set org.gnome.shell.extensions.chromaleon recolor-apps false

      if [ -n "$palette" ] && [ -s "$palette" ]; then
        primary="$(jq -r --arg mode "$mode" '.[$mode].primary // empty' "$palette")"
        if [ -n "$primary" ] && [ "$primary" != null ]; then
          gsettings set org.gnome.shell.extensions.chromaleon accent-color "$primary"
        fi
      fi

      gsettings set org.gnome.desktop.interface icon-theme Hatter-Slate >/dev/null 2>&1 || true
      gsettings set org.gnome.desktop.interface cursor-theme Bibata-Modern-Classic >/dev/null 2>&1 || true
      gsettings set org.gnome.desktop.interface cursor-size 24 >/dev/null 2>&1 || true

      # Este atajo hace que GNOME abra la terminal de Korunix con Ctrl+Alt+T, igual que
      # Cinnamon, sin depender de que GNOME Console esté instalada.
      dconf write /org/gnome/settings-daemon/plugins/media-keys/custom-keybindings "['/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/korunix-terminal/']"
      dconf write /org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/korunix-terminal/name "'Alacritty'"
      dconf write /org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/korunix-terminal/command "'alacritty'"
      dconf write /org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/korunix-terminal/binding "'<Control><Alt>t'"

      # El dock de GNOME se configura para parecerse al dock que usa Noctalia.
      dconf write /org/gnome/shell/extensions/dash-to-dock/dock-position "'BOTTOM'"
      dconf write /org/gnome/shell/extensions/dash-to-dock/dash-max-icon-size 40
      dconf write /org/gnome/shell/extensions/dash-to-dock/icon-size-fixed true
      dconf write /org/gnome/shell/extensions/dash-to-dock/extend-height false
      dconf write /org/gnome/shell/extensions/dash-to-dock/dock-fixed true
      dconf write /org/gnome/shell/extensions/dash-to-dock/autohide false
      dconf write /org/gnome/shell/extensions/dash-to-dock/intellihide false
      dconf write /org/gnome/shell/extensions/dash-to-dock/multi-monitor true
      dconf write /org/gnome/shell/extensions/dash-to-dock/show-favorites true
      dconf write /org/gnome/shell/extensions/dash-to-dock/show-running true
      dconf write /org/gnome/shell/extensions/dash-to-dock/show-show-apps-button true
      dconf write /org/gnome/shell/extensions/dash-to-dock/show-apps-at-top true
      dconf write /org/gnome/shell/extensions/dash-to-dock/show-apps-always-in-the-edge true
      dconf write /org/gnome/shell/extensions/dash-to-dock/show-trash false
      dconf write /org/gnome/shell/extensions/dash-to-dock/show-mounts false
      dconf write /org/gnome/shell/extensions/dash-to-dock/transparency-mode "'FIXED'"
      dconf write /org/gnome/shell/extensions/dash-to-dock/background-opacity 0.5
      dconf write /org/gnome/shell/extensions/dash-to-dock/running-indicator-style "'DOTS'"

      # Estas aplicaciones aparecen fijadas en el mismo orden que en el dock de Noctalia.
      dconf write /org/gnome/shell/favorite-apps "['firefox.desktop', 'org.gnome.Nautilus.desktop', 'spotify.desktop', 'steam.desktop', 'net.lutris.Lutris.desktop', 'anime-game-launcher.desktop', 'honkers-railway-launcher.desktop', 'vesktop.desktop', 'org.localsend.localsend_app.desktop', 'code.desktop', 'com.obsproject.Studio.desktop', 'org.kde.kdenlive.desktop', 'com.heroicgameslauncher.hgl.desktop', 'onlyoffice-desktopeditors.desktop', 'birdfont.desktop']"

      # La lista de aplicaciones se ordena alfabéticamente,
      # incluso cuando están dentro de una carpeta.
      dconf write /org/gnome/shell/extensions/alphabetical-app-grid/sort-folder-contents true
      dconf write /org/gnome/shell/extensions/alphabetical-app-grid/folder-order-position "'alphabetical'"
      dconf write /org/gnome/shell/extensions/alphabetical-app-grid/show-favourite-apps false

      # Estas reglas hacen que Steam, steamwebhelper y Visual Studio Code tengan esquinas redondeadas.
      # Se incluyen los nombres que pueden aparecer cuando Code usa XWayland
      # o cuando GNOME identifica la aplicación de otra forma.
      dconf write /org/gnome/shell/extensions/rounded-windows/corner-radius 12
      dconf write /org/gnome/shell/extensions/rounded-windows/smoothing 0.6
      dconf write /org/gnome/shell/extensions/rounded-windows/border-width 0
      dconf write /org/gnome/shell/extensions/rounded-windows/custom-shadow true
      dconf write /org/gnome/shell/extensions/rounded-windows/whitelist-mode true
      dconf write /org/gnome/shell/extensions/rounded-windows/blacklist "['steam', 'steamwebhelper', 'code', 'Code', 'Visual Studio Code']"
      dconf write /org/gnome/shell/extensions/rounded-windows/keep-rounded-maximized false
      dconf write /org/gnome/shell/extensions/rounded-windows/keep-rounded-fullscreen false

      gnome-extensions enable ${uuid} >/dev/null 2>&1 || true
      gnome-extensions enable ${dashToDock.extensionUuid} >/dev/null 2>&1 || true
      gnome-extensions enable ${appIndicator.extensionUuid} >/dev/null 2>&1 || true
      gnome-extensions enable ${roundedUuid} >/dev/null 2>&1 || true
      gnome-extensions enable ${alphabeticalUuid} >/dev/null 2>&1 || true
    '';
  };

  # GNOME identifica las ventanas Wayland mediante app_id. Estas entradas ocultas dejan
  # que Obsidian y LocalSend usen el icono correcto sin aparecer dos veces en el lanzador.
  obsidianAlias = pkgs.makeDesktopItem {
    name = "md.obsidian.Obsidian";
    desktopName = "Obsidian";
    exec = "obsidian %u";
    icon = "obsidian";
    startupWMClass = "md.obsidian.Obsidian";
    noDisplay = true;
  };

  localSendAlias = pkgs.makeDesktopItem {
    name = "org.localsend.localsend_app";
    desktopName = "LocalSend";
    exec = "localsend_app %U";
    icon = "localsend";
    startupWMClass = "localsend_app";
    noDisplay = true;
  };
in {
  services.desktopManager.gnome.enable = true;
  services.desktopManager.gnome.sessionPath = [
    alphabeticalGrid
    chromaleon
    roundedWindows
  ];

  environment.systemPackages = [
    alphabeticalGrid
    appIndicator
    dashToDock
    gnomeBackgrounds
    gnomeShellPalette
    obsidianAlias
    localSendAlias
    roundedWindows
  ];

  environment.etc."xdg/autostart/korunix-gnome-shell-palette.desktop".text = ''
    [Desktop Entry]
    Type=Application
    Name=Korunix · GNOME Shell
    Comment=Conserva Hatter y Bibata y configura el Shell de GNOME de Korunix
    Exec=${gnomeShellPalette}/bin/korunix-gnome-shell-palette
    OnlyShowIn=GNOME;
    NoDisplay=true
    X-GNOME-Autostart-enabled=true
  '';
}

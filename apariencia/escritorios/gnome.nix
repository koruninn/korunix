{
  config,
  pkgs,
  ...
}: let
  uuid = "user-accent-colors@fabito02";

  chromaleonSource = builtins.fetchGit {
    url = "https://github.com/Fabito02/ChromaLeon.git";
    rev = "f64d1135f749ed7b6f82777f6c13e30a1cb95e6b";
  };

  chromaleon = pkgs.stdenvNoCC.mkDerivation {
    pname = "gnome-shell-extension-chromaleon";
    version = "2.3.1";
    src = chromaleonSource;

    nativeBuildInputs = [pkgs.glib];
    dontConfigure = true;
    dontBuild = true;

    installPhase = ''
      extension=$out/share/gnome-shell/extensions/${uuid}
      mkdir -p "$extension"
      cp -r . "$extension/"
      glib-compile-schemas "$extension/schemas"
    '';
  };

  gnomeShellPalette = pkgs.writeShellApplication {
    name = "korunix-gnome-shell-palette";
    runtimeInputs = [
      pkgs.glib
      pkgs.gnome-shell
      pkgs.jq
    ];
    text = ''
      extension_dir="${chromaleon}/share/gnome-shell/extensions/${uuid}"
      palette="''${1:-}"
      mode="''${2:-dark}"

      export GSETTINGS_SCHEMA_DIR="$extension_dir/schemas"

      # ChromaLeon queda limitado al Shell. GTK, Qt y las aplicaciones con
      # plantillas consumen directamente la paleta M3 generada por Korunix.
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

      gnome-extensions enable ${uuid} >/dev/null 2>&1 || true
    '';
  };

  # GNOME empareja ventanas Wayland por app_id. Estas entradas ocultas evitan
  # los iconos genéricos de Obsidian y LocalSend sin duplicarlos en el launcher.
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
    exec = "localsend";
    icon = "localsend";
    startupWMClass = "org.localsend.localsend_app";
    noDisplay = true;
  };
in {
  services.desktopManager.gnome.enable = true;
  services.desktopManager.gnome.sessionPath = [chromaleon];

  environment.systemPackages = [
    gnomeShellPalette
    obsidianAlias
    localSendAlias
  ];

  environment.etc."xdg/autostart/korunix-gnome-shell-palette.desktop".text = ''
[Desktop Entry]
Type=Application
Name=Korunix · GNOME Shell
Comment=Conserva Hatter y Bibata y limita ChromaLeon al Shell de GNOME
Exec=${gnomeShellPalette}/bin/korunix-gnome-shell-palette
OnlyShowIn=GNOME;
NoDisplay=true
X-GNOME-Autostart-enabled=true
  '';
}

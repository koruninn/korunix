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
    version = "2.3.1-korunix";
    src = chromaleonSource;

    nativeBuildInputs = [
      pkgs.glib
      pkgs.python3
    ];
    dontConfigure = true;
    dontBuild = true;

    # ChromaLeon sirve únicamente como renderer del GNOME Shell. Su código
    # normal siempre escribe gtk.css y genera su propio tema de iconos incluso
    # cuando el tintado de aplicaciones está desactivado; Korunix necesita que
    # GTK y Hatter tengan un único propietario compartido con Noctalia.
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

  gnomeShellPalette = pkgs.writeShellApplication {
    name = "korunix-gnome-shell-palette";
    runtimeInputs = [
      pkgs.coreutils
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

      # Limpia una sola vez cualquier CSS que una versión anterior de
      # ChromaLeon hubiese dejado dentro de GTK. A partir de este build la
      # extensión ya no vuelve a escribir estos archivos.
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
    exec = "localsend_app %U";
    icon = "localsend";
    startupWMClass = "localsend_app";
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

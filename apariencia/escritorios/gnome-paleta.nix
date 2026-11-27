{pkgs, ...}: let
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

    # ChromaLeon solo cambia GNOME Shell para no competir con los temas de Noctalia.
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


  palette = pkgs.writeShellApplication {
    name = "korunix-gnome-shell-palette";
    runtimeInputs = [pkgs.glib pkgs.jq];
    text = ''
      export GSETTINGS_SCHEMA_DIR="${chromaleon}/share/gnome-shell/extensions/${uuid}/schemas"
      archivo="''${1:-}"
      modo="''${2:-dark}"
      [ -s "$archivo" ] || exit 0
      case "$modo" in light|dark) ;; *) exit 1 ;; esac
      primary="$(jq -r --arg mode "$modo" '.[$mode].primary // empty' "$archivo")"
      [ -n "$primary" ] || exit 0
      gsettings set org.gnome.shell.extensions.chromaleon gnome-colors false
      gsettings set org.gnome.shell.extensions.chromaleon custom-color true
      gsettings set org.gnome.shell.extensions.chromaleon tint-shell true
      gsettings set org.gnome.shell.extensions.chromaleon tint-panel true
      gsettings set org.gnome.shell.extensions.chromaleon tint-apps false
      gsettings set org.gnome.shell.extensions.chromaleon tint-gtk3 false
      gsettings set org.gnome.shell.extensions.chromaleon persistent-choices false
      gsettings set org.gnome.shell.extensions.chromaleon recolor-folders false
      gsettings set org.gnome.shell.extensions.chromaleon recolor-apps false
      gsettings set org.gnome.shell.extensions.chromaleon accent-color "$primary"
    '';
  };
in {
  services.desktopManager.gnome.sessionPath = [chromaleon];
  environment.systemPackages = [chromaleon palette];
}

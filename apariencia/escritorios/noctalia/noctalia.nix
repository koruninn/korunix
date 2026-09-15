{
  config,
  equipo,
  lib,
  pkgs,
  ...
}: let
  usuario = config.users.users.${equipo.persona};
  plugins = import ./plugins.nix;
  toml = pkgs.formats.toml {};
  python = pkgs.python3.withPackages (ps: [ps.tomli-w]);
  ajustes = import ./ajustes.nix {inherit equipo lib plugins;};
  noctaliaConfig = toml.generate "noctalia-config.toml" ajustes;

  noctaliaConfigSync = pkgs.writeShellApplication {
    name = "korunix-noctalia-config-sync";
    runtimeInputs = [pkgs.coreutils pkgs.xdg-user-dirs];
    text = ''
      home=${lib.escapeShellArg usuario.home}
      owner=${lib.escapeShellArg usuario.name}
      group=${lib.escapeShellArg usuario.group}
      config_dir="$home/.config/noctalia"
      settings_file="$home/.local/state/noctalia/settings.toml"

      install -d -m 0755 -o "$owner" -g "$group" "$config_dir"

      # Limpia únicamente las secciones que Korunix administra. El parser TOML
      # evita depender de la posición de las líneas y wallpaper queda intacto.
      if [ -f "$settings_file" ]; then
        ${python}/bin/python - "$settings_file" <<'PY'
import os
from pathlib import Path
import shutil
import sys
import tomllib
import tomli_w

settings = Path(sys.argv[1])
with settings.open("rb") as handle:
    data = tomllib.load(handle)

managed_top = {
    "accessibility",
    "shell",
    "location",
    "control_center",
    "bar",
    "dock",
    "osd",
    "lockscreen",
    "calendar",
    "plugins",
    "theme",
}
managed_widgets = {
    "fecha",
    "weather",
    "media",
    "cat",
    "bongo_cat",
    "calculator",
    "pomodoro_timer",
    "notes",
    "udiskie_manager",
    "tray",
    "network",
    "bluetooth",
    "lock_keys",
    "volume_input",
    "temperatura",
    "umbriel_displays",
    "umbriel_companion",
    "speedtest_meter",
    "phone_connect",
    "printers",
    "red_rx",
    "red_tx",
    "privacy",
    "screen_recorder",
}

changed = False
for key in managed_top:
    if key in data:
        del data[key]
        changed = True

widgets = data.get("widget")
if isinstance(widgets, dict):
    for key in managed_widgets:
        if key in widgets:
            del widgets[key]
            changed = True
    if not widgets:
        del data["widget"]

plugin_settings = data.get("plugin_settings")
if isinstance(plugin_settings, dict) and "noctalia/notes" in plugin_settings:
    del plugin_settings["noctalia/notes"]
    changed = True
    if not plugin_settings:
        del data["plugin_settings"]

if changed:
    backup = settings.with_name(settings.name + ".korunix-backup")
    shutil.copy2(settings, backup)
    temporary = settings.with_name(settings.name + ".korunix-tmp")
    with temporary.open("wb") as handle:
        tomli_w.dump(data, handle)
    os.replace(temporary, settings)
PY
        chown "$owner:$group" "$settings_file"
        if [ -f "$settings_file.korunix-backup" ]; then
          chown "$owner:$group" "$settings_file.korunix-backup"
        fi
      fi

      install -m 0644 -o "$owner" -g "$group" ${noctaliaConfig} "$config_dir/config.toml"

      # Las rutas personales se resuelven desde XDG, así funcionan aunque la
      # carpeta se llame Imágenes, Pictures u otra traducción.
      documents_dir="$(HOME="$home" xdg-user-dir DOCUMENTS)"
      pictures_dir="$(HOME="$home" xdg-user-dir PICTURES)"
      [ -n "$documents_dir" ] || documents_dir="$home/Documents"
      [ -n "$pictures_dir" ] || pictures_dir="$home/Pictures"

      notes_dir="$documents_dir/Notes"
      screenshots_dir="$pictures_dir/Capturas de pantalla"
      install -d -m 0755 -o "$owner" -g "$group" "$notes_dir" "$screenshots_dir"

      xdg_file="$config_dir/zz-korunix-xdg.toml"
      ${python}/bin/python - "$xdg_file" "$notes_dir" "$screenshots_dir" <<'PY'
from pathlib import Path
import sys
import tomli_w

output = Path(sys.argv[1])
notes = sys.argv[2]
screenshots = sys.argv[3]
data = {
    "plugin_settings": {
        "noctalia/notes": {
            "notes_dir": notes,
        },
    },
    "shell": {
        "screenshot": {
            "directory": screenshots,
        },
    },
}
with output.open("wb") as handle:
    tomli_w.dump(data, handle)
PY
      chown "$owner:$group" "$xdg_file"
      chmod 0644 "$xdg_file"
    '';
  };
in {
  programs.noctalia.enable = true;
  environment.etc."noctalia/config.toml".source = noctaliaConfig;

  # La lógica se construye como programa antes de activar el sistema. Un error
  # de shell falla durante el build, no a mitad de nixos-rebuild switch.
  system.activationScripts.noctaliaConfig.text = ''
    ${noctaliaConfigSync}/bin/korunix-noctalia-config-sync
  '';
}

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
  widgetsAdministrados = builtins.toJSON (builtins.attrNames ajustes.widget);

  noctaliaConfigSync = pkgs.writeShellApplication {
    name = "korunix-noctalia-config-sync";
    runtimeInputs = [pkgs.coreutils pkgs.xdg-user-dirs];
    text = ''
      home=${lib.escapeShellArg usuario.home}
      owner=${lib.escapeShellArg usuario.name}
      group=${lib.escapeShellArg usuario.group}
      config_home="$home/.config"
      state_home="$home/.local/state"
      config_dir="$config_home/noctalia"
      settings_file="$state_home/noctalia/settings.toml"

      install -d -m 0755 -o "$owner" -g "$group" "$config_dir"

      # Korunix elimina únicamente lo que administra. La lista de widgets sale
      # directamente de ajustes.nix y wallpaper queda deliberadamente fuera.
      if [ -f "$settings_file" ]; then
        ${python}/bin/python - "$settings_file" ${lib.escapeShellArg widgetsAdministrados} <<'PY'
import json
import os
from pathlib import Path
import shutil
import sys
import tomllib
import tomli_w

settings = Path(sys.argv[1])
managed_widgets = set(json.loads(sys.argv[2]))
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

      # Las carpetas personales se preguntan a XDG; no dependen del idioma ni
      # de un nombre como Imágenes/Pictures o Documentos/Documents.
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
    "plugin_settings": {"noctalia/notes": {"notes_dir": notes}},
    "shell": {"screenshot": {"directory": screenshots}},
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

  system.activationScripts.noctaliaConfig.text = ''
    ${noctaliaConfigSync}/bin/korunix-noctalia-config-sync
  '';
}

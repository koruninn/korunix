{config, equipo, lib, pkgs, ...}: let
  noctalia = config.programs.noctalia.package;
  gtkSession = pkgs.writeShellApplication {
    name = "korunix-gtk-session";
    runtimeInputs = [pkgs.coreutils pkgs.glib pkgs.python3 pkgs.bash];
    text = ''
      export XDG_DATA_DIRS="${pkgs.glib.getSchemaDataDirPath pkgs.gsettings-desktop-schemas}:''${XDG_DATA_DIRS:-/run/current-system/sw/share}"
      [ "''${1:-}" = noctalia-session ] || exit 2
      mode="''${2:-}"
      case "$mode" in light|dark) ;; *) exit 2 ;; esac
      config_home="''${XDG_CONFIG_HOME:-$HOME/.config}"
      if [ -f "$config_home/gtk-3.0/noctalia.css" ] && [ -f "$config_home/gtk-4.0/noctalia.css" ]; then
        bash "${noctalia}/share/noctalia/assets/templates/gtk/apply.sh" "$mode" || echo "No se pudo aplicar la importación GTK" >&2
      fi

      # Dejar que libadwaita siga el modo del escritorio.
      python3 - "$config_home" "$mode" <<'PY'
      import pathlib, re, sys
      root = pathlib.Path(sys.argv[1])
      mode = sys.argv[2]
      for v in (3, 4):
          path = root / f"gtk-{v}.0" / "settings.ini"
          path.parent.mkdir(parents=True, exist_ok=True)
          old = path.read_text() if path.exists() else "[Settings]\n"
          # Conservar las preferencias personales que no tienen que ver con el tema.
          keys = ["gtk-icon-theme-name", "gtk-application-prefer-dark-theme",
                  "gtk-cursor-theme-name", "gtk-cursor-theme-size"]
          if v == 3:
              keys.append("gtk-theme-name")
          lines = [line for line in old.splitlines() if not any(re.match(r"\s*" + key + r"\s*=", line) for key in keys)]
          if "[Settings]" not in lines:
              lines.insert(0, "[Settings]")
          pos = lines.index("[Settings]") + 1
          values = ["gtk-icon-theme-name=Hatter-Slate",
                    "gtk-cursor-theme-name=Bibata-Modern-Classic", "gtk-cursor-theme-size=24"]
          if v == 3:
              values += ["gtk-theme-name=" + ("adw-gtk3-dark" if mode == "dark" else "adw-gtk3"),
                         "gtk-application-prefer-dark-theme=" + ("1" if mode == "dark" else "0")]
          lines[pos:pos] = values
          new = "\n".join(lines) + "\n"
          if new != old:
              path.write_text(new)
      PY
      gsettings set org.gnome.desktop.interface icon-theme Hatter-Slate
      gsettings set org.gnome.desktop.interface cursor-theme Bibata-Modern-Classic
      gsettings set org.gnome.desktop.interface cursor-size 24
      # Cambiar el modo después de aplicar los colores.
      gsettings set org.gnome.desktop.interface color-scheme "prefer-$mode"
    '';
  };
in lib.mkIf (equipo.habilitarUmbriel or false) {
  environment.systemPackages = [pkgs.adw-gtk3 gtkSession];
  environment.etc."gtk-3.0/settings.ini".text = ''
    [Settings]
    gtk-icon-theme-name=Hatter-Slate
    gtk-cursor-theme-name=Bibata-Modern-Classic
    gtk-cursor-theme-size=24
  '';
  environment.etc."gtk-4.0/settings.ini".text = ''
    [Settings]
    gtk-icon-theme-name=Hatter-Slate
    gtk-cursor-theme-name=Bibata-Modern-Classic
    gtk-cursor-theme-size=24
  '';
}

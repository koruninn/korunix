{pkgs, ...}: let
  terminalShortcut = pkgs.writeShellApplication {
    name = "korunix-terminal-shortcut";
    runtimeInputs = [pkgs.python3 pkgs.glib];
    text = ''
      export XDG_DATA_DIRS="${pkgs.glib.getSchemaDataDirPath pkgs.gsettings-desktop-schemas}:''${XDG_DATA_DIRS:-/run/current-system/sw/share}"
      # Cambiar solo el atajo de Korunix y respetar los demás.
      python3 - <<'PY'
      import ast
      import subprocess
      schema = "org.gnome.settings-daemon.plugins.media-keys"
      key = "custom-keybindings"
      path = "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/korunix-terminal/"
      raw = subprocess.check_output(["gsettings", "get", schema, key], text=True).strip()
      paths = ast.literal_eval(raw.removeprefix("@as "))
      custom = schema + ".custom-keybinding:" + path
      for field, value in (("name", "Ptyxis"), ("command", "ptyxis"), ("binding", "<Control><Alt>t")):
          subprocess.run(["gsettings", "set", custom, field, repr(value)], check=True)
      if path not in paths:
          paths.append(path)
          subprocess.run(["gsettings", "set", schema, key, repr(paths)], check=True)
      PY
    '';
  };
in {
  environment.systemPackages = [pkgs.ptyxis terminalShortcut];
  environment.etc."xdg/autostart/korunix-terminal-shortcut.desktop".text = ''
    [Desktop Entry]
    Type=Application
    Name=Korunix · Atajo de terminal
    Exec=${terminalShortcut}/bin/korunix-terminal-shortcut
    OnlyShowIn=GNOME;
    NoDisplay=true
    X-GNOME-Autostart-enabled=true
  '';
  environment.gnome.excludePackages = [pkgs.gnome-console];
  environment.sessionVariables.TERMINAL = "ptyxis";

  xdg.terminal-exec = {
    enable = true;
    settings.default = ["org.gnome.Ptyxis.desktop"];
  };

  # Ocultar XTerm del menú.
  services.xserver.excludePackages = [pkgs.xterm];
  services.xserver.desktopManager.xterm.enable = false;
}

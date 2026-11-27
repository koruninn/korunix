{config, equipo, inputs, lib, pkgs, ...}: let
  usuario = config.users.users.koru;
  luciole = inputs.luciole.packages.${pkgs.stdenv.hostPlatform.system}.default;
  configLuciole = (pkgs.formats.toml {}).generate "luciole-config.toml" {
    schema_version = 1;
    browser = {
      color_scheme = "auto";
      terminal_command = "ptyxis";
      keybinds = {
        back = ["Alt+Left"];
        forward = ["Alt+Right"];
        up = ["Alt+Up"];
        home_folder = ["Alt+Home"];
        edit_location = ["Ctrl+L"];
        search = ["Ctrl+F" "Ctrl+K"];
        toggle_hidden = ["Ctrl+H"];
        toggle_preview = ["F8"];
        quick_preview = ["space"];
        new_tab = ["Ctrl+T"];
        close_tab = ["Ctrl+W"];
        new_window = ["Ctrl+N"];
        split_view = ["F3"];
        terminal = ["F4"];
        rename = ["F2"];
        create_folder = ["Ctrl+Shift+N"];
        keyboard_shortcuts = ["F1"];
      };
    };
  };
  # Separar el explorador predeterminado de cada escritorio y guardar una copia de sus asociaciones.
  mimePorEscritorio = pkgs.writeText "korunix-luciole-mime.py" ''
    import os
    import re
    import shutil
    import sys
    from pathlib import Path

    home = Path(sys.argv[1])
    config = home / ".config"
    config.mkdir(parents=True, exist_ok=True)
    for desktop, handler in (("umbriel", "dev.noctalia.luciole.desktop"),
                             ("gnome", "org.gnome.Nautilus.desktop")):
        path = config / (desktop + "-mimeapps.list")
        lines = path.read_text().splitlines() if path.exists() else []
        header = "[Default Applications]"
        value = "inode/directory=" + handler + ";"
        if header not in lines:
            if lines and lines[-1]:
                lines.append("")
            lines += [header, value]
        else:
            start = lines.index(header) + 1
            end = next((i for i in range(start, len(lines)) if lines[i].lstrip().startswith("[")), len(lines))
            keys = [i for i in range(start, end) if re.match(r"^\s*inode/directory\s*=", lines[i])]
            if keys:
                lines[keys[0]] = value
                for i in reversed(keys[1:]):
                    del lines[i]
            else:
                lines.insert(end, value)
        new = "\n".join(lines) + "\n"
        if path.exists() and path.read_text() == new:
            continue
        backup = path.with_name(path.name + ".pre-korunix-luciole")
        if path.exists() and not backup.exists():
            shutil.copy2(path, backup)
        tmp = path.with_name(path.name + ".korunix-tmp")
        tmp.write_text(new)
        os.replace(tmp, path)
  '';
in lib.mkIf (equipo.habilitarUmbriel or false) {
  environment.systemPackages = [luciole];

  system.activationScripts.lucioleConfig.text = ''
    config_dir=${lib.escapeShellArg "${usuario.home}/.config/luciole"}
    install -d -m 0755 -o ${usuario.name} -g ${usuario.group} "$config_dir"
    # Luciole conserva sus preferencias personales en settings.toml.
    if [ -f "$config_dir/config.toml" ] && [ ! -e "$config_dir/config.toml.pre-korunix" ]; then
      cp -p "$config_dir/config.toml" "$config_dir/config.toml.pre-korunix"
    fi
    install -m 0644 -o ${usuario.name} -g ${usuario.group} ${configLuciole} "$config_dir/config.toml"
    ${pkgs.util-linux}/bin/runuser -u ${usuario.name} -- ${lib.getExe pkgs.python3} ${mimePorEscritorio} ${lib.escapeShellArg usuario.home}
  '';
}

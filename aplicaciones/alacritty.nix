{
  config,
  equipo,
  pkgs,
  ...
}: let
  usuario = config.users.users.${equipo.persona};
in {
  # Alacritty es la terminal única de Korunix en todos los escritorios.
  environment.sessionVariables.TERMINAL = "alacritty";

  # Estándar freedesktop para aplicaciones que solicitan abrir una terminal.
  xdg.terminal-exec = {
    enable = true;
    settings.default = ["Alacritty.desktop"];
  };

  # No se exponen terminales alternativas instaladas por los escritorios.
  services.xserver.excludePackages = [pkgs.xterm];
  services.xserver.desktopManager.xterm.enable = false;
  environment.gnome.excludePackages = [pkgs.gnome-console];
  environment.cinnamon.excludePackages = [pkgs.gnome-terminal];
  programs.gnome-terminal.enable = false;

  # Cinnamon consulta su propio esquema para Ctrl+Alt+T, Nemo y otras acciones.
  services.xserver.desktopManager.cinnamon.extraGSettingsOverrides = ''
    [org.cinnamon.desktop.default-applications.terminal]
    exec='alacritty'
    exec-arg='-e'
  '';

  # Noctalia administra la paleta de Alacritty. Korunix solo añade un margen
  # discreto para que el contenido de la terminal no quede pegado a los bordes.
  system.activationScripts.alacrittyPadding.text = ''
    config_dir=${usuario.home}/.config/alacritty
    config_file="$config_dir/alacritty.toml"

    install -d -m 0755 \
      -o ${usuario.name} \
      -g ${usuario.group} \
      "$config_dir"

    if [ ! -f "$config_file" ]; then
      cat > "$config_file" <<'EOF'
[window]
padding = { x = 8, y = 8 }
dynamic_padding = true
EOF
      chown ${usuario.name}:${usuario.group} "$config_file"
    elif ! grep -q '^\[window\]' "$config_file"; then
      cat >> "$config_file" <<'EOF'

[window]
padding = { x = 8, y = 8 }
dynamic_padding = true
EOF
      chown ${usuario.name}:${usuario.group} "$config_file"
    fi
  '';
}

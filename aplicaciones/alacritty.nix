{
  config,
  equipo,
  pkgs,
  ...
}: let
  usuario = config.users.users.${equipo.persona};
in {
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

{
  config,
  pkgs,
  ...
}: {
  # Noctalia administra la paleta de Alacritty. Korunix solo añade un margen
  # discreto para que el contenido de la terminal no quede pegado a los bordes.
  system.activationScripts.alacrittyPadding.text = ''
    config_dir=${config.users.users.koru.home}/.config/alacritty
    config_file="$config_dir/alacritty.toml"

    install -d -m 0755 \
      -o ${config.users.users.koru.name} \
      -g ${config.users.users.koru.group} \
      "$config_dir"

    if [ ! -f "$config_file" ]; then
      cat > "$config_file" <<'EOF'
[window]
padding = { x = 8, y = 8 }
dynamic_padding = true
EOF
      chown ${config.users.users.koru.name}:${config.users.users.koru.group} "$config_file"
    elif ! grep -q '^\[window\]' "$config_file"; then
      cat >> "$config_file" <<'EOF'

[window]
padding = { x = 8, y = 8 }
dynamic_padding = true
EOF
      chown ${config.users.users.koru.name}:${config.users.users.koru.group} "$config_file"
    fi
  '';
}

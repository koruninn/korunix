{
  config,
  equipo,
  pkgs,
  ...
}: let
  usuario = config.users.users.${equipo.persona};
in {
  # Niri, Umbriel y GNOME comparten qt5ct/qt6ct. La paleta noctalia.conf se
  # regenera desde el mismo fondo en cada sesión, así Qt deja de ser una isla.
  qt.enable = true;

  environment.systemPackages = [
    pkgs.libsForQt5.qt5ct
    pkgs.qt6Packages.qt6ct
  ];

  environment.variables.QT_QPA_PLATFORMTHEME = "qt5ct:qt6ct";

  system.activationScripts.noctaliaQtConfig.text = ''
    for version in qt5ct qt6ct; do
      config_dir=${usuario.home}/.config/$version
      colors_dir="$config_dir/colors"
      config_file="$config_dir/$version.conf"

      install -d -m 0755 \
        -o ${usuario.name} \
        -g ${usuario.group} \
        "$colors_dir"

      printf '%s\n' \
        '[Appearance]' \
        "color_scheme_path=${usuario.home}/.config/$version/colors/noctalia.conf" \
        'custom_palette=true' \
        'standard_dialogs=default' \
        > "$config_file"

      chown ${usuario.name}:${usuario.group} "$config_file"
      chmod 0644 "$config_file"
    done
  '';
}

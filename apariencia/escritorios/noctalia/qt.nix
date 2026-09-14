{
  config,
  equipo,
  pkgs,
  ...
}: let
  usuario = config.users.users.${equipo.persona};
in {
  # Noctalia tematiza Qt mediante qt5ct/qt6ct sin escribir kdeglobals.
  # Así las aplicaciones Qt de Niri y Umbriel quedan separadas de Plasma.
  qt.enable = true;

  environment.systemPackages = [
    pkgs.libsForQt5.qt5ct
    pkgs.qt6Packages.qt6ct
  ];

  # Los dos configuradores apuntan a la paleta dinámica que genera la plantilla
  # Qt de Noctalia. Plasma no usa estos archivos porque conserva su integración KDE.
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

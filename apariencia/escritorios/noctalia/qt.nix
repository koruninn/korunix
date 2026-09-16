{
  equipo,
  pkgs,
  ...
}: let
  prepararQt = pkgs.writeShellApplication {
    name = "korunix-qt-config";
    runtimeInputs = [pkgs.coreutils];
    text = ''
      config_home="''${XDG_CONFIG_HOME:-$HOME/.config}"

      for version in qt5ct qt6ct; do
        config_dir="$config_home/$version"
        colors_dir="$config_dir/colors"
        config_file="$config_dir/$version.conf"

        install -d -m 0755 "$colors_dir"
        printf '%s\n' \
          '[Appearance]' \
          "color_scheme_path=$colors_dir/noctalia.conf" \
          'custom_palette=true' \
          'standard_dialogs=default' \
          > "$config_file"
        chmod 0644 "$config_file"
      done
    '';
  };
in {
  # Umbriel y GNOME comparten qt5ct/qt6ct. La paleta noctalia.conf se regenera
  # desde el mismo fondo en cada sesión, así Qt deja de ser una isla.
  qt.enable = true;

  environment.systemPackages = [
    pkgs.libsForQt5.qt5ct
    pkgs.qt6Packages.qt6ct
  ];

  environment.variables.QT_QPA_PLATFORMTHEME = "qt5ct:qt6ct";

  systemd.user.services.korunix-qt-config = {
    description = "Prepara Qt para la paleta de Noctalia";
    wantedBy = ["default.target"];
    unitConfig.ConditionUser = equipo.persona;
    serviceConfig = {
      Type = "oneshot";
      ExecStart = "${prepararQt}/bin/korunix-qt-config";
    };
  };
}

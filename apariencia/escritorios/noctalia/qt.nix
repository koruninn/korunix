{
  config,
  pkgs,
  ...
}: let
  qt6ctConfig = pkgs.writeText "korunix-qt6ct.conf" ''
    [Appearance]
    color_scheme_path=${config.users.users.koru.home}/.config/qt6ct/colors/noctalia.conf
    custom_palette=true
    icon_theme=Hatter-Slate
    standard_dialogs=default
    style=Fusion
  '';
in {
  # Qt necesita exponer sus plugins en el perfil para que las aplicaciones
  # lanzadas desde Niri puedan cargar qt6ct.
  qt.enable = true;

  environment.systemPackages = [
    pkgs.qt6Packages.qt6ct
  ];

  # Dejamos qt6ct apuntando directamente a la paleta que genera Noctalia.
  # La variable que activa qt6ct se define solo dentro de la sesión de Niri,
  # por lo que Plasma mantiene su propia configuración.
  system.activationScripts.noctaliaQtConfig.text = ''
    install -d -m 0755 \
      -o ${config.users.users.koru.name} \
      -g ${config.users.users.koru.group} \
      ${config.users.users.koru.home}/.config/qt6ct

    install -m 0644 \
      -o ${config.users.users.koru.name} \
      -g ${config.users.users.koru.group} \
      ${qt6ctConfig} \
      ${config.users.users.koru.home}/.config/qt6ct/qt6ct.conf
  '';
}

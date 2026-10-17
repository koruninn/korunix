{
  config,
  equipo,
  pkgs,
  ...
}: let
  usuario = config.users.users.${equipo.persona};
in {
  system.activationScripts.korunixKWalletDisabled.text = ''
    config_file=${usuario.home}/.config/kwalletrc

    install -d -m 0755 \
      -o ${usuario.name} \
      -g ${usuario.group} \
      "$(dirname "$config_file")"

    ${pkgs.kdePackages.kconfig}/bin/kwriteconfig6 \
      --file "$config_file" \
      --group Wallet \
      --key Enabled false

    chown ${usuario.name}:${usuario.group} "$config_file"
  '';
}

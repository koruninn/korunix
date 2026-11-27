{
  config,
  equipo,
  pkgs,
  lib,
  ...
}: let
  usuario = config.users.users.koru;

  hooks = pkgs.writeText "noctalia-fondos-dia-noche.toml" ''
    [hooks]
    # GNOME y Noctalia comparten el mismo fondo.
    started = "korunix-wallpaper-sync session-start"
    wallpaper_changed = "korunix-wallpaper-sync from-noctalia \"$NOCTALIA_WALLPAPER_PATH\""
    theme_mode_changed = "korunix-wallpaper-sync noctalia-mode"
  '';
in lib.mkIf (equipo.habilitarUmbriel or false) {
  system.activationScripts.noctaliaFondosDiaNoche.text = ''
    config_dir=${usuario.home}/.config/noctalia

    install -d -m 0755 \
      -o ${usuario.name} \
      -g ${usuario.group} \
      "$config_dir"

    install -m 0644 \
      -o ${usuario.name} \
      -g ${usuario.group} \
      ${hooks} \
      "$config_dir/korunix-fondos-dia-noche.toml"
  '';
}

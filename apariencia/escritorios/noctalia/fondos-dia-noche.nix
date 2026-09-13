{
  config,
  equipo,
  pkgs,
  ...
}: let
  usuario = config.users.users.${equipo.persona};

  hooks = pkgs.writeText "noctalia-fondos-dia-noche.toml" ''
    [hooks]
    # Al terminar de iniciar Noctalia, escoge un fondo del conjunto que
    # corresponde al modo efectivo actual (claro u oscuro).
    started = "noctalia msg wallpaper-random"

    # theme.mode = auto usa la ubicación de Noctalia para resolver amanecer y
    # atardecer. Cuando cruza claro ↔ oscuro, wallpaper-random respeta
    # directory_light / directory_dark y cambia inmediatamente de conjunto.
    theme_mode_changed = "noctalia msg wallpaper-random"
  '';
in {
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

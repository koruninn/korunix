{
  config,
  equipo,
  pkgs,
  ...
}: let
  usuario = config.users.users.${equipo.persona};

  hooks = pkgs.writeText "noctalia-fondos-dia-noche.toml" ''
    [hooks]
    # GNOME y Umbriel usan el mismo fondo para que el cambio se vea igual en todo el equipo.
    # Noctalia ya cambia GTK3, GTK4 y el tema claro u oscuro cuando se cambia el fondo.
    # No añadimos otro proceso para hacer lo mismo, porque habría dos configuraciones
    # intentando cambiar GTK al mismo tiempo y algunas aplicaciones podrían quedar
    # con el tema anterior, especialmente las que usan libadwaita como Nautilus.
    started = "korunix-wallpaper-sync session-start"
    wallpaper_changed = "korunix-wallpaper-sync from-noctalia \"$NOCTALIA_WALLPAPER_PATH\""
  '';
in {
  system.activationScripts.noctaliaFondosDiaNoche.text = ''
    config_dir=${usuario.home}/.config/noctalia

    install -d -m 0755 \
      -o ${usuario.name} \
      -g ${usuario.group} \
      "$config_dir"

    # Dejamos este archivo listo para que Noctalia y GNOME usen el mismo fondo.
    install -m 0644 \
      -o ${usuario.name} \
      -g ${usuario.group} \
      ${hooks} \
      "$config_dir/korunix-fondos-dia-noche.toml"
  '';
}

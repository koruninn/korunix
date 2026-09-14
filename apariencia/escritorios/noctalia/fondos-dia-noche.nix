{
  config,
  equipo,
  pkgs,
  ...
}: let
  usuario = config.users.users.${equipo.persona};

  hooks = pkgs.writeText "noctalia-fondos-dia-noche.toml" ''
    [hooks]
    # Korunix mantiene un único estado de fondo para GNOME y Umbriel.
    # Noctalia ya reaplica GTK3/GTK4 y sincroniza color-scheme por sí mismo
    # cuando cambia entre claro y oscuro. No añadimos un segundo hook de modo:
    # competiría con el post_hook GTK de Noctalia y puede dejar aplicaciones
    # libadwaita como Nautilus una generación por detrás.
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

    # Este archivo pertenecía al puente Plasma↔Noctalia anterior. Si queda en
    # HOME definiría un segundo [hooks].started y competiría con el coordinador
    # compartido actual.
    rm -f "$config_dir/gtk-session.toml"

    install -m 0644 \
      -o ${usuario.name} \
      -g ${usuario.group} \
      ${hooks} \
      "$config_dir/korunix-fondos-dia-noche.toml"
  '';
}

{
  config,
  equipo,
  pkgs,
  ...
}: let
  usuario = config.users.users.${equipo.persona};
  noctaliaPackage = config.programs.noctalia.package;

  # Noctalia ya aplica sus plantillas antes de disparar theme_mode_changed.
  # Nuestro coordinador de fondos corre después y puede volver a escribir el
  # color-scheme; por eso fijamos explícitamente el modo resuelto de Noctalia
  # antes y después de sincronizar el fondo. Así libadwaita (Nautilus) recibe
  # siempre el último valor correcto y cambia en vivo también bajo Umbriel.
  modeSync = pkgs.writeShellApplication {
    name = "korunix-noctalia-mode-sync";
    runtimeInputs = [
      pkgs.bash
      pkgs.glib
    ];
    text = ''
      mode="''${1:-}"
      case "$mode" in
        dark|light) ;;
        *) exit 0 ;;
      esac

      sync_appearance() {
        ${pkgs.bash}/bin/bash \
          ${noctaliaPackage}/share/noctalia/assets/templates/gtk/apply.sh \
          --appearance-only "$mode" \
          >/dev/null 2>&1 || true
      }

      sync_appearance
      korunix-wallpaper-sync mode-change "$mode" >/dev/null 2>&1 || true
      sync_appearance
    '';
  };

  hooks = pkgs.writeText "noctalia-fondos-dia-noche.toml" ''
    [hooks]
    # Korunix mantiene un único estado de fondo para GNOME y Umbriel.
    # Noctalia sigue siendo quien muestra el fondo en Umbriel, pero cada cambio
    # se refleja inmediatamente en el estado compartido y en GNOME.
    started = "korunix-wallpaper-sync session-start"
    wallpaper_changed = "korunix-wallpaper-sync from-noctalia \"$NOCTALIA_WALLPAPER_PATH\""
    theme_mode_changed = "${modeSync}/bin/korunix-noctalia-mode-sync \"$NOCTALIA_THEME_MODE\""
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

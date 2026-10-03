{
  config,
  equipo,
  lib,
  pkgs,
  ...
}: let
  usuario = config.users.users.${equipo.persona};
  plugins = import ./plugins.nix;
  toml = pkgs.formats.toml {};
  ajustes = import ./ajustes.nix {inherit equipo lib plugins;};
  noctaliaConfig = toml.generate "noctalia-config.toml" ajustes;

  # Aquí se arma un pequeño programa que toma la configuración que Korunix
  # define para Noctalia y la deja en la carpeta del usuario.
  # Además, limpia la parte que Korunix controla para no mezclar ajustes
  # manuales del usuario con los ajustes de sistema.
  noctaliaConfigSync = pkgs.writeShellApplication {
    name = "korunix-noctalia-config-sync";
    runtimeInputs = [pkgs.coreutils pkgs.python3 pkgs.xdg-user-dirs];
    text = ''
      home=${lib.escapeShellArg usuario.home}
      owner=${lib.escapeShellArg usuario.name}
      group=${lib.escapeShellArg usuario.group}
      config_dir="$home/.config/noctalia"
      settings_file="$home/.local/state/noctalia/settings.toml"

      # Creamos la carpeta de configuración de Noctalia para la persona si no existe.
      install -d -m 0755 -o "$owner" -g "$group" "$config_dir"

      # Si Noctalia ya tiene un archivo con ajustes guardados, aquí se quitan
      # solo los bloques que Korunix administra. Así la persona puede seguir
      # teniendo cambios propios, pero no se mezclan con los de la configuración del sistema.
      if [ -f "$settings_file" ]; then
        tmp_file="$settings_file.korunix-tmp"
        skip_managed=false
        while IFS= read -r line || [ -n "$line" ]; do
          case "$line" in
            "[accessibility]"|\
            "[shell]"|"[shell."*|\
            "[location]"|"[location."*|\
            "[control_center]"|"[control_center."*|"[[control_center."*|\
            "[bar]"|"[bar."*|"[[bar."*|\
            "[dock]"|"[dock."*|\
            "[osd]"|"[osd."*|\
            "[lockscreen]"|"[lockscreen."*|\
            "[calendar]"|"[calendar."*|\
            "[plugins]"|"[plugins."*|\
            "[theme]"|"[theme."*|\
            "[plugin_settings.\"noctalia/notes\"]"|\
            "[widget.fecha]"|"[widget.weather]"|"[widget.media]"|"[widget.cat]"|\
            "[widget.bongo_cat]"|"[widget.calculator]"|"[widget.pomodoro_timer]"|\
            "[widget.notes]"|"[widget.udiskie_manager]"|"[widget.tray]"|\
            "[widget.network]"|"[widget.bluetooth]"|"[widget.lock_keys]"|\
            "[widget.volume_input]"|"[widget.temperatura]"|"[widget.umbriel_displays]"|\
            "[widget.umbriel_companion]"|"[widget.speedtest_meter]"|\
            "[widget.phone_connect]"|"[widget.printers]"|"[widget.red_rx]"|\
            "[widget.red_tx]"|"[widget.privacy]"|"[widget.screen_recorder]")
              skip_managed=true
              continue
              ;;
            "["*)
              skip_managed=false
              ;;
          esac
          if [ "$skip_managed" = false ]; then
            printf '%s\n' "$line"
          fi
        done < "$settings_file" > "$tmp_file"
        install -m 0644 -o "$owner" -g "$group" "$tmp_file" "$settings_file"
        rm -f "$tmp_file"
      fi

      # Aquí se escribe la configuración principal que Korunix quiere dejar en Noctalia.
      install -m 0644 -o "$owner" -g "$group" ${noctaliaConfig} "$config_dir/config.toml"

      # Ajustamos la carpeta de notas para que Noctalia use la carpeta de documentos
      # de la persona, sin depender de una ruta fija o rara del sistema.
      documents_dir="$(HOME="$home" xdg-user-dir DOCUMENTS)"
      if [ -n "$documents_dir" ]; then
        notes_dir="$documents_dir/Notes"
        notes_value="$(python3 -c 'import json, sys; print(json.dumps(sys.argv[1], ensure_ascii=False))' "$notes_dir")"
        xdg_file="$config_dir/zz-korunix-xdg.toml"
        xdg_tmp="$xdg_file.korunix-tmp"
        {
          printf '%s\n' '[plugin_settings."noctalia/notes"]'
          printf 'notes_dir = %s\n' "$notes_value"
        } > "$xdg_tmp"
        install -m 0644 -o "$owner" -g "$group" "$xdg_tmp" "$xdg_file"
        rm -f "$xdg_tmp"
      fi
    '';
  };
in {
  programs.noctalia.enable = true;
  environment.etc."noctalia/config.toml".source = noctaliaConfig;

  # Este paso se ejecuta al activar el sistema para dejar la configuración
  # de Noctalia en la carpeta del usuario y que quede lista al iniciar sesión.
  # Si algo falla aquí, la construcción se corta antes de aplicar cambios al equipo.
  system.activationScripts.noctaliaConfig.text = ''
    ${noctaliaConfigSync}/bin/korunix-noctalia-config-sync
  '';
}

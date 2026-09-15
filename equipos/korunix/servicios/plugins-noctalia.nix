{
  equipo,
  pkgs,
  ...
}: let
  usuario = equipo.persona;
  home = "/home/${usuario}";
in {
  # Herramientas externas que usan los plugins de Noctalia que sí conserva Korunix.
  environment.systemPackages = [
    pkgs.coreutils
    pkgs.cups
    pkgs.evtest
    pkgs.ffmpeg
    pkgs.gpu-screen-recorder
    pkgs.kdePackages.kdeconnect-kde
    pkgs.speedtest-cli
    pkgs.sshfs
    pkgs.udiskie
    pkgs.xdg-utils
    pkgs.yt-dlp
  ];

  # Bongo Cat necesita leer los dispositivos de entrada.
  users.users.${usuario}.extraGroups = ["input"];

  # La lista de plugins pertenece a Korunix; sus ajustes individuales siguen siendo editables.
  system.activationScripts.noctaliaPlugins = {
    deps = ["noctaliaConfig"];
    text = ''
      settings_file=${home}/.local/state/noctalia/settings.toml
      if [ -f "$settings_file" ]; then
        tmp_file="$settings_file.korunix-plugins-tmp"
        skip_plugins=false
        while IFS= read -r line || [ -n "$line" ]; do
          case "$line" in
            "[plugins]")
              skip_plugins=true
              continue
              ;;
            "["*)
              skip_plugins=false
              ;;
          esac
          if [ "$skip_plugins" = false ]; then
            printf '%s\n' "$line"
          fi
        done < "$settings_file" > "$tmp_file"
        install -m 0644 -o ${usuario} -g users "$tmp_file" "$settings_file"
        rm -f "$tmp_file"
      fi
    '';
  };
}

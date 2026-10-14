{
  config,
  equipo,
  inputs,
  lib,
  pkgs,
  ...
}: let
  usuario = config.users.users.${equipo.persona};
in {
  nixpkgs.overlays = [
    inputs.millennium.overlays.default
  ];

  programs.steam = {
    enable = true;
    package = pkgs.steam;
    remotePlay.openFirewall = true;
    dedicatedServer.openFirewall = true;
  };

  # Soporte general para mandos: Xbox por Bluetooth, PlayStation, Nintendo,
  # 8BitDo, PowerA y otros dispositivos cubiertos por reglas udev comunes.
  hardware.steam-hardware.enable = true;
  hardware.uinput.enable = true;
  hardware.xpadneo.enable = true;
  services.udev.packages = [ pkgs.game-devices-udev-rules ];

  programs.gamemode.enable = true;

  # Steam espera que ~/.steam/steam sea un enlace a ~/.local/share/Steam.
  # Algunas instalaciones antiguas dejan allí una carpeta real, que bloquea
  # el arranque. La movemos a un respaldo; nunca la borramos ni sobrescribimos
  # la instalación principal. Si la ruta no es inequívocamente segura, no tocamos nada.
  system.activationScripts.korunixSteamPath.text = ''
    home=${lib.escapeShellArg usuario.home}
    owner=${lib.escapeShellArg usuario.name}
    group=${lib.escapeShellArg usuario.group}

    steam_dir="$home/.steam"
    steam_link="$steam_dir/steam"
    steam_data="$home/.local/share/Steam"

    install -d -m 0755 -o "$owner" -g "$group" \
      "$steam_dir" "$home/.local/share"

    if [ -L "$steam_link" ]; then
      # Un enlace existente se deja intacto; no se cambia una ruta elegida por el usuario.
      :
    elif [ -d "$steam_link" ]; then
      if [ -d "$steam_data" ] && [ ! -L "$steam_data" ]; then
        backup="$steam_dir/steam.korunix-backup-$(date +%Y%m%d-%H%M%S)-$$"
        mv -- "$steam_link" "$backup"
      elif [ ! -e "$steam_data" ] && [ ! -L "$steam_data" ]; then
        mv -- "$steam_link" "$steam_data"
      else
        echo "Korunix: no se reparó la ruta de Steam; el destino existe o es un enlace."
      fi
    fi

    # Después de apartar la carpeta conflictiva, crea el enlace en esta misma activación.
    if [ ! -e "$steam_link" ] && [ ! -L "$steam_link" ]; then
      if [ -d "$steam_data" ] && [ ! -L "$steam_data" ]; then
        ln -s "$steam_data" "$steam_link"
        chown -h "$owner:$group" "$steam_link"
      elif [ ! -e "$steam_data" ] && [ ! -L "$steam_data" ]; then
        # Steam aún no ha creado su directorio de datos; el enlace puede apuntar a la ruta futura.
        ln -s "$steam_data" "$steam_link"
        chown -h "$owner:$group" "$steam_link"
      else
        echo "Korunix: no se creó el enlace de Steam porque el destino no es seguro."
      fi
    fi
  '';
}

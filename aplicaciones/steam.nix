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
    package = pkgs.millennium-steam;
    gamescopeSession.enable = lib.mkForce false;
    remotePlay.openFirewall = true;
    dedicatedServer.openFirewall = true;
  };

  programs.gamescope.enable = true;

  hardware.steam-hardware.enable = true;
  hardware.uinput.enable = true;
  hardware.xpadneo.enable = true;
  services.udev.packages = [pkgs.game-devices-udev-rules];

  programs.gamemode.enable = true;

  # ROG Ally: soporte para controles y botones especiales.
  services.handheld-daemon = {
    enable = true;
    user = equipo.persona;
    ui.enable = true;
    adjustor = {
      enable = true;
      loadAcpiCallModule = true;
    };
  };

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

    if [ ! -e "$steam_link" ] && [ ! -L "$steam_link" ]; then
      if [ -d "$steam_data" ] && [ ! -L "$steam_data" ]; then
        ln -s "$steam_data" "$steam_link"
        chown -h "$owner:$group" "$steam_link"
      elif [ ! -e "$steam_data" ] && [ ! -L "$steam_data" ]; then
        ln -s "$steam_data" "$steam_link"
        chown -h "$owner:$group" "$steam_link"
      else
        echo "Korunix: no se creó el enlace de Steam porque el destino no es seguro."
      fi
    fi
  '';
}

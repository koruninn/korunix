{
  equipo,
  lib,
  pkgs,
  ...
}: let
  plugins = import ../../../apariencia/escritorios/noctalia/plugins.nix;

  paquetes = {
    cups = pkgs.cups;
    "gpu-screen-recorder" = pkgs.gpu-screen-recorder;
    kdeconnect = pkgs.kdePackages.kdeconnect-kde;
    "speedtest-cli" = pkgs.speedtest-cli;
    sshfs = pkgs.sshfs;
    udiskie = pkgs.udiskie;
    "xdg-utils" = pkgs.xdg-utils;
    "yt-dlp" = pkgs.yt-dlp;
  };

  paquetesPlugins = map (nombre: paquetes.${nombre}) (lib.unique plugins.dependencias);

  limpiarPlugins = pkgs.writeShellApplication {
    name = "korunix-noctalia-plugin-cleanup";
    runtimeInputs = [pkgs.coreutils];
    text = ''
      materialized_root="''${XDG_STATE_HOME:-$HOME/.local/state}/noctalia/plugins/materialized"
      [ -d "$materialized_root" ] || exit 0

      for plugin_dir in ${lib.concatMapStringsSep " " lib.escapeShellArg plugins.descartados}; do
        rm -rf -- "''${materialized_root:?}/$plugin_dir"
      done
    '';
  };
in {
  # Herramientas generales del equipo. FFmpeg permanece disponible aunque no
  # haya ningún plugin que lo pida.
  environment.systemPackages = [
    pkgs.coreutils
    pkgs.evtest
    pkgs.ffmpeg
  ] ++ paquetesPlugins;

  # La limpieza del estado de plugins pertenece a la sesión de la persona, no
  # a una activación ejecutada como root.
  systemd.user.services.korunix-noctalia-plugin-cleanup = {
    description = "Limpia plugins descartados de Noctalia";
    wantedBy = ["default.target"];
    unitConfig.ConditionUser = equipo.persona;
    serviceConfig = {
      Type = "oneshot";
      ExecStart = "${limpiarPlugins}/bin/korunix-noctalia-plugin-cleanup";
    };
  };
}

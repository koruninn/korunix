{
  config,
  equipo,
  lib,
  pkgs,
  ...
}: let
  usuario = config.users.users.${equipo.persona};
  plugins = import ../../../apariencia/escritorios/noctalia/plugins.nix;
  limpiarPlugins = pkgs.writeShellApplication {
    name = "korunix-noctalia-plugin-cleanup";
    runtimeInputs = [pkgs.coreutils];
    text = ''
      materialized_root=${lib.escapeShellArg "${usuario.home}/.local/state/noctalia/plugins/materialized"}
      for plugin_dir in ${lib.concatMapStringsSep " " lib.escapeShellArg plugins.descartados}; do
        rm -rf -- "$materialized_root/$plugin_dir"
      done
    '';
  };
in {
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

  # input vive en personas/default.nix: Bongo Cat y los mandos de Xbox lo usan.
  system.activationScripts.noctaliaPlugins = {
    deps = ["noctaliaConfig"];
    text = ''
      ${limpiarPlugins}/bin/korunix-noctalia-plugin-cleanup
    '';
  };
}

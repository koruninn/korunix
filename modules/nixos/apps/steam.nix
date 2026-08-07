{
  config,
  pkgs,
  inputs,
  lib,
  ...
}: {
  programs.steam = {
    enable = true;
    remotePlay.openFirewall = true;
    dedicatedServer.openFirewall = true;
    package = pkgs.millennium-steam;
  };

  programs.gamemode.enable = true;
}

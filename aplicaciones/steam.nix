{
  inputs,
  pkgs,
  ...
}: {
  # Millennium publica un overlay que sustituye Steam por su paquete integrado.
  nixpkgs.overlays = [
    inputs.millennium.overlays.default
  ];

  programs.steam = {
    enable = true;
    package = pkgs.millennium-steam;
    remotePlay.openFirewall = true;
    dedicatedServer.openFirewall = true;
  };

  programs.gamemode.enable = true;
}

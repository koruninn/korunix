{
  inputs,
  pkgs,
  ...
}: {
  nixpkgs.overlays = [
    inputs.millennium.overlays.default
  ];

  programs.steam = {
    enable = true;
    package = pkgs.millennium-steam;
    remotePlay.openFirewall = true;
    dedicatedServer.openFirewall = true;
  };

  # Reglas udev para mandos y hardware de juego. Complementa el grupo input,
  # que Korunix conserva para Bongo Cat y los mandos de Xbox.
  hardware.steam-hardware.enable = true;
  programs.gamemode.enable = true;
}

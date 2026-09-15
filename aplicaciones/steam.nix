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

  # Soporte general para mandos: Xbox por Bluetooth, PlayStation, Nintendo,
  # 8BitDo, PowerA y otros dispositivos cubiertos por reglas udev comunes.
  hardware.steam-hardware.enable = true;
  hardware.uinput.enable = true;
  hardware.xpadneo.enable = true;
  services.udev.packages = [pkgs.game-devices-udev-rules];

  programs.gamemode.enable = true;
}

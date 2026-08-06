{
  config,
  pkgs,
  ...
}: {
  imports = [
    ./audio.nix
    ./bootloader.nix
    ./cachix.nix
    ./desktop.nix
    ./networking.nix
    ./power.nix
    ./printing.nix
    ./touchpad.nix
  ];
}

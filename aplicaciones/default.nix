{
  config,
  pkgs,
  ...
}: {
  imports = [
    ./aagl.nix
    ./alacritty.nix
    ./fastfetch.nix
    ./fetch.nix
    ./figma.nix
    ./fish.nix
    ./flatpak.nix
    ./localsend.nix
    ./obs.nix
    ./paquetes.nix
    ./spicetify.nix
    ./steam.nix
  ];
}

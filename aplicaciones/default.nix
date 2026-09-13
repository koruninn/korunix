{
  config,
  pkgs,
  ...
}: {
  imports = [
    ./aagl.nix
    ./alacritty.nix
    ./fastfetch.nix
    ./figma.nix
    ./fish.nix
    ./flatpak.nix
    ./localsend.nix
    ./nautilus.nix
    ./obs.nix
    ./paquetes.nix
    ./predeterminadas.nix
    ./spicetify.nix
    ./steam.nix
  ];
}

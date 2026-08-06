{
  config,
  pkgs,
  inputs,
  ...
}: {
  imports = [
    ./aagl.nix
    ./fish.nix
    ./firefox.nix
    ./flatpak.nix
    ./localsend.nix
    ./obs.nix
    ./packages.nix
    ./steam.nix
  ];

  _module.args = {inherit inputs;};
}

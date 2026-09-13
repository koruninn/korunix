{
  config,
  pkgs,
  ...
}: {
  imports = [
    ./noctalia.nix
    ./niri.nix
    ./qt.nix
  ];
}

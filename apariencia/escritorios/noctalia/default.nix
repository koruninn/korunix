{
  config,
  pkgs,
  ...
}: {
  imports = [
    ./noctalia.nix
    ./niri.nix
    ./umbriel.nix
    ./qt.nix
  ];
}

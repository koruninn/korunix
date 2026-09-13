{
  config,
  pkgs,
  ...
}: {
  imports = [
    ./noctalia.nix
    ./fondos-dia-noche.nix
    ./niri.nix
    ./umbriel.nix
    ./qt.nix
  ];
}

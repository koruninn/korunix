{
  config,
  pkgs,
  ...
}: {
  imports = [
    ./noctalia.nix
    ./fondos-dia-noche.nix
    ./portales.nix
    ./niri.nix
    ./umbriel.nix
    ./hyprland.nix
    ./qt.nix
  ];
}

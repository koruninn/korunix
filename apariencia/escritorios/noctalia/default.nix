{
  inputs,
  ...
}: {
  imports = [
    inputs.noctalia.nixosModules.default
    ./noctalia.nix
    ./fondos-dia-noche.nix
    ./monitor.nix
    ./portales.nix
    ./niri.nix
    ./umbriel.nix
    ./qt.nix
  ];
}

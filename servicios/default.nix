{...}: {
  imports = [
    ./arranque.nix
    ./almacenamiento.nix
    ./sesiones.nix
    ./decky.nix
    ./umbriel.nix
    ./mandos.nix
    ./gamescope.nix
    ./keyring.nix
    ./microfono.nix
    ./mprisence.nix
    ./redes.nix
  ];

  services.fprintd.enable = false;
}

{ inputs, ... }: {
  imports = [
    ./configuracion.nix
    ./hardware.nix
    ./personas
    ./servicios
    ../../apariencia
    inputs.noctalia.nixosModules.default
  ];
}

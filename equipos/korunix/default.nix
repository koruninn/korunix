{ inputs, ... }: {
  imports = [
    ./configuracion.nix
    ./hardware.nix
    ./personas
    ./servicios
    ../../aplicaciones
    ../../apariencia
    inputs.noctalia.nixosModules.default
  ];
}

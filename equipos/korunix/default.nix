{ inputs, ... }: {
  imports = [
    ./configuracion.nix
    ./hardware.nix
    ./personas
    ./servicios
    ../../aplicaciones
    ../../apariencia
    inputs.aagl.nixosModules.default
    inputs.noctalia.nixosModules.default
  ];
}

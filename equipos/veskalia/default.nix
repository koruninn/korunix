{inputs, ...}: {
  imports = [
    inputs.nixos-hardware.nixosModules.asus-ally-rc71l
    ./configuracion.nix
    ./hardware.nix
    ./personas
    ./servicios
    ./apariencia.nix
    ../../aplicaciones
  ];
}

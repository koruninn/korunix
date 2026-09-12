{ lib, modulesPath, ... }:

{
  # Reemplazar este archivo con la configuración generada para el hardware.
  imports = [
    (modulesPath + "/installer/scan/not-detected.nix")
  ];

  nixpkgs.hostPlatform = lib.mkDefault "x86_64-linux";
}

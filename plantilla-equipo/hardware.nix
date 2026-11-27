{
  equipo,
  lib,
  modulesPath,
  ...
}: {
  # Sustituir por la configuración de hardware que genera NixOS.
  imports = [
    (modulesPath + "/installer/scan/not-detected.nix")
  ];

  # Usar la arquitectura indicada en equipo.nix.
  nixpkgs.hostPlatform = lib.mkDefault equipo.arquitectura;
}

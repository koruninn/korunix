{
  equipo,
  lib,
  modulesPath,
  ...
}: {
  # Cambia este archivo por la ajustes que NixOS genera para tu hardware.
  imports = [
    (modulesPath + "/installer/scan/not-detected.nix")
  ];

  # Mientras se use la plantilla, la plataforma sigue la arquitectura declarada
  # en equipo.nix para no mantener el mismo dato en dos sitios.
  nixpkgs.hostPlatform = lib.mkDefault equipo.arquitectura;
}

{
  equipo,
  lib,
  modulesPath,
  ...
}: {
  # Reemplazar este archivo con la configuración generada para el hardware.
  imports = [
    (modulesPath + "/installer/scan/not-detected.nix")
  ];

  # Mientras se use la plantilla, la plataforma sigue la arquitectura declarada
  # en equipo.nix para no mantener el mismo dato en dos sitios.
  nixpkgs.hostPlatform = lib.mkDefault equipo.arquitectura;
}

{
  config,
  pkgs,
  inputs,
  ...
}: {
  imports = [
    inputs.noctalia.homeModules.default
    ./ajustes/barra.nix
    ./ajustes/dock.nix
    ./ajustes/bloqueo.nix
    ./ajustes/osd.nix
    ./ajustes/servicios.nix
    ./ajustes/shell.nix
    ./ajustes/tema.nix
    ./ajustes/fondo.nix
  ];

  programs.noctalia.enable = true;
}

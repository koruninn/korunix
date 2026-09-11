{
  config,
  pkgs,
  ...
}: {
  imports = [    
    ./arranque.nix
    ./audio.nix
    ./energia.nix
    ./redes.nix
  ];
}

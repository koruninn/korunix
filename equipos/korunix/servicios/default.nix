{
  config,
  pkgs,
  ...
}: {
  imports = [    
    ./arranque.nix
    ./audio.nix
    ./cachix.nix    
    ./energia.nix
    ./redes.nix
    ./touchpad.nix
  ];
}

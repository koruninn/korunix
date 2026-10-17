{...}: {
  imports = [
    ./arranque.nix
    ../../../apariencia/escritorios/noctalia
    ./sysboard.nix
    ./steam-deck.nix
    ./cachix.nix
    ./keyring.nix
    ./mprisence.nix
    ./plugins-noctalia.nix
    ./redes.nix
  ];
}

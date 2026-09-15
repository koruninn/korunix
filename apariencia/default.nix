{
  config,
  pkgs,
  ...
}: {
  imports = [
    ./hatter.nix
    ./cursor.nix
    ./gtk.nix
    ./tipografia.nix
    ./fondos.nix
    ./escritorios
    ./greeter
  ];
}

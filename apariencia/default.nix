{
  config,
  pkgs,
  ...
}: {
  imports = [
    ./hatter.nix
    ./cursor.nix
    ./gtk.nix
    ./fondos.nix
    ./escritorios
    ./greeter
  ];
}

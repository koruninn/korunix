{
  config,
  pkgs,
  ...
}: {
  imports = [
    ./hatter.nix
    ./cursor.nix
    ./gtk.nix
    ./escritorios
    ./greeter
  ];
}

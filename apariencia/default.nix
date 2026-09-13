{
  config,
  pkgs,
  ...
}: {
  imports = [
    ./hatter.nix
    ./gtk.nix
    ./escritorios
    ./greeter
  ];
}

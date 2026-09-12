{
  config,
  pkgs,
  ...
}: {
  imports = [
    ./gtk.nix
    ./escritorios
    ./greeter
  ];
}

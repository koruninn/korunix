{
  config,
  pkgs,
  ...
}: {
  imports = [
    ./hatter.nix
    ./escritorios
    ./greeter
  ];
}

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
    ./perfil.nix
    ./fondos.nix
    ./escritorios
    ./greeter
  ];
}

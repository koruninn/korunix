{
  equipo,
  lib,
  pkgs,
  ...
}: let
  tieneFoto = equipo ? foto;
  cuenta = pkgs.writeText "korunix-accountsservice-${equipo.persona}" ''
    [User]
    Icon=/var/lib/AccountsService/icons/${equipo.persona}
    SystemAccount=false
  '';
in {
  # La foto pertenece a la persona del equipo, no a GNOME ni a Noctalia.
  # AccountsService permite que ambos escritorios y el greeter lean la misma.
  systemd.tmpfiles.rules = lib.optionals tieneFoto [
    "d /var/lib/AccountsService 0755 root root -"
    "d /var/lib/AccountsService/icons 0755 root root -"
    "d /var/lib/AccountsService/users 0700 root root -"
    "L+ /var/lib/AccountsService/icons/${equipo.persona} - - - - ${equipo.foto}"
    "L+ /var/lib/AccountsService/users/${equipo.persona} - - - - ${cuenta}"
  ];
}

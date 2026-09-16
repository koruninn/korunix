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

  instalarPerfil = pkgs.writeShellApplication {
    name = "korunix-profile-picture";
    runtimeInputs = [pkgs.coreutils];
    text = ''
      install -d -m 0755 /var/lib/AccountsService/icons
      install -d -m 0700 /var/lib/AccountsService/users
      install -m 0644 ${equipo.foto} /var/lib/AccountsService/icons/${equipo.persona}
      install -m 0600 ${cuenta} /var/lib/AccountsService/users/${equipo.persona}
    '';
  };
in {
  # La foto pertenece a la persona del equipo, no a GNOME ni a Noctalia.
  # Se copia a AccountsService para que GNOME y el greeter compartan la misma
  # sin convertir sus archivos mutables en enlaces al almacén de Nix.
  system.activationScripts.korunixProfile = lib.mkIf tieneFoto {
    text = ''
      ${instalarPerfil}/bin/korunix-profile-picture
    '';
  };
}

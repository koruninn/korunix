{config, lib, pkgs, ...}: let
  usuario = config.users.users.koru.name;
  fuentes = (import ./appimages.nix).gearlever;
  archivoFuentes = pkgs.writeText "gearlever-fuentes.json" (builtins.toJSON fuentes);
  registrar = pkgs.writeShellScript "korunix-gearlever-fuentes" ''
    exec ${lib.getExe pkgs.python3} ${./gearlever-fuentes.py} ${archivoFuentes} ${lib.getExe pkgs.gearlever}
  '';
in {
  environment.systemPackages = [pkgs.gearlever];

  systemd.user.services.korunix-gearlever-fuentes = lib.mkIf (fuentes != []) {
    description = "Registrar las fuentes de actualización de Gear Lever";
    wantedBy = ["graphical-session.target"];
    partOf = ["graphical-session.target"];
    unitConfig.ConditionUser = usuario;
    serviceConfig = {
      Type = "oneshot";
      ExecStart = registrar;
    };
  };
}

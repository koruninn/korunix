{pkgs}: let
  configuracion = builtins.fromTOML (builtins.readFile ./configuracion.toml);
  nombres = configuracion.aplicaciones.instaladas or [];

  resolver = nombre:
    if builtins.hasAttr nombre pkgs
    then let
      paquete = builtins.getAttr nombre pkgs;
    in {
      elegida = nombre;
      nombre =
        if paquete ? pname && paquete.pname != null
        then builtins.toString paquete.pname
        else nombre;
      version =
        if paquete ? version && paquete.version != null
        then builtins.toString paquete.version
        else "";
      valor = paquete;
    }
    else
      throw ''
        No encontré «${nombre}» en Nixpkgs.
        Revisa el nombre en configuracion.toml.
      '';

  resueltas = map resolver nombres;
in {
  aplicaciones = map (aplicacion: aplicacion.valor) resueltas;

  plan = map
    (aplicacion:
      builtins.removeAttrs aplicacion ["valor"])
    resueltas;
}

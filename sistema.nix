{pkgs}: let
  configuracion = builtins.fromTOML (builtins.readFile ./configuracion.toml);
  nombres = configuracion.aplicaciones.instaladas or [];

  buscarAplicacion = nombre:
    if builtins.hasAttr nombre pkgs
    then builtins.getAttr nombre pkgs
    else
      throw ''
        No encontré «${nombre}» en Nixpkgs.
        Revisa el nombre en configuracion.toml.
      '';
in {
  aplicaciones = map buscarAplicacion nombres;
}

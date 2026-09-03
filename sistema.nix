{pkgs}: let
  # Leemos exactamente las mismas decisiones que ve la persona.
  configuracion = builtins.fromTOML (builtins.readFile ./configuracion.toml);

  nombres = configuracion.aplicaciones.instaladas or [];

  buscarAplicacion = nombre:
    if builtins.hasAttr nombre pkgs
    then builtins.getAttr nombre pkgs
    else
      throw ''
        Korunix no encontró la aplicación «${nombre}» en Nixpkgs.
        Revisa el nombre en configuracion.toml.
      '';
in {
  # Una lista humana se convierte aquí en paquetes reales de NixOS.
  aplicaciones = map buscarAplicacion nombres;
}

{
  description = "Korunix";

  inputs = {
    nixpkgs-estable.url = "github:NixOS/nixpkgs/nixos-26.05";
    nixpkgs-inestable.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = {
    nixpkgs-estable,
    nixpkgs-inestable,
    ...
  }: let
    # La persona cambia una sola palabra en configuracion.toml.
    configuracion = builtins.fromTOML (builtins.readFile ./configuracion.toml);
    canal = configuracion.canal or "estable";

    nixpkgs =
      if canal == "estable"
      then nixpkgs-estable
      else if canal == "inestable"
      then nixpkgs-inestable
      else
        throw ''
          Korunix no conoce el canal «${canal}».
          Usa "estable" o "inestable" en configuracion.toml.
        '';

    sistema = "x86_64-linux";

    pkgs = import nixpkgs {
      system = sistema;
      config.allowUnfree = true;
    };

    resultado = import ./sistema.nix {inherit pkgs;};
  in {
    # Por ahora este resultado solo demuestra el modelo Lego:
    # nombres humanos arriba → paquetes reales abajo.
    packages.${sistema}.default = pkgs.buildEnv {
      name = "korunix-aplicaciones";
      paths = resultado.aplicaciones;
    };

    formatter.${sistema} = pkgs.alejandra;
  };
}

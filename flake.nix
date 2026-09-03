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

    # Rust revisa las decisiones humanas. Nix empaqueta el mismo programa.
    programa = pkgs.rustPlatform.buildRustPackage {
      pname = "korunix";
      version = "0.1.0";
      src = ./.;

      cargoLock = {
        lockFile = ./Cargo.lock;
      };
    };
  in {
    packages.${sistema} = {
      default = programa;
      korunix = programa;

      # Esto sigue demostrando que los nombres humanos se convierten en paquetes.
      aplicaciones = pkgs.buildEnv {
        name = "korunix-aplicaciones";
        paths = resultado.aplicaciones;
      };
    };

    formatter.${sistema} = pkgs.alejandra;
  };
}

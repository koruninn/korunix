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
    configuracion = builtins.fromTOML (builtins.readFile ./configuracion.toml);
    nombre = configuracion.nombre or "nixos";
    canal = configuracion.canal or "estable";
    personas = configuracion.personas or [];
    escritorio = (configuracion.escritorio or {}).principal or "niri";
    noctaliaActivo = escritorio == "niri" || escritorio == "hyprland";

    nixpkgs =
      if canal == "estable"
      then nixpkgs-estable
      else if canal == "inestable"
      then nixpkgs-inestable
      else
        throw ''
          No conozco el canal «${canal}».
          Pon "estable" o "inestable" en configuracion.toml.
        '';

    sistema = "x86_64-linux";

    pkgs = import nixpkgs {
      system = sistema;

      # Algunas aplicaciones necesitan esto para poder instalarse.
      config.allowUnfree = true;
    };

    # NixOS 26.05 todavía no trae Noctalia. En ese canal solo esta pieza
    # concreta viene del Nixpkgs inestable que ya forma parte del flake.
    noctaliaPackage =
      if builtins.hasAttr "noctalia" pkgs
      then pkgs.noctalia
      else nixpkgs-inestable.legacyPackages.${sistema}.noctalia;

    resolver = elegida:
      if builtins.hasAttr elegida pkgs
      then let
        paquete = builtins.getAttr elegida pkgs;
      in {
        inherit elegida paquete;
        nombre =
          if paquete ? pname && paquete.pname != null
          then builtins.toString paquete.pname
          else elegida;
        version =
          if paquete ? version && paquete.version != null
          then builtins.toString paquete.version
          else "";
      }
      else
        throw ''
          No encontré «${elegida}» en Nixpkgs.
          Revisa el nombre en configuracion.toml.
        '';

    resueltas = map resolver (configuracion.aplicaciones.instaladas or []);
    aplicaciones = map (aplicacion: aplicacion.paquete) resueltas;

    planAplicaciones =
      map
      (aplicacion: builtins.removeAttrs aplicacion ["paquete"])
      resueltas;

    planPersonas =
      map (persona: {
        inherit (persona) cuenta;
        administrador = persona.administrador or false;
      })
      personas;

    programa = pkgs.rustPlatform.buildRustPackage {
      pname = "korunix";
      version = "0.1.0";
      src = ./.;

      cargoLock.lockFile = ./Cargo.lock;

      passthru.plan = {
        inherit nombre canal escritorio;
        personas = planPersonas;
        revision = nixpkgs.rev or "";
        aplicaciones = planAplicaciones;
        noctalia = noctaliaActivo;
        noctalia_version =
          if noctaliaActivo
          then noctaliaPackage.version or ""
          else "";
      };
    };
  in {
    packages.${sistema} = {
      default = programa;
      korunix = programa;

      aplicaciones = pkgs.buildEnv {
        name = "korunix-aplicaciones";
        paths = aplicaciones;
      };
    };

    nixosConfigurations.korunix = nixpkgs.lib.nixosSystem {
      system = sistema;

      specialArgs = {
        inherit
          aplicaciones
          escritorio
          noctaliaPackage
          nombre
          personas
          programa
          ;
      };

      modules = [./sistema.nix];
    };

    formatter.${sistema} = pkgs.alejandra;
  };
}

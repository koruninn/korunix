{
  description = "Korunix";

  inputs = {
    # NixOS unstable para los equipos que siguen el canal de desarrollo.
    nixpkgs.url = "nixpkgs/nixos-unstable";

    # NixOS stable para los equipos que requieren una base estable.
    nixpkgs-stable.url = "nixpkgs/nixos-26.05";

    # Anime Game Launcher
    aagl.url = "github:ezKEa/aagl-gtk-on-nix";
    aagl.inputs.nixpkgs.follows = "nixpkgs";

    # Alejandra
    alejandra.url = "github:kamadorueda/alejandra";
    alejandra.inputs.nixpkgs.follows = "nixpkgs";

    # Flatpak declarativo
    nix-flatpak.url = "github:gmodena/nix-flatpak?ref=latest";

    # Figma
    figma-linux-next.url = "github:arximus88/figma-linux-next";

    # Millennium para Steam
    millennium.url = "github:SteamClientHomebrew/Millennium?dir=packages/nix";

    # Spicetify-Nix (Spotify + Spicetify)
    spicetify-nix = {
      url = "github:Gerg-L/spicetify-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # Zen Browser
    zen-browser = {
      url = "github:youwen5/zen-browser-flake";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # Shell Noctalia
    noctalia = {
      url = "github:noctalia-dev/noctalia/";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    alejandra,
    figma-linux-next,
    nix-flatpak,
    nixpkgs,
    nixpkgs-stable,
    noctalia,
    self,
    spicetify-nix,
    zen-browser,
    ...
  } @ inputs: let
    directorios = builtins.readDir ./equipos;
    equipos = builtins.filter (
      nombre: directorios.${nombre} == "directory"
    ) (builtins.attrNames directorios);

    crearEquipo = nombre: let
      equipoOriginal = import ./equipos/${nombre}/equipo.nix;

      cadenaObligatoria = campo:
        if !(builtins.hasAttr campo equipoOriginal)
        then throw "Falta ${campo} en equipos/${nombre}/equipo.nix."
        else let
          valor = builtins.getAttr campo equipoOriginal;
        in
          if builtins.isString valor && valor != ""
          then valor
          else throw "${campo} debe ser una cadena no vacía en equipos/${nombre}/equipo.nix.";

      canalEquipo = cadenaObligatoria "canal";
      arquitecturaEquipo = cadenaObligatoria "arquitectura";
      personaEquipo = cadenaObligatoria "persona";

      pantallaEquipo =
        if !(equipoOriginal ? pantalla)
        then null
        else let
          pantalla = equipoOriginal.pantalla;
          campos = ["nombre" "modo" "escala"];
          faltantes = builtins.filter (campo: !(builtins.hasAttr campo pantalla)) campos;
        in
          if faltantes == []
          then pantalla
          else
            throw "Pantalla incompleta en equipos/${nombre}/equipo.nix. Faltan: ${builtins.concatStringsSep ", " faltantes}.";

      equipo =
        equipoOriginal
        // {
          canal = canalEquipo;
          arquitectura = arquitecturaEquipo;
          persona = personaEquipo;
        }
        // (
          if pantallaEquipo == null
          then {}
          else {pantalla = pantallaEquipo;}
        );

      nixpkgsSeleccionado =
        if canalEquipo == "stable"
        then nixpkgs-stable
        else if canalEquipo == "unstable"
        then nixpkgs
        else throw "Canal no válido en equipos/${nombre}/equipo.nix: ${canalEquipo}. Usa stable o unstable.";
      system = arquitecturaEquipo;
      lib = nixpkgsSeleccionado.lib;
    in {
      name = nombre;
      value = lib.nixosSystem {
        inherit system;
        specialArgs = {
          inherit inputs equipo nombre;
          canal = canalEquipo;
        };
        modules = [
          ./equipos/${nombre}
          ./modulos/base
          {
            networking.hostName = nombre;
          }
        ];
      };
    };
  in {
    formatter = nixpkgs.lib.genAttrs ["x86_64-linux" "aarch64-linux"] (
      system: nixpkgs.legacyPackages.${system}.alejandra
    );

    nixosConfigurations = builtins.listToAttrs (map crearEquipo equipos);
  };
}

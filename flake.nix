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
      equipo = import ./equipos/${nombre}/equipo.nix;
      nixpkgsSeleccionado =
        if equipo.canal == "stable"
        then nixpkgs-stable
        else nixpkgs;
      system = equipo.arquitectura;
      lib = nixpkgsSeleccionado.lib;
      pkgs = import nixpkgsSeleccionado {
        inherit system;
        config.allowUnfree = true;
      };
    in {
      name = nombre;
      value = lib.nixosSystem {
        inherit system;
        specialArgs = {
          inherit inputs equipo nombre;
          canal = equipo.canal;
        };
        modules = [
          ./equipos/${nombre}
          ./modulos/base
          ./aplicaciones
          inputs.aagl.nixosModules.default
          {
            networking.hostName = nombre;
            environment.systemPackages = [
              pkgs.alejandra
            ];
          }
        ];
      };
    };
  in {
    formatter.x86_64-linux = nixpkgs.legacyPackages.x86_64-linux.alejandra;

    nixosConfigurations = builtins.listToAttrs (map crearEquipo equipos);
  };
}

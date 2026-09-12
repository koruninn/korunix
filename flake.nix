{
  description = "Korunix";

  nixConfig = {
    extra-substituters = [
      "https://cache.forall.systems"
    ];
    extra-trusted-public-keys = [
      "cache.forall.systems:5PmD7QO4MSF8YgyRZtkSGXRDo96H3bybIf2SsQh8ScI="
    ];
  };

  inputs = {
    # NixOS unstable para los equipos que siguen el canal de desarrollo.
    nixpkgs.url = "nixpkgs/nixos-unstable";

    # NixOS stable para los equipos que requieren una base estable.
    nixpkgs-stable.url = "nixpkgs/nixos-26.05";

    # Nixpkgs exacto fijado por affinity-nix para que sus derivaciones
    # coincidan con los artefactos publicados en cache.forall.systems.
    affinity-nixpkgs.url = "github:NixOS/nixpkgs/dc5d91f840324650bac8c379428c7037a416959a";

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

    # Affinity para Linux
    affinity-nix.url = "github:mrshmllow/affinity-nix";

    # Shell Noctalia
    noctalia = {
      url = "github:noctalia-dev/noctalia/";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    affinity-nix,
    affinity-nixpkgs,
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
      affinityPkgs = import affinity-nixpkgs {
        inherit system;
        config.allowUnfree = true;
        overlays = [affinity-nix.overlays.default];
      };
      affinity = affinityPkgs.affinity-v3;
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
          ./apariencia
          inputs.aagl.nixosModules.default
          inputs.noctalia.nixosModules.default
          inputs.spicetify-nix.nixosModules.default
          {
            networking.hostName = nombre;
            environment.systemPackages = [
              pkgs.alejandra
              affinity
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

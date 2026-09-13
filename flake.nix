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
      canalEquipo =
        equipoOriginal.canal
        or (throw "Falta canal en equipos/${nombre}/equipo.nix.");
      arquitecturaEquipo =
        equipoOriginal.arquitectura
        or (throw "Falta arquitectura en equipos/${nombre}/equipo.nix.");
      personaEquipo =
        equipoOriginal.persona
        or (throw "Falta persona en equipos/${nombre}/equipo.nix.");

      equipo = equipoOriginal // {
        canal = canalEquipo;
        arquitectura = arquitecturaEquipo;
        persona = personaEquipo;
      };

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
    formatter.x86_64-linux = nixpkgs.legacyPackages.x86_64-linux.alejandra;

    nixosConfigurations = builtins.listToAttrs (map crearEquipo equipos);
  };
}

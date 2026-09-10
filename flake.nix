{
  description = "Korunix";

  inputs = {
    # Core NixOS package repository
    nixpkgs.url = "nixpkgs/nixos-unstable";

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
    noctalia,
    self,
    spicetify-nix,
    zen-browser,
    ...
  } @ inputs: let
    system = "x86_64-linux";
    lib = nixpkgs.lib;
    pkgs = import nixpkgs {
      inherit system;
    };
  in {
    # Formateador automático
    formatter.${system} = pkgs.alejandra;

    # Configuración del Sistema (NixOS)
    nixosConfigurations = {
      korunix = lib.nixosSystem {
        inherit system;
        specialArgs = {inherit inputs;};
        modules = [
          ./equipos/korunix/configuracion.nix
          ./aplicaciones
          ./apariencia
          inputs.aagl.nixosModules.default
          inputs.noctalia.nixosModules.default          
          inputs.spicetify-nix.nixosModules.default
          {
            environment.systemPackages = [alejandra.defaultPackage.${system}];
          }
        ];
      };
    };
  };
}

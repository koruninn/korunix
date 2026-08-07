{
  description = "Korunix";

  inputs = {
    # Core NixOS package repository

    nixpkgs.url = "nixpkgs/nixos-unstable";

    hatter = {
      url = "github:Mibea/Hatter";

      flake = false;
    };

    # User environment and dotfile management

    home-manager = {
      url = "github:nix-community/home-manager";

      inputs.nixpkgs.follows = "nixpkgs";
    };

    millennium.url = "github:SteamClientHomebrew/Millennium?dir=packages/nix";

    # Anime Game Launcher

    aagl.url = "github:ezKEa/aagl-gtk-on-nix";

    aagl.inputs.nixpkgs.follows = "nixpkgs";

    # Alejandra

    alejandra.url = "github:kamadorueda/alejandra/4.0.0";

    alejandra.inputs.nixpkgs.follows = "nixpkgs";

    # Declarative Flatpak application management

    nix-flatpak.url = "github:gmodena/nix-flatpak?ref=latest";

    # Declarative Spicetify configuration (Spotify theming)

    spicetify-nix = {
      url = "github:Gerg-L/spicetify-nix";

      inputs.nixpkgs.follows = "nixpkgs";
    };

    # Noctalia desktop shell (bar, notifications, lock screen)

    noctalia = {
      url = "github:noctalia-dev/noctalia/";

      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    aagl,
    alejandra,
    nixpkgs,
    home-manager,
    spicetify-nix,
    noctalia,
    nix-flatpak,
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

    # 1. Configuración del Sistema (NixOS)

    nixosConfigurations = {
      korunix = lib.nixosSystem {
        inherit system;

        specialArgs = {inherit inputs;};

        modules = [
          ./hosts/korunix/configuration.nix

          inputs.aagl.nixosModules.default

          inputs.noctalia.nixosModules.default

          {
            environment.systemPackages = [alejandra.defaultPackage.${system}];
          }
        ];
      };
    };

    # 2. Configuración del Usuario (Home Manager)

    homeConfigurations = {
      koru = home-manager.lib.homeManagerConfiguration {
        inherit pkgs;

        extraSpecialArgs = {inherit inputs;};

        modules = [
          ./users/koru/home.nix
        ];
      };
    };
  };
}

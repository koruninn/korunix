{
  description = "Korunix";

  # Cada entrada representa una pieza externa que Korunix necesita para construir
  # el sistema. Las versiones concretas quedan fijadas en flake.lock para que una
  # reconstrucción futura utilice exactamente las mismas revisiones.
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";

    hatter = {
      url = "github:Mibea/Hatter";
      flake = false;
    };

    millennium.url = "github:SteamClientHomebrew/Millennium?dir=packages/nix";

    aagl = {
      url = "github:ezKEa/aagl-gtk-on-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    alejandra = {
      url = "github:kamadorueda/alejandra/4.0.0";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    nix-flatpak.url = "github:gmodena/nix-flatpak?ref=latest";

    spicetify-nix = {
      url = "github:Gerg-L/spicetify-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    noctalia = {
      url = "github:noctalia-dev/noctalia/";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs @ {
    alejandra,
    nixpkgs,
    ...
  }: let
    lib = nixpkgs.lib;

    # Los hosts se descubren por nombre de archivo. Añadir hosts/portatil.nix y
    # hardware/portatil.nix basta para que aparezca nixosConfigurations.portatil.
    hostEntries = builtins.readDir ./hosts;

    hostFiles =
      lib.filterAttrs (
        name: type:
          type == "regular" && lib.hasSuffix ".nix" name
      )
      hostEntries;

    hostIds = map (
      name: lib.removeSuffix ".nix" name
    ) (builtins.attrNames hostFiles);

    hostFileFor = hostId: ./hosts + "/${hostId}.nix";
    hardwareFileFor = hostId: ./hardware + "/${hostId}.nix";

    hostDataFor = hostId: import (hostFileFor hostId);

    systems = lib.unique (map (
        hostId: (hostDataFor hostId).system
      )
      hostIds);

    makeHost = hostId: let
      hostFile = hostFileFor hostId;
      hardwareCandidate = hardwareFileFor hostId;
      hardwareFile =
        if builtins.pathExists hardwareCandidate
        then hardwareCandidate
        else throw "Falta hardware/${hostId}.nix para el host ${hostId}.";
      host = hostDataFor hostId;
      system = host.system;
    in
      lib.nixosSystem {
        inherit system;

        # specialArgs solo transporta contexto estructural. Las decisiones que
        # una persona puede editar viven en config.korunix.*.
        specialArgs = {
          inherit inputs;

          korunixContext = {
            inherit hostId hostFile hardwareFile;
            usersPath = ./users;
            configPath = ./config;
          };
        };

        modules = [
          hardwareFile

          ./modules/core.nix
          ./modules/hardware.nix
          ./modules/boot.nix
          ./modules/localization.nix
          ./modules/desktop.nix
          ./modules/apps.nix
          ./modules/services.nix
          ./modules/users.nix

          inputs.aagl.nixosModules.default
          inputs.noctalia.nixosModules.default
          inputs.spicetify-nix.nixosModules.default

          # El archivo del host es datos humanos, no otro módulo oculto. Aquí se
          # convierte esa información en el árbol declarativo de Korunix.
          {
            korunix =
              host.korunix
              // {
                inherit hostId;
                users = host.users;
              };

            environment.systemPackages = [
              alejandra.defaultPackage.${system}
            ];
          }
        ];
      };
  in {
    # Alejandra está disponible para cada arquitectura que realmente aparezca
    # entre los hosts declarados, en lugar de asumir x86_64 globalmente.
    formatter = lib.genAttrs systems (
      system: alejandra.defaultPackage.${system}
    );

    nixosConfigurations = lib.genAttrs hostIds makeHost;
  };
}

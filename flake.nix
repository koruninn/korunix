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

    pkgsFor = system:
      import nixpkgs {
        inherit system;
      };

    # La aplicación gráfica consume el checkout humano mediante KORUNIX_ROOT o
    # ~/.korunix. Todavía no sustituye el motor ni instala una configuración.
    korunixGuiFor = system: let
      pkgs = pkgsFor system;
      python = pkgs.python3.withPackages (pythonPackages: [
        pythonPackages.babel
        pythonPackages.pygobject3
      ]);
    in
      pkgs.stdenvNoCC.mkDerivation {
        pname = "korunix";
        version = "0.1.0";
        src = ./app;

        nativeBuildInputs = [
          pkgs.desktop-file-utils
          pkgs.gobject-introspection
          pkgs.makeWrapper
          pkgs.wrapGAppsHook4
        ];

        buildInputs = [
          pkgs.gtk4
          pkgs.libadwaita
        ];

        dontBuild = true;
        doCheck = true;

        checkPhase = ''
          runHook preCheck
          ${python}/bin/python3 -m compileall -q .
          runHook postCheck
        '';

        installPhase = ''
          runHook preInstall

          install -Dm755 korunix.py "$out/share/korunix/korunix.py"
          install -Dm644 korunix_backend.py "$out/share/korunix/korunix_backend.py"
          install -Dm644 korunix_i18n.py "$out/share/korunix/korunix_i18n.py"
          install -Dm644 style.css "$out/share/korunix/style.css"
          install -Dm644 io.github.koruninn.Korunix.desktop \
            "$out/share/applications/io.github.koruninn.Korunix.desktop"

          makeWrapper ${python}/bin/python3 "$out/bin/korunix" \
            --add-flags "$out/share/korunix/korunix.py"

          runHook postInstall
        '';

        meta = {
          description = "Centro de control gráfico de Korunix";
          mainProgram = "korunix";
          platforms = lib.platforms.linux;
        };
      };

    makeHost = hostId: let
      hostFile = hostFileFor hostId;
      hardwareCandidate = hardwareFileFor hostId;
      hardwareFile =
        if builtins.pathExists hardwareCandidate
        then hardwareCandidate
        else throw "Falta hardware/${hostId}.nix para el host ${hostId}.";
      host = hostDataFor hostId;

      # El formato actual hace que users sea un attrset: las claves son IDs
      # portables y los valores contienen únicamente estado local del host.
      # La rama de lista permite leer un host antiguo durante una migración.
      hostUsersRaw = host.users or [];
      hostUserIds =
        if builtins.isList hostUsersRaw
        then hostUsersRaw
        else builtins.attrNames hostUsersRaw;
      hostUserSettings =
        if builtins.isList hostUsersRaw
        then {}
        else hostUsersRaw;

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
                users = hostUserIds;
                userSettings = hostUserSettings;
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

    packages = lib.genAttrs systems (system: {
      default = korunixGuiFor system;
      korunix = korunixGuiFor system;
    });

    apps = lib.genAttrs systems (system: {
      default = {
        type = "app";
        program = "${korunixGuiFor system}/bin/korunix";
        meta.description = "Abrir el centro de control de Korunix";
      };

      korunix = {
        type = "app";
        program = "${korunixGuiFor system}/bin/korunix";
        meta.description = "Abrir el centro de control de Korunix";
      };
    });

    checks = lib.genAttrs systems (system: {
      korunix-gui = korunixGuiFor system;
    });

    nixosConfigurations = lib.genAttrs hostIds makeHost;
  };
}

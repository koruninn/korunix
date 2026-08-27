{
  description = "Korunix";

  # Cada entrada representa una pieza externa que Korunix necesita para construir
  # el sistema. Las versiones concretas quedan fijadas en flake.lock para que una
  # reconstrucción futura utilice exactamente las mismas revisiones.
  inputs = {
    # La entrada histórica sigue representando Inestable.
    nixpkgs.url = "nixpkgs/nixos-unstable";

    # Cada host puede utilizar 26.05 como base sin retirar el conjunto inestable
    # que algunas aplicaciones aisladas pueden necesitar.
    nixpkgsStable.url = "nixpkgs/nixos-26.05";

    hatter = {
      url = "github:Mibea/Hatter";
      flake = false;
    };

    millennium.url = "github:SteamClientHomebrew/Millennium?dir=packages/nix";

    # AAGL principal acompaña al canal inestable.
    aagl = {
      url = "github:ezKEa/aagl-gtk-on-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # AAGL publica una rama correspondiente a NixOS 26.05.
    aaglStable = {
      url = "github:ezKEa/aagl-gtk-on-nix/release-26.05";
      inputs.nixpkgs.follows = "nixpkgsStable";
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

    # Los hosts se descubren por nombre de archivo. Añadir equipos/portatil.nix y
    # equipos/portatil-detectado.nix basta para que aparezca nixosConfigurations.portatil.
    hostEntries = builtins.readDir ./equipos;

    hostFiles =
      lib.filterAttrs (
        name: type:
          type
          == "regular"
          && lib.hasSuffix ".nix" name
          && !(lib.hasSuffix "-detectado.nix" name)
      )
      hostEntries;

    hostIds = map (
      name: lib.removeSuffix ".nix" name
    ) (builtins.attrNames hostFiles);

    hostFileFor = hostId: ./equipos + "/${hostId}.nix";
    hardwareFileFor = hostId: ./equipos + "/${hostId}-detectado.nix";

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
        else throw "Falta equipos/${hostId}-detectado.nix para el host ${hostId}.";
      host = hostDataFor hostId;

      # El canal es estado del host, no del repositorio completo.
      channel = host.korunix.channel or "stable";

      selectedNixpkgs =
        if channel == "stable"
        then inputs.nixpkgsStable
        else inputs.nixpkgs;

      selectedAagl =
        if channel == "stable"
        then inputs.aaglStable
        else inputs.aagl;

      # Los módulos que consumen `inputs.nixpkgs` o `inputs.aagl` reciben la
      # familia correspondiente al host que se está evaluando.
      hostInputs =
        inputs
        // {
          nixpkgs = selectedNixpkgs;
          aagl = selectedAagl;
        };

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

      # Patrón deliberadamente limitado: la base sigue siendo una sola rama,
      # pero los módulos pueden solicitar una excepción de paquete concreta.
      pkgsStable = inputs.nixpkgsStable.legacyPackages.${system};
      pkgsUnstable = inputs.nixpkgs.legacyPackages.${system};
    in
      selectedNixpkgs.lib.nixosSystem {
        inherit system;

        # specialArgs solo transporta contexto estructural. Las decisiones que
        # una persona puede editar viven en config.korunix.*.
        specialArgs = {
          inputs = hostInputs;
          inherit pkgsStable pkgsUnstable;

          korunixContext = {
            inherit hostId hostFile hardwareFile;
            personasPath = ./personas;
            configPath = ./config;
          };
        };

        modules = [
          hardwareFile

          ./sistema/base.nix
          ./sistema/equipo.nix
          ./sistema/arranque.nix
          ./sistema/idioma.nix
          ./sistema/escritorio.nix
          ./sistema/aplicaciones.nix
          ./sistema/servicios.nix
          ./sistema/personas.nix

          selectedAagl.nixosModules.default
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

    # API declarativa que puede consumir el motor sin reimplementar el modelo.
    # Los nombres son conceptos del producto; las salidas impuestas por Nix,
    # como nixosConfigurations, conservan su nombre oficial.
    korunix = {
      esquema = 1;
      canales = import ./canales.nix;
      predeterminados = import ./predeterminados.nix;
      equipos = lib.genAttrs hostIds hostDataFor;
    };

    nixosConfigurations = lib.genAttrs hostIds makeHost;
  };
}

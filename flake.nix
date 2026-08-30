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

    # Los hosts se descubren por nombre de archivo. Añadir configuracion/equipos/portatil.nix y
    # generado/equipos/portatil-detectado.nix basta para que aparezca nixosConfigurations.portatil.
    hostEntries = builtins.readDir ./configuracion/equipos;

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

    hostFileFor = hostId: ./configuracion/equipos + "/${hostId}.nix";
    hardwareFileFor = hostId: ./generado/equipos + "/${hostId}-detectado.nix";

    hostDataFor = hostId: import (hostFileFor hostId);

    # El producto se publica para las dos arquitecturas que el bootstrap sabe
    # detectar. Las salidas no dependen de que exista ya un host local de esa
    # arquitectura dentro de configuracion/equipos.
    supportedSystems = ["x86_64-linux" "aarch64-linux"];
    hostSystems = lib.unique (map (
        hostId: (hostDataFor hostId).system
      )
      hostIds);
    systems = lib.unique (
      [
        "x86_64-linux"
        "aarch64-linux"
      ]
      ++ map (
        hostId: (hostDataFor hostId).system
      )
      hostIds
    );

    # El código empaquetado nunca arrastra la configuración humana de quien
    # construyó Korunix ni los resultados locales de desarrollo.
    productSource = lib.cleanSourceWith {
      src = ./.;
      filter = path: type: let
        root = toString ./. + "/";
        relative = lib.removePrefix root (toString path);
      in
        !(lib.hasPrefix ".git/" relative)
        && !(lib.hasPrefix "target/" relative)
        && !(lib.hasPrefix "configuracion/equipos/" relative)
        && !(lib.hasPrefix "configuracion/personas/" relative)
        && !(lib.hasPrefix "generado/equipos/" relative);
    };

    pkgsFor = system:
      import nixpkgs {
        inherit system;
      };

    # Motor Rust único. Nix empaqueta el ejecutable; las operaciones del
    # sistema vivo ya no dependen del dominio Bash histórico.
    korunixMotorFor = system: let
      pkgs = pkgsFor system;
    in
      pkgs.rustPlatform.buildRustPackage {
        pname = "korunix";
        version = "0.1.0";
        src = productSource;

        cargoLock.lockFile = ./Cargo.lock;

        # El motor llama herramientas del sistema vivo. Se empaquetan sus
        # ejecutables de consulta/operación para que la GUI y `nix run .#motor`
        # no dependan accidentalmente del PATH de la sesión desde la que se abren.
        nativeBuildInputs = [
          pkgs.makeWrapper
        ];

        cargoBuildFlags = [
          "--bin"
          "korunix"
        ];

        postFixup = ''
          wrapProgram "$out/bin/korunix" \
            --prefix PATH : ${lib.makeBinPath [
            pkgs.coreutils
            pkgs.ffmpeg
            pkgs.flatpak
            pkgs.fwupd
            pkgs.git
            pkgs.gnutar
            pkgs.jq
            pkgs.nix
            pkgs.pciutils
            pkgs.pulseaudio
            pkgs.pipewire
            pkgs.systemd
            pkgs.udisks2
            pkgs.util-linux
            pkgs.v4l-utils
            pkgs.wireplumber
          ]}
        '';

        meta = {
          description = "Motor operativo de Korunix";
          mainProgram = "korunix";
          platforms = lib.platforms.linux;
        };
      };

    # La interfaz es Rust + GTK/libadwaita y únicamente presenta el motor
    # público. Las dependencias gráficas son opcionales en Cargo para que el
    # motor de terminal siga siendo una pieza pequeña e independiente.
    korunixGuiFor = system: let
      pkgs = pkgsFor system;
    in
      pkgs.rustPlatform.buildRustPackage {
        pname = "korunix-interfaz";
        version = "0.1.0";
        src = productSource;

        cargoLock.lockFile = ./Cargo.lock;

        nativeBuildInputs = [
          pkgs.appstream
          pkgs.desktop-file-utils
          pkgs.makeWrapper
          pkgs.pkg-config
          pkgs.wrapGAppsHook4
        ];

        buildInputs = [
          pkgs.gtk4
          pkgs.libadwaita
        ];

        cargoBuildFlags = [
          "--features"
          "interfaz"
          "--bin"
          "korunix-interfaz"
        ];

        cargoInstallFlags = [
          "--features"
          "interfaz"
          "--bin"
          "korunix-interfaz"
        ];

        postInstall = ''
          install -Dm644 sistema/interfaz/io.github.koruninn.Korunix.desktop \
            "$out/share/applications/io.github.koruninn.Korunix.desktop"
          install -Dm644 sistema/interfaz/io.github.koruninn.Korunix.metainfo.xml \
            "$out/share/metainfo/io.github.koruninn.Korunix.metainfo.xml"

          desktop-file-validate "$out/share/applications/io.github.koruninn.Korunix.desktop"
          appstreamcli validate --no-net \
            "$out/share/metainfo/io.github.koruninn.Korunix.metainfo.xml"

          makeWrapper             "$out/bin/korunix-interfaz"             "$out/bin/korunix"             --set KORUNIX_MOTOR_BIN ${korunixMotorFor system}/bin/korunix
        '';

        meta = {
          description = "Centro de control GTK/libadwaita de Korunix";
          mainProgram = "korunix";
          platforms = lib.platforms.linux;
        };
      };

    # Puente único de instalación y actualización. Puede ejecutarse desde
    # GitHub o desde una copia local/USB. Conserva configuracion/, generado/ y el
    # historial Git; si el código local está modificado se niega a sobrescribirlo.
    # El bootstrap público vive como Bash mínimo y no contiene dominio.
    # Solo consigue/abre el motor Rust y le entrega la adopción real.
    korunixBootstrapFor = system: let
      pkgs = pkgsFor system;
    in
      pkgs.writeShellApplication {
        name = "korunix-bootstrap";

        runtimeInputs = [
          pkgs.coreutils
          pkgs.nix
        ];

        # Cuando el bootstrap se ejecuta directamente como aplicación Nix, el
        # código está en el store y no es editable. Entregamos al puente Bash la
        # fuente exacta del producto para que pueda crear ~/.korunix antes de
        # arrancar el motor. La lógica de adopción continúa íntegramente en Rust.
        text = ''
          export KORUNIX_BOOTSTRAP_SOURCE=${lib.escapeShellArg (toString productSource)}
          ${builtins.readFile ./scripts/korunix-bootstrap}
        '';
      };

    makeHost = hostId: let
      hostFile = hostFileFor hostId;
      hardwareCandidate = hardwareFileFor hostId;
      hardwareFile =
        if builtins.pathExists hardwareCandidate
        then hardwareCandidate
        else throw "Falta generado/equipos/${hostId}-detectado.nix para el host ${hostId}.";
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
            personasPath = ./configuracion/personas;
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
      motor = korunixMotorFor system;
      bootstrap = korunixBootstrapFor system;
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

      motor = {
        type = "app";
        program = "${korunixMotorFor system}/bin/korunix";
        meta.description = "Ejecutar el motor Rust de Korunix";
      };

      bootstrap = {
        type = "app";
        program = "${korunixBootstrapFor system}/bin/korunix-bootstrap";
        meta.description = "Instalar o actualizar Korunix conservando la configuración humana";
      };
    });

    checks = lib.genAttrs systems (system: {
      korunix-gui = korunixGuiFor system;
      korunix-motor = korunixMotorFor system;
      korunix-bootstrap = korunixBootstrapFor system;
    });

    # API declarativa que puede consumir el motor sin reimplementar el modelo.
    # Los nombres son conceptos del producto; las salidas impuestas por Nix,
    # como nixosConfigurations, conservan su nombre oficial.

    nixosConfigurations = lib.genAttrs hostIds makeHost;
  };
}

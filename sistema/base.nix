{
  config,
  inputs,
  lib,
  pkgs,
  pkgsUnstable ? null,
  ...
}: let
  cfg = config.korunix;

  # Hatter llega como un repositorio de iconos, no como un paquete de Nix. Este
  # pequeño envoltorio lo instala en una ubicación estándar para todo el sistema.
  hatterIcons = pkgs.runCommand "korunix-hatter-icons" {} ''
    mkdir -p "$out/share/icons"
    cp -R ${inputs.hatter}/Hatter "$out/share/icons/Hatter"
    cp -R ${inputs.hatter}/Hatter-Green "$out/share/icons/Hatter-Green"
    cp -R ${inputs.hatter}/Hatter-Slate "$out/share/icons/Hatter-Slate"
  '';

  # Algunas aplicaciones pueden llegar a una rama de nixpkgs antes que a la
  # estable. Solo usamos el conjunto inestable como excepción puntual cuando
  # el paquete no existe en la base elegida para el sistema.
  fetchPackage =
    if pkgs ? fetch
    then pkgs.fetch
    else if pkgsUnstable != null && pkgsUnstable ? fetch
    then pkgsUnstable.fetch
    else throw "Korunix necesita Fetch, pero ninguna fuente disponible lo contiene.";
in {
  options.korunix = {
    enable = lib.mkEnableOption "Korunix";

    hostId = lib.mkOption {
      type = lib.types.str;
      description = "Identificador estructural estable del equipo dentro de Korunix.";
    };

    hostName = lib.mkOption {
      type = lib.types.str;
      description = "Nombre visible del equipo en el sistema y en la red.";
    };

    stateVersion = lib.mkOption {
      type = lib.types.str;
      description = "Versión de compatibilidad histórica de esta instalación de NixOS.";
    };

    channel = lib.mkOption {
      type = lib.types.enum ["stable" "unstable"];
      default = "stable";
      description = ''
        Canal de actualizaciones de este equipo. Estable prioriza una base
        mantenida de NixOS; inestable prioriza versiones más recientes.
        Esta decisión nunca modifica system.stateVersion.
      '';
    };

    users = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [];
      description = "Personas que tienen una cuenta en este equipo.";
    };

    # La identidad portable vive en configuracion/personas/<id>.nix. Estos valores pertenecen a
    # la relación entre esa persona y este host concreto.
    userSettings = lib.mkOption {
      type = lib.types.attrsOf (lib.types.submodule {
        options = {
          accountName = lib.mkOption {
            type = lib.types.nullOr lib.types.str;
            default = null;
            description = "Nombre UNIX local cuando el host necesita una excepción al perfil portable.";
          };

          homeDirectory = lib.mkOption {
            type = lib.types.nullOr lib.types.str;
            default = null;
            description = "Home local de la persona en este equipo.";
          };

          administrator = lib.mkOption {
            type = lib.types.bool;
            default = false;
            description = "Si la persona administra este equipo concreto.";
          };

          deferredCapabilities = lib.mkOption {
            type = lib.types.listOf lib.types.str;
            default = [];
            description = "Capacidades portables que este host todavía no puede satisfacer.";
          };

          deferredInputMethods = lib.mkOption {
            type = lib.types.listOf lib.types.str;
            default = [];
            description = ''
              Métodos de entrada portables que esta máquina todavía no puede
              satisfacer. La intención permanece en el perfil de la persona.
            '';
          };

          preservedGroups = lib.mkOption {
            type = lib.types.listOf lib.types.str;
            default = [];
            description = "Grupos técnicos locales conservados durante una adopción.";
          };

          githubSshIdentityFile = lib.mkOption {
            type = lib.types.nullOr lib.types.str;
            default = null;
            description = "Ruta local de la clave SSH que esta persona usa con GitHub en este host.";
          };
        };
      });

      default = {};
      description = "Estado local por usuario que no forma parte de su identidad portable.";
    };
  };

  config = lib.mkIf cfg.enable {
    networking.hostName = cfg.hostName;

    # stateVersion protege decisiones de compatibilidad histórica. Cambiar de
    # canal o actualizar paquetes no modifica este valor automáticamente.
    system.stateVersion = cfg.stateVersion;

    nix.settings = {
      experimental-features = [
        "nix-command"
        "flakes"
      ];

      auto-optimise-store = true;

      extra-substituters = [
        "https://noctalia.cachix.org"
        "https://ezkea.cachix.org"
      ];

      extra-trusted-public-keys = [
        "noctalia.cachix.org-1:pCOR47nnMEo5thcxNDtzWpOxNFQsBRglJzxWPp3dkU4="
        "ezkea.cachix.org-1:io85OCXmr5WwSZQYw7066RA2fNdOeOwGEgMDwiDxUCg="
      ];
    };

    nixpkgs.config = {
      allowUnfree = true;

      permittedInsecurePackages = [
        "electron-40.10.5"
      ];
    };

    nixpkgs.overlays = [
      inputs.millennium.overlays.default
    ];

    # Alacritty y Fish forman parte de la experiencia de terminal de Korunix.
    # Fetch lleva Fastfetch como dependencia interna, por lo que Fastfetch no se
    # instala ni se muestra como elección duplicada.
    environment.systemPackages = [
      pkgs.alacritty
      fetchPackage
      pkgs.bibata-cursors
      hatterIcons
    ];

    # Korunix usa el agente estándar de OpenSSH. GCR pertenece a la
    # integración de GNOME y no debe apropiarse de SSH_AUTH_SOCK.
    services.gnome.gcr-ssh-agent.enable = false;
    programs.ssh.startAgent = true;

    programs.fish = {
      enable = true;
      interactiveShellInit = builtins.readFile ../config/fish.conf;
    };

    programs.appimage = {
      enable = true;
      binfmt = true;
    };

    hardware.graphics = {
      enable = true;
      enable32Bit = true;
    };

    environment.sessionVariables = {
      XCURSOR_THEME = "Bibata-Modern-Classic";
      XCURSOR_SIZE = "24";
    };
  };
}

{
  config,
  inputs,
  lib,
  pkgs,
  ...
}: let
  cfg = config.korunix;

  # Hatter llega como un repositorio de iconos, no como un paquete de Nix. Este
  # pequeño envoltorio lo instala en una ubicación estándar para todo el sistema.
  hatterIcons = pkgs.runCommandNoCC "korunix-hatter-icons" {} ''
    mkdir -p "$out/share/icons"
    cp -R ${inputs.hatter}/Hatter "$out/share/icons/Hatter"
    cp -R ${inputs.hatter}/Hatter-Slate "$out/share/icons/Hatter-Slate"
  '';
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

    users = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [];
      description = "Personas que tienen una cuenta en este equipo.";
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

    # Alacritty y Fish forman parte de la experiencia de terminal de Korunix. El
    # programa Fetch depende de Fastfetch dentro de su propio paquete de nixpkgs,
    # por lo que Fastfetch no se instala ni se muestra como elección duplicada.
    environment.systemPackages = [
      pkgs.alacritty
      pkgs.fetch
      pkgs.bibata-cursors
      hatterIcons
    ];

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

    # La selección definitiva entre systemd-boot y GRUB se hará en un bloque
    # dedicado a firmware y dual boot. Por ahora se conserva el cargador vigente
    # para que retirar Home Manager no cambie también la cadena de arranque.
    boot = {
      kernelPackages = pkgs.linuxPackages_latest;

      kernelParams = [
        "quiet"
        "splash"
        "boot.shell_on_fail"
      ];

      plymouth.enable = true;

      loader = {
        limine = {
          enable = true;
          extraConfig = ''
            timeout: 5
          '';
        };

        efi.canTouchEfiVariables = true;
      };
    };

    environment.sessionVariables = {
      XCURSOR_THEME = "Bibata-Modern-Classic";
      XCURSOR_SIZE = "24";
    };
  };
}

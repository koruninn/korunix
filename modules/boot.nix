{
  config,
  lib,
  pkgs,
  ...
}: let
  cfg = config.korunix.boot;
  firmware = config.korunix.hardware.firmware;
in {
  options.korunix.boot = {

    # /boot es la disposición estándar de Korunix. Se puede conservar otra
    # ubicación cuando un equipo existente ya utiliza una ESP diferente.
    efiMountPoint = lib.mkOption {
      type = lib.types.str;
      default = "/boot";
      description = "Punto de montaje de la partición EFI.";
    };

    # Solo los equipos BIOS necesitan conocer el disco físico donde instalar
    # GRUB. En una adopción futura Korunix detectará y guardará este valor.
    biosDevice = lib.mkOption {
      type = lib.types.nullOr lib.types.str;
      default = null;
      description = "Disco de arranque para equipos BIOS/Legacy.";
    };
  };

  config = lib.mkIf config.korunix.enable (lib.mkMerge [
    {
      assertions = [
        {
          assertion =
            firmware != "bios"
            || cfg.biosDevice != null;

          message = ''
            Korunix detectó un equipo BIOS/Legacy, pero no tiene registrado
            el disco donde debe instalar GRUB.
          '';
        }
      ];


      # Estas decisiones ya existían en core.nix. Se conservan aquí para que
      # mover la responsabilidad al módulo de arranque no cambie la experiencia.
      boot = {
        kernelPackages = pkgs.linuxPackages_latest;

        kernelParams = [
          "quiet"
          "splash"
          "boot.shell_on_fail"
        ];

        plymouth.enable = true;

        # El menú permanece disponible durante unos segundos para que una
        # persona siempre pueda entrar a un punto de recuperación.
        loader.timeout = 5;
      };
    }

    (lib.mkIf (firmware == "uefi") {
      boot.loader = {
        efi = {
          canTouchEfiVariables = true;
          efiSysMountPoint = cfg.efiMountPoint;
        };

        systemd-boot = {
          enable = true;

          # El menú de arranque conserva una cantidad pequeña y comprensible de
          # estados. El historial completo de Nix sigue siendo una cuestión
          # separada administrada por la política de limpieza de Korunix.
          configurationLimit = 3;

          # Evita editar parámetros del kernel desde el menú de arranque.
          editor = false;

          # Una configuración nueva dispone de tres intentos de arranque.
          # Si nunca alcanza boot-complete.target, systemd-boot puede saltarla
          # y recurrir a una configuración anterior funcional.
          bootCounting = {
            enable = true;
            tries = 3;
          };
        };
      };
    })

    (lib.mkIf (firmware == "bios") {
      boot.loader.grub = {
        enable = true;

        # La aserción anterior impide construir un host BIOS sin dispositivo.
        device =
          if cfg.biosDevice == null
          then "nodev"
          else cfg.biosDevice;

        configurationLimit = 3;
      };
    })
  ]);
}

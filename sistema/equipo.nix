# Hardware y capacidades base administradas por Korunix.
#
# Este módulo contiene hechos de la máquina que Korunix necesita conocer para
# tomar decisiones automáticamente. No son preferencias que una persona deba
# aprender ni escoger durante el uso normal.
{
  config,
  lib,
  pkgs,
  ...
}: let
  cfg = config.korunix.hardware;

  nvidiaDevices =
    lib.filter
    (gpu: gpu.vendor == "nvidia")
    cfg.graphics;

  hasNvidia = nvidiaDevices != [];

  # Si hay varias GPU NVIDIA, usamos los módulos abiertos únicamente cuando
  # todas ellas están incluidas en el catálogo compatible adoptado.
  nvidiaOpenSupported =
    hasNvidia
    && lib.all
    (gpu: gpu.nvidiaOpen)
    nvidiaDevices;
in {
  options.korunix.hardware = {
    # UEFI o BIOS se detecta al adoptar el equipo. Se conserva declarativamente
    # porque Nix necesita conocerlo durante la construcción.
    firmware = lib.mkOption {
      type = lib.types.enum [
        "uefi"
        "bios"
      ];

      description = "Tipo de firmware detectado y adoptado para este equipo.";
    };

    # Las GPU son hechos físicos del host. Las preferencias visuales siguen
    # perteneciendo al escritorio, no a este modelo.
    graphics = lib.mkOption {
      type = lib.types.listOf (
        lib.types.submodule {
          options = {
            pciAddress = lib.mkOption {
              type = lib.types.str;
              description = "Dirección PCI de la GPU.";
            };

            name = lib.mkOption {
              type = lib.types.str;
              description = "Nombre detectado de la GPU.";
            };

            vendor = lib.mkOption {
              type = lib.types.enum [
                "amd"
                "intel"
                "nvidia"
                "unknown"
              ];

              description = "Fabricante gráfico normalizado.";
            };

            vendorId = lib.mkOption {
              type = lib.types.str;
              description = "PCI Vendor ID.";
            };

            deviceId = lib.mkOption {
              type = lib.types.str;
              description = "PCI Device ID.";
            };

            subsystemVendorId = lib.mkOption {
              type = lib.types.str;
              default = "0000";
              description = "PCI Subsystem Vendor ID.";
            };

            subsystemDeviceId = lib.mkOption {
              type = lib.types.str;
              default = "0000";
              description = "PCI Subsystem Device ID.";
            };

            driver = lib.mkOption {
              type = lib.types.str;
              default = "none";
              description = "Controlador del kernel asociado a la GPU.";
            };

            primary = lib.mkOption {
              type = lib.types.bool;
              default = false;
              description = "La GPU aparece como boot_vga en el kernel.";
            };

            kind = lib.mkOption {
              type = lib.types.enum [
                "integrated"
                "dedicated"
                "unknown"
              ];
              default = "unknown";

              description = ''
                Tipo físico de GPU. Korunix conserva unknown cuando la evidencia
                local no basta para distinguir integrada de dedicada.
              '';
            };

            nvidiaOpen = lib.mkOption {
              type = lib.types.bool;
              default = false;

              description = ''
                La combinación PCI está incluida en el catálogo oficial de GPU
                compatibles con los módulos abiertos de NVIDIA.
              '';
            };
          };
        }
      );

      default = [];
      description = "Adaptadores gráficos detectados y adoptados para este host.";
    };
  };

  config = lib.mkIf config.korunix.enable (
    lib.mkMerge [
      {
        hardware.enableRedistributableFirmware = true;

        # Mesa y la infraestructura gráfica común.
        hardware.graphics.enable = true;

        # Korunix mantiene la interfaz sin privilegios y solicita autorización
        # administrativa únicamente al cruzar una operación de sistema.
        # El centro multimedia usa PipeWire/WirePlumber como única política de audio.
        security.rtkit.enable = true;

        services.pipewire = {
          enable = true;
          alsa.enable = true;
          pulse.enable = true;
          wireplumber.enable = true;
        };

        security.polkit.enable = true;

        services.fwupd.enable = true;

        # Korunix controla explícitamente cuándo consulta actualizaciones de
        # firmware para no introducir autorizaciones inesperadas al activar NixOS.
        systemd.services.fwupd-refresh.enable = false;
        systemd.timers.fwupd-refresh.enable = false;

        # Dependencias temporales del detector Bash. El futuro binario de
        # Korunix absorberá esta lógica.
        environment.systemPackages = [
          pkgs.pipewire
          pkgs.wireplumber
          pkgs.pulseaudio
          pkgs.v4l-utils
          pkgs.ffmpeg
          pkgs.python3
          pkgs.fwupd
          pkgs.polkit
          pkgs.pciutils
          pkgs.jq
        ];
      }

      (lib.mkIf (pkgs.stdenv.hostPlatform.system == "x86_64-linux") {
        hardware.graphics.enable32Bit = true;
      })

      (lib.mkIf hasNvidia {
        services.xserver.videoDrivers = ["nvidia"];

        hardware.nvidia = {
          modesetting.enable = true;
          open = nvidiaOpenSupported;
        };
      })
    ]
  );
}

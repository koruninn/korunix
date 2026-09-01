{
  config,
  lib,
  pkgs,
  ...
}: let
  cfg = config.korunix.services;
  productDefaults = import ./predeterminados.nix;

  serviceOption = name: description:
    lib.mkOption {
      type = lib.types.bool;
      default = productDefaults.services.${name};
      inherit description;
    };
in {
  options.korunix.services = {
    avahi = serviceOption "avahi" "Descubrimiento de dispositivos en la red local.";
    bluetooth = serviceOption "bluetooth" "Bluetooth.";
    # Compatibilidad de lectura para configuraciones anteriores. El valor ya
    # no gobierna el servicio: SSH es una capacidad permanente de Korunix.
    ssh = lib.mkOption {
      type = lib.types.bool;
      default = true;
      visible = false;
      description = "Campo histórico conservado solo para migración; SSH permanece activo.";
    };
    sunshine = serviceOption "sunshine" "Transmisión de juegos y escritorio con Sunshine.";
    printing = serviceOption "printing" "Impresión y escaneo.";
    virtualization = serviceOption "virtualization" "Máquinas virtuales.";

    printingDriver = lib.mkOption {
      type = lib.types.nullOr (lib.types.enum [
        "epson-201207w"
      ]);
      default = productDefaults.services.printingDriver;
      description = "Controlador adicional que necesita la impresora de este equipo.";
    };
  };

  config = lib.mkIf config.korunix.enable {
    networking.networkmanager.enable = true;

    # El firewall siempre permanece activo. Las excepciones se derivan de las
    # capacidades habilitadas para que nadie tenga que administrar puertos a mano.
    networking.firewall = {
      enable = true;
    };

    services.pulseaudio.enable = false;
    security.rtkit.enable = true;

    services.pipewire = {
      enable = true;
      alsa.enable = true;
      alsa.support32Bit = true;
      pulse.enable = true;
    };

    services.avahi = {
      enable = cfg.avahi;
      openFirewall = cfg.avahi;
    };

    hardware.bluetooth.enable = cfg.bluetooth;

    # xpadneo es una integración preventiva de bajo coste cuando Bluetooth forma
    # parte del equipo; evita pedir otra decisión al conectar un mando Xbox luego.
    hardware.xpadneo.enable = cfg.bluetooth;

    # SSH forma parte permanente de Korunix. No depende de una preferencia
    # del host: el firewall sigue activo y abre únicamente la regla de OpenSSH.
    services.openssh = {
      enable = true;
      openFirewall = true;
    };

    services.sunshine = {
      enable = cfg.sunshine;
      openFirewall = cfg.sunshine;
      autoStart = cfg.sunshine;
      capSysAdmin = cfg.sunshine;
    };

    services.power-profiles-daemon.enable = true;
    services.upower.enable = true;

    services.printing = {
      enable = cfg.printing;

      drivers =
        lib.optionals
        (cfg.printing && cfg.printingDriver == "epson-201207w")
        [pkgs.epson_201207w];
    };

    hardware.sane.enable = cfg.printing;

    programs.virt-manager.enable = cfg.virtualization;
    virtualisation.libvirtd.enable = cfg.virtualization;

    systemd.services.libvirt-default-network = lib.mkIf cfg.virtualization {
      description = "Activa la red predeterminada de las máquinas virtuales";

      wantedBy = [
        "multi-user.target"
      ];

      after = [
        "libvirtd.service"
      ];

      serviceConfig = {
        Type = "oneshot";
        RemainAfterExit = true;
      };

      script = ''
        ${pkgs.libvirt}/bin/virsh net-autostart default

        if ! ${pkgs.libvirt}/bin/virsh net-info default \
          | grep -q "Active:.*yes"; then
          ${pkgs.libvirt}/bin/virsh net-start default
        fi
      '';
    };
  };
}

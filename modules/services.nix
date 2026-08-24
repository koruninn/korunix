{
  config,
  lib,
  pkgs,
  ...
}: let
  cfg = config.korunix.services;
in {
  options.korunix.services = {
    avahi = lib.mkEnableOption "descubrimiento de dispositivos en la red local";
    bluetooth = lib.mkEnableOption "Bluetooth";
    ssh = lib.mkEnableOption "acceso remoto mediante SSH";
    sunshine = lib.mkEnableOption "transmisión de juegos y escritorio con Sunshine";
    printing = lib.mkEnableOption "impresión y escaneo";
    virtualization = lib.mkEnableOption "máquinas virtuales";

    printingDriver = lib.mkOption {
      type = lib.types.nullOr (lib.types.enum [
        "epson-201207w"
      ]);
      default = null;
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

    services.openssh = {
      enable = cfg.ssh;
      openFirewall = cfg.ssh;
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

{
  config,
  lib,
  modulesPath,
  ...
}: {
  imports = [
    (modulesPath + "/installer/scan/not-detected.nix")
  ];

  # Esta salida de vídeo es un hecho detectado de este equipo.
  # La resolución y los Hz se eligen en configuracion.toml.
  _module.args.salidaMonitor = "DP-1";

  # Este disco fue detectado en este equipo. La persona lo reconoce por
  # modelo y capacidad; UUID y ruta siguen siendo detalles técnicos.
  _module.args.unidadesDetectadas = {
    "ST3500413AS · 500 GB" = {
      uuid = "036F8E656FF00FB2";
      sistemaArchivos = "ntfs";
      ruta = "/mnt/datos";
      modelo = "ST3500413AS";
      capacidad = "500 GB";
      transporte = "SATA";
      uid = 1000;
      gid = 100;
    };
  };

  boot.initrd.availableKernelModules = ["xhci_pci" "ahci" "nvme" "usbhid" "usb_storage" "sd_mod"];
  boot.initrd.kernelModules = [];
  boot.kernelModules = [];
  boot.extraModulePackages = [];

  # Estas UUID apuntan a las particiones que ya usa esta computadora.
  fileSystems."/" = {
    device = "/dev/disk/by-uuid/634cf87a-50e4-4f66-ac22-dbcea3f71ae7";
    fsType = "btrfs";
  };

  fileSystems."/home" = {
    device = "/dev/disk/by-uuid/634cf87a-50e4-4f66-ac22-dbcea3f71ae7";
    fsType = "btrfs";
    options = ["subvol=home"];
  };

  fileSystems."/nix" = {
    device = "/dev/disk/by-uuid/634cf87a-50e4-4f66-ac22-dbcea3f71ae7";
    fsType = "btrfs";
    options = ["subvol=nix"];
  };

  fileSystems."/boot" = {
    device = "/dev/disk/by-uuid/8AC8-4004";
    fsType = "vfat";
    options = ["fmask=0077" "dmask=0077"];
  };

  swapDevices = [
    {device = "/dev/disk/by-uuid/8c5c67e0-1a31-4868-b5ed-0e022f102e59";}
  ];

  # Este equipo arranca con UEFI, así que usa systemd-boot.
  boot.loader.systemd-boot.enable = true;
  boot.loader.efi.canTouchEfiVariables = true;

  nixpkgs.hostPlatform = lib.mkDefault "x86_64-linux";
  hardware.cpu.amd.updateMicrocode =
    lib.mkDefault config.hardware.enableRedistributableFirmware;
}

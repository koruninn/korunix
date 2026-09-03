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

  # Estos datos describen la unidad que Korunix conoce como «datos».
  # La persona solo decide en configuracion.toml si quiere tenerla disponible.
  _module.args.unidadesDetectadas = {
    datos = {
      uuid = "036F8E656FF00FB2";
      sistemaArchivos = "ntfs";
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

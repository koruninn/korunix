# Hardware de la ROG Ally usada como equipo principal de Korunix.
# Basado en hardware-configuration.nix generado por NixOS en la Ally.
# Se conservan los ajustes declarativos específicos que ya tenía Veskalia.

{
  config,
  lib,
  pkgs,
  modulesPath,
  ...
}: {
  imports = [
    (modulesPath + "/installer/scan/not-detected.nix")
  ];

  # Módulos de almacenamiento, USB y lector SD detectados en la Ally.
  boot.initrd.availableKernelModules = [
    "nvme"
    "xhci_pci"
    "usbhid"
    "usb_storage"
    "sd_mod"
    "sdhci_pci"
  ];
  boot.initrd.kernelModules = [];
  boot.kernelModules = ["kvm-amd"];
  boot.extraModulePackages = [];

  # Particiones de la ROG Ally: raíz, arranque y swap.
  fileSystems."/" = {
    device = "/dev/disk/by-uuid/52c21821-6e98-4b20-8fb3-b12bf9ccf3d4";
    fsType = "ext4";
  };

  fileSystems."/boot" = {
    device = "/dev/disk/by-uuid/D8FA-E23C";
    fsType = "vfat";
    options = ["fmask=0077" "dmask=0077"];
  };

  swapDevices = [
    {device = "/dev/disk/by-uuid/dbe7a991-c4bc-40b4-9a48-ddf04a084ac9";}
  ];

  # Se mantienen los valores por defecto y la actualización de microcódigo AMD.
  nixpkgs.hostPlatform = lib.mkDefault "x86_64-linux";
  hardware.cpu.amd.updateMicrocode = lib.mkDefault config.hardware.enableRedistributableFirmware;
}

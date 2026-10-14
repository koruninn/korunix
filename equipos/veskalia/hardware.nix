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
    device = "/dev/disk/by-uuid/4472e6df-6d0e-433a-bab2-6282ede221de";
    fsType = "ext4";
  };

  fileSystems."/boot" = {
    device = "/dev/disk/by-uuid/87D4-BC01";
    fsType = "vfat";
    options = ["fmask=0077" "dmask=0077"];
  };

  swapDevices = [
    {device = "/dev/disk/by-uuid/968c833a-c1e4-44cb-8aa5-66c0802bd60f";}
  ];

  # Se mantienen los valores por defecto y la actualización de microcódigo AMD.
  nixpkgs.hostPlatform = lib.mkDefault "x86_64-linux";
  hardware.cpu.amd.updateMicrocode = lib.mkDefault config.hardware.enableRedistributableFirmware;
}

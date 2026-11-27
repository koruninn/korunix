# Hardware de Veskalia, generado inicialmente por NixOS.

{
  config,
  inputs,
  lib,
  pkgs,
  modulesPath,
  ...
}: {
  imports = [
    inputs.nixos-hardware.nixosModules.asus-ally-rc71l
    (modulesPath + "/installer/scan/not-detected.nix")
  ];

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

  # OpenCL para DaVinci Resolve.
  hardware.graphics = {
    enable = true;
    enable32Bit = true;
    extraPackages = with pkgs; [
      mesa.opencl
    ];
  };

  environment.variables = {
    RUSTICL_ENABLE = "radeonsi";
  };

  # Cambia estos UUID si instalas Korunix en otro equipo.
  fileSystems."/" = {
    device = "/dev/disk/by-uuid/cc77e3b3-d868-430b-9cc8-f1226bf21e4b";
    fsType = "ext4";
  };

  fileSystems."/boot" = {
    device = "/dev/disk/by-uuid/20CA-6C51";
    fsType = "vfat";
    options = ["fmask=0077" "dmask=0077"];
  };

  swapDevices = [
    {device = "/dev/disk/by-uuid/09d76aed-037b-482f-8691-948ee77740ab";}
  ];

  nixpkgs.hostPlatform = lib.mkDefault "x86_64-linux";
  hardware.cpu.amd.updateMicrocode = lib.mkDefault config.hardware.enableRedistributableFirmware;
}

# Hardware detectado por NixOS y adoptado por Korunix.
# Este archivo describe dispositivos, discos y módulos detectados para este equipo.
# Korunix no lo regenera silenciosamente y una actualización nunca cambia su
# contenido sin mostrar antes qué hardware pretende adoptar.
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

  # Tipo de firmware detectado físicamente al adoptar este equipo. El módulo de
  # arranque lo consume para elegir systemd-boot o GRUB sin preguntarlo al usuario.
  korunix.hardware.firmware = "uefi";

  # Adaptador gráfico detectado en este equipo. boot_vga identifica la GPU
  # primaria, pero no basta para afirmar si es integrada o dedicada.
  korunix.hardware.graphics = [
    {
      pciAddress = "0000:05:00.0";
      name = "AMD Cezanne [Radeon Vega Series / Radeon Vega Mobile Series]";
      vendor = "amd";
      vendorId = "1002";
      deviceId = "1638";
      subsystemVendorId = "1458";
      subsystemDeviceId = "d000";
      driver = "amdgpu";
      primary = true;
      kind = "unknown";
      nvidiaOpen = false;
    }
  ];

  boot.initrd.availableKernelModules = ["xhci_pci" "ahci" "nvme" "usbhid" "usb_storage" "sd_mod"];
  boot.initrd.kernelModules = [];
  boot.kernelModules = [];
  boot.extraModulePackages = [];

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

  nixpkgs.hostPlatform = lib.mkDefault "x86_64-linux";
  hardware.cpu.amd.updateMicrocode = lib.mkDefault config.hardware.enableRedistributableFirmware;
}

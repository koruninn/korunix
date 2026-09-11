{
  config,
  pkgs,
  lib,
  ...
}: {
# Bootloader.
  boot.loader.grub = {
  enable = true;
  device = "/dev/sda";
  useOSProber = true;
};

   # Use latest kernel.
  boot.kernelPackages = pkgs.linuxPackages_latest;

    # Boot silencioso estilo consola/Steam Deck
  boot.kernelParams = ["quiet" "splash" "boot.shell_on_fail"];
  boot.plymouth.enable = true;
};


{
  config,
  pkgs,
  lib,
  ...
}: {
  boot.loader.systemd-boot = {
    enable = true;
    configurationLimit = 10;
  };

  boot.loader.efi.canTouchEfiVariables = true;
  boot.loader.systemd-boot.edk2-uefi-shell.enable = true;

  boot.kernelPackages = pkgs.linuxPackages_latest;
  # Ocultar los errores ACPI en el arranque, pero conservarlos en el registro.
  # quiet debe ir antes del nivel de mensajes de consola.
  boot.consoleLogLevel = 3;
  boot.kernelParams = lib.mkBefore ["quiet" "splash" "boot.shell_on_fail"];
  boot.plymouth.enable = true;

}

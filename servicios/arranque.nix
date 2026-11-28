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

  # Usar el kernel de soporte prolongado.
  boot.kernelPackages = pkgs.linuxPackages;
  # Mantener limpio el arranque sin perder el registro de errores.
  boot.consoleLogLevel = 3;
  boot.kernelParams = lib.mkBefore ["quiet" "splash" "boot.shell_on_fail"];
  boot.plymouth.enable = true;

}

{
  config,
  pkgs,
  lib,
  ...
}: {
  # Systemd-boot
  boot.loader.systemd-boot = {
    enable = true;
    # Limita el número de generaciones de NixOS en el menú para no saturar la pantalla
    configurationLimit = 10;
  };

  # Permitimos la modificación de variables EFI (obligatorio para systemd-boot)
  boot.loader.efi.canTouchEfiVariables = true;
  
  # Opcional: Si Windows está en un disco/partición EFI distinta y systemd-boot no lo ve de forma nativa,
  # esto añade una herramienta en el menú para cargar la shell UEFI y arrancar cualquier OS.
  boot.loader.systemd-boot.edk2-uefi-shell.enable = true;

  # Usar el kernel más reciente
  boot.kernelPackages = pkgs.linuxPackages_latest;

  # Boot silencioso estilo consola/Steam Deck
  boot.kernelParams = ["quiet" "splash" "boot.shell_on_fail"];
  boot.plymouth.enable = true;

  # Configuración del Display Manager
  services.displayManager = {
    defaultSession = lib.mkForce "niri"; 
  };
}


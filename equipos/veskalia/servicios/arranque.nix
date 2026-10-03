{
  config,
  pkgs,
  lib,
  ...
}: {
  # El menú de arranque de este equipo usa systemd-boot.
  boot.loader.systemd-boot = {
    enable = true;
    # Conserva solo las últimas 10 generaciones para que el menú no se llene de versiones antiguas.
    configurationLimit = 10;
  };

  # Deja que systemd-boot pueda actualizar las variables de arranque guardadas en la memoria EFI.
  boot.loader.efi.canTouchEfiVariables = true;

  # Si hay otro equipo en una partición EFI que systemd-boot no detecta solo,
  # esta opción añade una herramienta para abrir la shell UEFI y arrancarlo a mano.
  boot.loader.systemd-boot.edk2-uefi-shell.enable = true;

  # Usa la versión más nueva del kernel que esté disponible en este canal del equipo.
  boot.kernelPackages = pkgs.linuxPackages_latest;

  # Esconde la mayor parte de los mensajes durante el arranque y deja Plymouth como pantalla de inicio.
  boot.kernelParams = ["quiet" "splash" "boot.shell_on_fail"];
  boot.plymouth.enable = true;

  # Hace que Umbriel sea la sesión gráfica que se inicia por defecto después de iniciar sesión.
  services.displayManager = {
    defaultSession = lib.mkForce "umbriel";
  };
}

{
  config,
  pkgs,
  lib,
  ...
}: {
  # Este equipo usa GRUB como menú de inicio y puede detectar otros sistemas instalados.
  boot.loader.grub = {
    enable = true;
    device = "/dev/sda";
    useOSProber = true;
  };

  # Usa la versión más nueva del kernel que esté disponible en este canal del equipo.
  boot.kernelPackages = pkgs.linuxPackages_latest;

  # Esconde la mayor parte de los mensajes durante el arranque y deja Plymouth como pantalla de inicio.
  boot.kernelParams = ["quiet" "splash" "boot.shell_on_fail"];
  boot.plymouth.enable = true;
}

{pkgs, ...}: {
  # Permite abrir programas que no vienen instalados como paquetes normales.
  # Esto ayuda con apps portables y con archivos AppImage.
  programs.nix-ld.enable = true;

  # AppImage es la forma más simple de lanzar programas portables sin dejar
  # de usar la configuración declarativa de NixOS.
  environment.systemPackages = with pkgs; [
    appimage-run
  ];
}

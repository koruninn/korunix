{
  config,
  pkgs,
  ...
}: {
  # Habilitamos Limine
  boot.loader.limine = {
    enable = true;
    # Inyecta la entrada de Windows directamente en el limine.conf generado por NixOS
    extraConfig = ''
        timeout: 5
    '';
  };

  # Permitimos la modificación de variables EFI (recomendado para instalaciones UEFI)
  boot.loader.efi.canTouchEfiVariables = true;

  # Usar el kernel más reciente
  boot.kernelPackages = pkgs.linuxPackages_latest;

  # Boot silencioso estilo consola/Steam Deck (Oculta las letras de carga)
  boot.kernelParams = ["quiet" "splash" "boot.shell_on_fail"];
  boot.plymouth.enable = true;

  services.displayManager = {
    # Esto fuerza a GDM a seleccionar Niri por defecto al encender
    defaultSession = "niri";
  };

  services.xserver = {
    enable = true;
    displayManager.gdm = {
      enable = true;
    };
  };
}

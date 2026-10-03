{pkgs, ...}: {
  # El nombre de este equipo sale solo del nombre de la carpeta donde está la ajustes.
  networking.networkmanager.enable = true;

  # Avahi deja ver este equipo y otros servicios compatibles dentro de la red local.
  services.avahi = {
    enable = true;
    openFirewall = true;
  };

  # Bluetooth deja conectar mandos, audífonos y otros dispositivos sin cables.
  hardware.bluetooth.enable = true;

  # SSH deja entrar a este equipo desde otro aparato de forma remota.
  services.openssh = {
    enable = true;
    openFirewall = true;
  };

  # Tailscale crea una red privada para acceder a Korunix desde fuera de casa sin exponer directamente
  # los servicios a Internet ni tener que configurar a mano el reenvío de puertos del router.
  services.tailscale.enable = true;
  environment.systemPackages = [pkgs.trayscale];

  # Sunshine deja transmitir el escritorio y los juegos a otros dispositivos de la red o de Tailscale.
  services.sunshine = {
    enable = true;
    openFirewall = true;
    autoStart = true;
    capSysAdmin = true;
  };

  networking.firewall = rec {
    allowedTCPPortRanges = [
      {
        from = 1714;
        to = 1764;
      }
    ];
    allowedUDPPortRanges = allowedTCPPortRanges;
  };

  networking.firewall.enable = true;
}

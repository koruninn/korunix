{...}: {
  # El hostname se deriva del nombre del equipo en flake.nix.
  networking.networkmanager.enable = true;

  # Avahi
  services.avahi = {
    enable = true;
    openFirewall = true;
  };

  # SSH
  services.openssh = {
    enable = true;
    openFirewall = true;
  };

  # LocalSend abre declarativamente su propio puerto. No mantenemos rangos
  # adicionales para servicios que este equipo no instala.
  networking.firewall.enable = true;
}

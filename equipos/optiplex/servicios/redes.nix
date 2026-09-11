{
  config,
  pkgs,
  ...
}: {
  # Hostname
  networking.hostName = "optiplex";
  # Activar redes
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

{...}: {
  # El hostname se deriva del nombre del equipo en flake.nix.
  networking.networkmanager.enable = true;

  # Avahi
  services.avahi = {
    enable = true;
    openFirewall = true;
  };

  # Bluetooth
  hardware.bluetooth.enable = true;

  # SSH
  services.openssh = {
    enable = true;
    openFirewall = true;
  };

  # Sunshine
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

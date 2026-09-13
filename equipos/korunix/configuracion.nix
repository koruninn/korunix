{
  pkgs,
  ...
}:

{
  time.timeZone = "America/Lima";
  i18n.defaultLocale = "es_PE.UTF-8";
  services.printing.enable = true;

  # Ajustes exclusivos de este equipo: monitor.
  environment.etc."niri/monitor.kdl".text = ''
    output "DP-1" {
        mode "1920x1080@120.000"
        scale 1
    }
  '';

  nixpkgs.config.allowUnfree = true;
  environment.systemPackages = [ pkgs.alejandra ];

  system.stateVersion = "26.05";
  nix.settings.experimental-features = [ "nix-command" "flakes" ];
}

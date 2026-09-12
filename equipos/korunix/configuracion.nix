{ ... }:

{
  time.timeZone = "America/Lima";
  i18n.defaultLocale = "es_PE.UTF-8";
  services.printing.enable = true;

  users.users."koru" = {
    isNormalUser = true;
    description = "André";
    extraGroups = [ "networkmanager" "wheel" ];
  };

  # Ajustes exclusivos de este equipo: monitor y dotfiles de koru.
  environment.etc."niri/monitor.kdl".text = ''
    output "DP-1" {
        mode "1920x1080@120.000"
        scale 1
    }
  '';
  systemd.tmpfiles.rules = [
    "L+ /home/koru/.config/fish/config.fish - - - - /home/koru/.korunix/config.fish"
  ];

  nixpkgs.config.allowUnfree = true;

  system.stateVersion = "26.05";
  nix.settings.experimental-features = [ "nix-command" "flakes" ];
}

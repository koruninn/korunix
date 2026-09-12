{ pkgs, ... }:

{
  networking.networkmanager.enable = true;

  time.timeZone = "America/Lima";
  i18n.defaultLocale = "es_PE.UTF-8";

  services.xserver.enable = true;
  services.xserver.displayManager.lightdm.enable = true;
  services.displayManager.autoLogin = {
    enable = true;
    user = "dell";
  };

  services.xserver.desktopManager.cinnamon.enable = true;

  services.xserver.xkb = {
    layout = "es";
    variant = "deadtilde";
  };

  console.keyMap = "es";

  services.pulseaudio.enable = false;
  security.rtkit.enable = true;
  services.pipewire = {
    enable = true;
    alsa.enable = true;
    alsa.support32Bit = true;
    pulse.enable = true;
  };

  programs.firefox.enable = false;
  nixpkgs.config.allowUnfree = true;

  environment.systemPackages = with pkgs; [
    (google-chrome.override {
      commandLineArgs = "--password-store=basic";
    })
  ];

  system.stateVersion = "26.05";
  nix.settings.experimental-features = [ "nix-command" "flakes" ];
}

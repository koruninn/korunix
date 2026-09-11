# Edit this configuration file to define what should be installed on
# your system.  Help is available in the configuration.nix(5) man page
# and in the NixOS manual (accessible by running ‘nixos-help’).

{ config, pkgs, ... }:

{
  imports =
    [ # Include the results of the hardware scan.
      ./hardware.nix
    ];

  networking.hostName = "optiplex"; # Define your hostname.
  # networking.wireless.enable = true;  # Enables wireless support via wpa_supplicant.

  # Enable networking
  networking.networkmanager.enable = true;

  # Set your time zone.
  time.timeZone = "America/Lima";

  # Select internationalisation properties.
  i18n.defaultLocale = "es_PE.UTF-8";

  # Enable the X11 windowing system.
  services.xserver.enable = true;

# Enable LightDM Display Manager and Autologin
  services.xserver.displayManager.lightdm.enable = true;
  services.displayManager.autoLogin = {
    enable = true;
    user = "dell";
  };

  # Enable the Cinnamon Desktop Environment.
  services.xserver.desktopManager.cinnamon.enable = true;

  # Configure keymap in X11
  services.xserver.xkb = {
    layout = "es";
    variant = "deadtilde";
  };

  # Configure console keymap
  console.keyMap = "es";

  # Enable sound with pipewire.
  services.pulseaudio.enable = false;
  security.rtkit.enable = true;
  services.pipewire = {
    enable = true;
    alsa.enable = true;
    alsa.support32Bit = true;
    pulse.enable = true;
  };

  # Install firefox.
  programs.firefox.enable = false;

  # Allow unfree packages
  nixpkgs.config.allowUnfree = true;

  # List packages installed in system profile. To search, run:
  # $ nix search wget
  environment.systemPackages = with pkgs; [
    (google-chrome.override {
    commandLineArgs = "--password-store=basic";
  })
  ];

  system.stateVersion = "26.05"; # Did you read the comment?

}

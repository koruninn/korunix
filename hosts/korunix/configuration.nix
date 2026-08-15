{
  config,
  pkgs,
  inputs,
  ...
}: {
  imports = [
    ../../modules/nixos/apps
    ../../modules/nixos/services
    ./hardware-configuration.nix
  ];

  # Set your time zone.
  time.timeZone = "America/Lima";

  # Select internationalisation properties.
  i18n.defaultLocale = "es_PE.UTF-8";

  # Habilitar Fish a nivel de sistema para que genere las estructuras necesarias
  programs.fish.enable = true;

  # Noctalia cache
  nix.settings = {
    extra-substituters = ["https://noctalia.cachix.org"];
    extra-trusted-public-keys = ["noctalia.cachix.org-1:pCOR47nnMEo5thcxNDtzWpOxNFQsBRglJzxWPp3dkU4="];
  };

  nixpkgs.config.permittedInsecurePackages = [
    "electron-40.10.5"
  ];

  nixpkgs.overlays = [inputs.millennium.overlays.default];

  # Define a user account.
  users.users."koru" = {
    isNormalUser = true;
    description = "André";
    extraGroups = ["networkmanager" "Libvirtd" "kvm" "input" "uinput" "wheel" "lp" "scanner" "adbusers"];
    shell = pkgs.fish;
  };

  # Allow unfree packages
  nixpkgs.config.allowUnfree = true;

  # Enable appimages
  programs.appimage = {
    enable = true;
    binfmt = true;
  };

  hardware.xpadneo.enable = true;

  # Enable 32 bit support
  hardware.graphics = {
    enable = true;
    enable32Bit = true;
  };

  programs.virt-manager.enable = true;

  virtualisation.libvirtd.enable = true;

  systemd.services.libvirt-default-network = {
    description = "Activate libvirt default network";

    wantedBy = ["multi-user.target"];
    after = ["libvirtd.service"];

    serviceConfig = {
      Type = "oneshot";
      RemainAfterExit = true;
    };

    script = ''
      ${pkgs.libvirt}/bin/virsh net-autostart default

      if ! ${pkgs.libvirt}/bin/virsh net-info default \
        | grep -q "Active:.*yes"; then
        ${pkgs.libvirt}/bin/virsh net-start default
      fi
    '';
  };

  system.stateVersion = "26.05";

  nix.settings.experimental-features = ["nix-command" "flakes"];
}

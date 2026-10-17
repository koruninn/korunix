{
  lib,
  pkgs,
  ...
}: {
  time.timeZone = "America/Lima";
  i18n.defaultLocale = "es_PE.UTF-8";

  services.printing.enable = true;
  nixpkgs.config.allowUnfree = true;

  # Cobertura amplia de idiomas y sistemas de escritura.
  fonts = {
    enableDefaultPackages = true;
    packages = with pkgs; [
      noto-fonts
    ];
  };

  environment.systemPackages = [
    pkgs.alejandra
  ];

  programs.ssh.askPassword =
    lib.mkForce "${pkgs.seahorse}/libexec/seahorse/ssh-askpass";

  system.stateVersion = "26.05";
}

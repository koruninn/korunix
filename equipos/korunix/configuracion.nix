{
  pkgs,
  ...
}: {
  time.timeZone = "America/Lima";
  i18n.defaultLocale = "es_PE.UTF-8";
  services.printing.enable = true;

  nixpkgs.config.allowUnfree = true;
  environment.systemPackages = [pkgs.alejandra];

  system.stateVersion = "26.05";
}

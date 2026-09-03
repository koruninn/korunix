{
  aplicaciones,
  nombre,
  programa,
  ...
}: {
  imports = [./hardware.nix];

  networking.hostName = nombre;
  networking.networkmanager.enable = true;

  nix.settings.experimental-features = ["nix-command" "flakes"];
  nixpkgs.config.allowUnfree = true;

  environment.systemPackages = aplicaciones ++ [programa];

  # Conserva la compatibilidad de la instalación actual.
  system.stateVersion = "26.05";
}

{
  config,
  pkgs,
  inputs,
  ...
}: {
  # Importamos el módulo de nix-flatpak directamente aquí
  imports = [
    inputs.nix-flatpak.nixosModules.nix-flatpak
  ];

  # Habilitamos Flatpak en el sistema.
  services.flatpak.enable = true;

  # Aplicaciones instaladas declarativamente desde Flathub.
  services.flatpak.packages = [
    "io.github.brunofin.Cohesion"
    "net.nokyan.Resources"
  ];

  services.flatpak.update.auto = {
    enable = true;
    onCalendar = "weekly";
  };
}

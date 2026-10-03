{inputs, ...}: {
  # Traemos aquí el bloque de nix-flatpak para usarlo en el sistema.
  imports = [
    inputs.nix-flatpak.nixosModules.nix-flatpak
  ];

  # Activamos Flatpak en el equipo.
  services.flatpak.enable = true;

  # Aquí dejamos instaladas las apps que queremos desde Flathub.
  services.flatpak.packages = [
    "io.github.brunofin.Cohesion"
  ];

  services.flatpak.update.auto = {
    enable = true;
    onCalendar = "weekly";
  };
}

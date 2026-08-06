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

  # Habilitamos el demonio de Flatpak en el sistema
  services.flatpak.enable = true;

  # Declaramos los paquetes que queremos instalar desde Flathub
  services.flatpak.packages = [
    "io.github.DraqueT.PolyGlot"
    "io.github.brunofin.Cohesion"
  ];

  # Opcional: Para asegurar que los repositorios de Flathub se actualicen solos
  services.flatpak.update.auto = {
    enable = true;
    onCalendar = "weekly"; # Puedes cambiarlo a "daily"
  };
}

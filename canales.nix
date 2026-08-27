# Canales de actualización disponibles para Korunix.
#
# Esta es una fuente declarativa de Nix. La elección concreta sigue viviendo
# en cada equipo y system.stateVersion permanece independiente.
{
  schemaVersion = 1;
  default = "stable";

  channels = {
    stable = {
      label = "Estable";
      description = "Prioriza estabilidad y cambios menos frecuentes.";
      nixpkgs_ref = "nixos-26.05";
      aagl_ref = "release-26.05";
      label_en = "Stable";
      description_en = "Prioritizes stability and less frequent changes.";
      label_hu = "Stabil";
      description_hu = "A stabilitást és a ritkább változásokat részesíti előnyben.";
    };

    unstable = {
      label = "Inestable";
      description = "Prioriza software reciente y cambios más frecuentes.";
      nixpkgs_ref = "nixos-unstable";
      aagl_ref = "main";
      label_en = "Unstable";
      description_en = "Prioritizes newer software and more frequent changes.";
      label_hu = "Instabil";
      description_hu = "Az újabb szoftvereket és a gyakoribb változásokat részesíti előnyben.";
    };
  };
}

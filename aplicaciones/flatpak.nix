{inputs, ...}: {
  imports = [
    inputs.nix-flatpak.nixosModules.nix-flatpak
  ];

  services.flatpak.enable = true;

  services.flatpak.packages = [
    "io.github.brunofin.Cohesion" # Sin paquete en Nixpkgs.
    "io.github.DraqueT.PolyGlot" # La versión nativa no funciona adecuadamente.
    "sh.cider.Cider" # El paquete nativo está desactualizado.
  ];

  services.flatpak.update.auto = {
    enable = true;
    onCalendar = "weekly";
  };
}

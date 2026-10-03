{pkgs, ...}: {
  # Esta es la ajustes de cursor que usa Korunix como base en todo el equipo.
  # Aunque cada escritorio puede tener sus propios ajustes, las aplicaciones parten de esta misma ajustes.
  environment.systemPackages = [pkgs.bibata-cursors];

  environment.variables = {
    XCURSOR_THEME = "Bibata-Modern-Classic";
    XCURSOR_SIZE = "24";
  };
}

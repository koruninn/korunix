{pkgs, ...}: {
  # Cursor común de Korunix. Los compositores pueden tener además su ajuste
  # nativo, pero toda la sesión y las aplicaciones heredan esta misma base.
  environment.systemPackages = [pkgs.bibata-cursors];

  environment.variables = {
    XCURSOR_THEME = "Bibata-Modern-Ice";
    XCURSOR_SIZE = "24";
  };
}

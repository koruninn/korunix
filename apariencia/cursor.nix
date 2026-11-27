{pkgs, ...}: {
  environment.systemPackages = [pkgs.bibata-cursors];

  environment.variables = {
    XCURSOR_THEME = "Bibata-Modern-Classic";
    XCURSOR_SIZE = "24";
  };
}

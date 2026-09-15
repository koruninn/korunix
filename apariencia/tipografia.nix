{pkgs, ...}: {
  fonts.packages = [pkgs.adwaita-fonts];
  fonts.fontconfig.defaultFonts.sansSerif = ["Adwaita Sans"];
}

{ pkgs, ... }:

{
  # Plasma no debe sincronizar su tema GTK dentro de otras sesiones Wayland.
  # kde-gtk-config es quien exporta preferencias de Plasma a aplicaciones GTK.
  environment.plasma6.excludePackages = [
    pkgs.kdePackages.kde-gtk-config
  ];
}

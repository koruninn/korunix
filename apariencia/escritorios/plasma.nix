{pkgs, ...}: let
  materialYou = pkgs.python3Packages."kde-material-you-colors";
in {
  # Plasma 6 únicamente como sesión Wayland.
  services.desktopManager.plasma6.enable = true;

  environment.plasma6.excludePackages = [
    pkgs.kdePackages.kwin-x11
  ];

  # Colores dinámicos propios de Plasma, independientes de Noctalia.
  environment.systemPackages = [
    materialYou
  ];

  environment.etc."xdg/autostart/kde-material-you-colors.desktop".text = ''
[Desktop Entry]
Type=Application
Name=KDE Material You Colors
Comment=Colores dinámicos de Plasma a partir del fondo
Exec=${materialYou}/bin/kde-material-you-colors
Icon=color-management
OnlyShowIn=KDE;
X-KDE-AutostartScript=true
  '';

  # Num Lock encendido al iniciar Plasma.
  environment.etc."xdg/kcminputrc".text = ''
[Keyboard]
NumLock=0
  '';

  # Plasma no debe copiar su apariencia hacia GTK.
  environment.etc."xdg/kded5rc".text = ''
[Module-gtkconfig]
autoload=false
  '';

  # Plasma usa Dolphin como gestor de archivos sin cambiar Umbriel.
  environment.etc."xdg/kde-mimeapps.list".text = ''
[Default Applications]
inode/directory=org.kde.dolphin.desktop;
  '';

  # Orden declarativo de los lanzadores fijados del panel de Plasma.
  environment.etc."xdg/plasma-org.kde.plasma.desktop-appletsrc".text = ''
[Containments][2][Applets][5][Configuration][General]
launchers=applications:systemsettings.desktop,applications:org.kde.dolphin.desktop,applications:zen.desktop
  '';

  environment.variables = {
    XKB_DEFAULT_LAYOUT = "es";
    XKB_DEFAULT_VARIANT = "deadtilde";
  };
}

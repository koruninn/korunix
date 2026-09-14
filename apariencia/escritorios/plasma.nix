{pkgs, ...}: {
  # Plasma 6 únicamente como sesión Wayland.
  services.desktopManager.plasma6.enable = true;

  environment.plasma6.excludePackages = [
    pkgs.kdePackages.kwin-x11
  ];

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

  environment.variables = {
    XKB_DEFAULT_LAYOUT = "es";
    XKB_DEFAULT_VARIANT = "deadtilde";
  };
}

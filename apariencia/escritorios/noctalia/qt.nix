{pkgs, ...}: {
  # Las aplicaciones Qt/KDE de la sesión Niri usan la integración nativa de KDE.
  # Noctalia escribe su KColorScheme en kdeglobals; esta integración hace que
  # Dolphin, Kate, Kdenlive y otras aplicaciones Qt lean esos colores directamente.
  qt.enable = true;

  environment.systemPackages = with pkgs.kdePackages; [
    kio
    plasma-integration
    systemsettings
  ];
}

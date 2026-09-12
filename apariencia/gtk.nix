{ pkgs, ... }: {
  # Tema de iconos global: Hatter para GTK y aplicaciones que usan la especificación XDG.
  environment.systemPackages = [
    pkgs.hatter-icon-theme
  ];

  # GTK mantiene su apariencia independiente de Plasma y Qt.
  # No se instala un kdeglobals global: Plasma debe conservar sus preferencias
  # dentro de su propia sesión y no exportarlas a Niri/GTK.
  environment.etc = {
    "gtk-2.0/gtkrc".text = ''
      gtk-icon-theme-name="Hatter-Slate"
    '';

    "gtk-3.0/settings.ini".text = ''
      [Settings]
      gtk-icon-theme-name=Hatter-Slate
    '';

    "gtk-4.0/settings.ini".text = ''
      [Settings]
      gtk-icon-theme-name=Hatter-Slate
    '';
  };
}

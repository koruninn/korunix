{ pkgs, ... }: {
  # Tema de iconos global: Hatter para GTK y aplicaciones que usan la especificación XDG.
  environment.systemPackages = [
    pkgs.hatter-icon-theme
  ];

  # Mantiene GTK independiente de cualquier estilo Qt.
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

    # Qt puede consultar este archivo para el nombre del tema de iconos sin
    # imponer un estilo Qt sobre GTK.
    "xdg/kdeglobals".text = ''
      [Icons]
      Theme=Hatter-Slate
    '';
  };
}

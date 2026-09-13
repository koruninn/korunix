{ pkgs, ... }: {
  # Noctalia usa adw-gtk3/adw-gtk3-dark para sincronizar GTK 3 con el modo
  # claro u oscuro. Lutris y otras aplicaciones GTK 3 dependen de este tema.
  environment.systemPackages = [
    pkgs.adw-gtk3
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

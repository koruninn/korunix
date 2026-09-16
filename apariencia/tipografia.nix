{pkgs, ...}: let
  compartido = import ./escritorios/compartido.nix;
  aplicarGnome = pkgs.writeShellApplication {
    name = "korunix-tipografia-gnome";
    runtimeInputs = [pkgs.glib];
    text = ''
      gsettings set org.gnome.desktop.interface font-name '${compartido.tipografia} 11'
      gsettings set org.gnome.desktop.interface document-font-name '${compartido.tipografia} 11'
    '';
  };
in {
  fonts.packages = [pkgs.adwaita-fonts];
  fonts.fontconfig.defaultFonts.sansSerif = [compartido.tipografia];

  environment.etc."xdg/autostart/korunix-tipografia-gnome.desktop".text = ''
    [Desktop Entry]
    Type=Application
    Name=Korunix · Tipografía
    Exec=${aplicarGnome}/bin/korunix-tipografia-gnome
    OnlyShowIn=GNOME;
    NoDisplay=true
    X-GNOME-Autostart-enabled=true
  '';
}

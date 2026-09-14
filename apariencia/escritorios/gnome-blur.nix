{pkgs, ...}: let
  blurMyShell = pkgs.gnomeExtensions.blur-my-shell;

  enableBlurMyShell = pkgs.writeShellApplication {
    name = "korunix-gnome-blur-my-shell";
    runtimeInputs = [pkgs.gnome-shell];
    text = ''
      gnome-extensions enable ${blurMyShell.extensionUuid} >/dev/null 2>&1 || true
    '';
  };
in {
  # Blur My Shell queda instalado declarativamente y se habilita al iniciar GNOME.
  # No imponemos aquí parámetros visuales adicionales: la extensión conserva sus
  # valores propios para evitar competir con ChromaLeon y el resto del Shell.
  environment.systemPackages = [
    blurMyShell
    enableBlurMyShell
  ];

  environment.etc."xdg/autostart/korunix-gnome-blur-my-shell.desktop".text = ''
    [Desktop Entry]
    Type=Application
    Name=Korunix · Blur My Shell
    Comment=Habilita Blur My Shell en la sesión GNOME
    Exec=${enableBlurMyShell}/bin/korunix-gnome-blur-my-shell
    OnlyShowIn=GNOME;
    NoDisplay=true
    X-GNOME-Autostart-enabled=true
  '';
}

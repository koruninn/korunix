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
  # Blur My Shell se deja instalado desde los ajustes de Korunix y se activa
  # solo cuando GNOME arranca.
  # No cambiamos aquí su parte visual para que la extensión use sus propios
  # valores y no sobrescriba lo que Korunix ya definió para GNOME.
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

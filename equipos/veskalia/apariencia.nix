{
  lib,
  pkgs,
  ...
}: {
  # Escritorio tradicional: GNOME.
  services.xserver.enable = true;
  services.desktopManager.gnome.enable = true;

  # Teclado español.
  services.xserver.xkb = {
    layout = "es";
    variant = "deadtilde";
  };
  console.keyMap = "es";

  # No competir con el arranque automático de Jovian.
  services.displayManager.gdm.enable = lib.mkForce false;

  # Extensiones y gestor de extensiones.
  environment.systemPackages = with pkgs; [
    gnomeExtensions.dash-to-dock
    gnomeExtensions.blur-my-shell
    gnomeExtensions.alphabetical-app-grid
    gnome-extension-manager
  ];

  programs.dconf.enable = true;
  programs.dconf.profiles.user.databases = [
    {
      settings = {
        "org/gnome/shell" = {
          enabled-extensions = [
            "dash-to-dock@micxgx.gmail.com"
            "blur-my-shell@aunetx"
            "AlphabeticalAppGrid@stuarthayhurst"
            "user-accent-colors@fabito02"
          ];
        };

        # Teclado táctil de GNOME.
        "org/gnome/desktop/a11y/applications" = {
          screen-keyboard-enabled = true;
        };
      };
    }
  ];
}

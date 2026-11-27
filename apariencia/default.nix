{
  lib,
  pkgs,
  ...
}: {
  imports = [
    ./hatter.nix
    ./cursor.nix
    ./gtk.nix
    ./fondos.nix
    ./escritorios/gnome-paleta.nix
  ];

  services.xserver.enable = true;
  services.desktopManager.gnome.enable = true;

  services.xserver.xkb = {
    layout = "es";
    variant = "deadtilde";
  };
  console.keyMap = "es";

  services.displayManager.gdm.enable = true;

  environment.systemPackages = with pkgs; [
    gnomeExtensions.dash-to-dock
    gnomeExtensions.blur-my-shell
    gnomeExtensions.alphabetical-app-grid
    gnomeExtensions.advanced-media-controller
    gnomeExtensions.extension-list
    gnomeExtensions.legacy-gtk3-theme-scheme-auto-switcher
    gnome-extension-manager
  ];

  programs.kdeconnect = {
    enable = true;
    package = pkgs.gnomeExtensions.gsconnect;
  };

  programs.dconf.enable = true;
  programs.dconf.profiles.user.databases = [
    {
      # Mantener los favoritos del repositorio por encima de los guardados en GNOME.
      locks = ["/org/gnome/shell/favorite-apps"];
      settings = {
        "org/gnome/desktop/interface" = {
          icon-theme = "Hatter-Slate";
          cursor-theme = "Bibata-Modern-Classic";
          cursor-size = lib.gvariant.mkInt32 24;
        };

        "org/gnome/shell" = {
          favorite-apps = (import ./escritorios/dock.nix).gnome;
          # Activar las extensiones sin impedir que se desactiven a mano.
          disable-user-extensions = false;
          enabled-extensions = [
            "dash-to-dock@micxgx.gmail.com"
            "blur-my-shell@aunetx"
            "AlphabeticalAppGrid@stuarthayhurst"
            "user-accent-colors@fabito02"
            pkgs.gnomeExtensions.advanced-media-controller.extensionUuid
            pkgs.gnomeExtensions.extension-list.extensionUuid
            pkgs.gnomeExtensions.gsconnect.extensionUuid
            pkgs.gnomeExtensions.legacy-gtk3-theme-scheme-auto-switcher.extensionUuid
          ];
        };

        "org/gnome/settings-daemon/plugins/media-keys" = {
          custom-keybindings = ["/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/korunix-terminal/"];
        };
        "org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/korunix-terminal" = {
          name = "Ptyxis";
          command = "ptyxis";
          binding = "<Control><Alt>t";
        };

        "org/gnome/desktop/a11y/applications" = {
          screen-keyboard-enabled = true;
        };
      };
    }
  ];
}

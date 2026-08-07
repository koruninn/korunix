{
  config,
  pkgs,
  inputs,
  ...
}: let
  spicePkgs = inputs.spicetify-nix.legacyPackages.${pkgs.stdenv.hostPlatform.system};
in {
  nixpkgs.config.allowUnfree = true;
  home.username = "koru";
  home.homeDirectory = "/home/koru";
  home.stateVersion = "25.05";

  imports = [
    inputs.spicetify-nix.homeManagerModules.default
    ../../modules/nixos/apps/fastfetch.nix
    ../../modules/home/window-manager/niri.nix
    ../../modules/home/window-manager/noctalia/noctalia.nix
    ../../modules/nixos/apps/spicetify.nix
  ];

  programs.home-manager.enable = true;

  i18n.inputMethod.enabled = null;

  xdg.configFile."fish/config.fish".source =
    config.lib.file.mkOutOfStoreSymlink "/home/koru/.korunix/config.fish";

  home.file.".local/share/icons/Hatter".source = "${inputs.hatter}/Hatter";

  home.file.".local/share/icons/Hatter-Green".source = "${inputs.hatter}/Hatter-Green";

  # Snippet Noctalia para Obsidian
  home.file.".obsidian/snippets/noctalia.css" = {
    source = config.lib.file.mkOutOfStoreSymlink "/home/koru/.korunix/modules/home/window-manager/noctalia/themes/obsidian/obsidian.css";
    force = true;
  };

  # Tema para Heroic Games Launcher
  home.file.".config/heroic/themes/noctalia.css" = {
    source = config.lib.file.mkOutOfStoreSymlink "/home/koru/.korunix/modules/home/window-manager/noctalia/themes/heroic/noctalia.css";
    force = true;
  };

  # Configuración GTK
  gtk = {
    enable = true;

    iconTheme = {
      name = "Hatter-Green";
    };
    cursorTheme = {
      name = "Bibata-Modern-Classic";
      package = pkgs.bibata-cursors;
    };
  };

  dconf.settings = {
    "org/gnome/desktop/interface" = {
      icon-theme = "Hatter-Green";
    };
  };
}

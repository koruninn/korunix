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

  programs.home-manager = {
    enable = true;
  };

  i18n.inputMethod = {
    enabled = null;
    # Evita que Home Manager intente preconfigurar IBus/Fcitx de forma genérica
  };

  # Enlazamos el archivo de configuración de Fish fuera del /nix/store/
  xdg.configFile."fish/config.fish".source = config.lib.file.mkOutOfStoreSymlink "/home/koru/.korunix/config.fish";

  # CONFIGURACIÓN DEL TEMA DE ICONOS GTK PARA NIRI
  gtk = {
    enable = true;
    iconTheme = {
      name = "Hatter-Green";
      package = pkgs.hatter-icon-theme or null;
    };
  };
}

{
  config,
  pkgs,
  inputs, # Cambiado: pasamos inputs en vez de spicePkgs
  ...
}: let
  spicePkgs = inputs.spicetify-nix.legacyPackages.${pkgs.stdenv.hostPlatform.system};
in {
  programs.spicetify = {
    enable = true;
    enabledExtensions = with spicePkgs.extensions; [
      adblock
      spicyLyrics
      oneko
    ];
    theme = spicePkgs.themes.defaultDynamic;
    #colorScheme = "Everforest";
  };
}

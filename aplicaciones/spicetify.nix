{
  inputs,
  pkgs,
  ...
}: let
  spicePkgs = inputs.spicetify-nix.legacyPackages.${pkgs.stdenv.hostPlatform.system};
  # Lucid Lyrics desactivado
  #
  # lucidLyricsJs = pkgs.fetchurl {
  #   url = "https://lucid-lyrics.sanooj.uk/spice/lucid-lyrics.js";
  #   hash = "sha256-o+sha5aC0gJDYb/NDEHSvdZzvxNhDzVLqzOswdHq6u0";
  # };
  #
  # lucidLyrics = pkgs.runCommand "lucid-lyrics" {} ''
  #   mkdir -p $out
  #   cp ${lucidLyricsJs} $out/lucid-lyrics.js
  # '';
in {
  imports = [
    inputs.spicetify-nix.nixosModules.spicetify
  ];

  programs.spicetify = {
    enable = true;

    enabledExtensions = with spicePkgs.extensions; [
      adblock
      oneko
    ];

    theme = spicePkgs.themes.default;
  };
}

{
  inputs,
  pkgs,
  ...
}: let
  spicePkgs = inputs.spicetify-nix.legacyPackages.${pkgs.stdenv.hostPlatform.system};

  lucidLyrics = {
    src = pkgs.runCommand "lucid-lyrics-src" {} ''
      mkdir -p $out
      cp ${pkgs.fetchurl {
        url = "https://lucid-lyrics.sanooj.uk/spice/lucid-lyrics.js";
        hash = "sha256-o+sha5aC0gJDYb/NDEHSvdZzvxNhDzVLqzOswdHq6u0";
      }} $out/lucid-lyrics.js
    '';
    name = "lucid-lyrics.js";
  };
in {
  imports = [
    inputs.spicetify-nix.nixosModules.spicetify
  ];

  programs.spicetify = {
    enable = true;

    enabledExtensions =
      (with spicePkgs.extensions; [
        adblock
        oneko
      ])
      ++ [lucidLyrics];

    theme = spicePkgs.themes.defaultDynamic;
  };
}

{pkgs, ...}: let
  betterLyrics = pkgs.fetchurl {
    url = "https://github.com/better-lyrics/better-lyrics/releases/download/v2.3.3/chrome-v2.3.3.zip";
    hash = "sha256-Vt2jVnlpX1ZfDrwHE6eSJb/yxijlfMX7n3yGucEaWZM=";
  };

  betterLyricsShaders = pkgs.fetchurl {
    url = "https://github.com/better-lyrics/shaders/releases/download/v1.2.0/chrome-v1.2.0.zip";
    hash = "sha256-EzWCZ0hhOVn1rfMNihnWWH0XsaNXWos2OJ0rqWcZkTw=";
  };

  betterLyricsExtensions = pkgs.runCommand "pear-desktop-better-lyrics-extensions" {
    nativeBuildInputs = [pkgs.unzip];
  } ''
    mkdir -p $out/better-lyrics $out/better-lyrics-shaders
    unzip -q ${betterLyrics} -d $out/better-lyrics
    unzip -q ${betterLyricsShaders} -d $out/better-lyrics-shaders
  '';
in {
  environment.systemPackages = [
    (pkgs.symlinkJoin {
      name = "pear-desktop-better-lyrics";
      paths = [pkgs.pear-desktop];
      nativeBuildInputs = [pkgs.makeWrapper];
      postBuild = ''
        wrapProgram $out/bin/pear-desktop \
          --add-flags "--load-extension=${betterLyricsExtensions}/better-lyrics" \
          --add-flags "--load-extension=${betterLyricsExtensions}/better-lyrics-shaders"
      '';
    })
  ];
}

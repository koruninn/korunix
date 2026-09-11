{pkgs, ...}: let
  betterLyrics = pkgs.fetchurl {
    url = "https://github.com/better-lyrics/better-lyrics/releases/download/v2.4.0.7/chrome-v2.4.0.7.zip";
    hash = "sha256-2PnSGnaQAUDaR9MPzqSZSZnu7vQ7ymrOQJP4Yh1WAyQ=";
  };

  betterLyricsShaders = pkgs.fetchurl {
    url = "https://github.com/better-lyrics/shaders/releases/download/v1.2.0/chrome-v1.2.0.zip";
    hash = "sha256-EzWCZ0hhOVn1rfMNihnWWH0XsaNXWos2OJ0rqWcZkTw=";
  };

  betterLyricsExtensions = pkgs.runCommand "pear-desktop-better-lyrics-extensions" {
    nativeBuildInputs = [pkgs.unzip pkgs.ripgrep];
  } ''
    mkdir -p $out/better-lyrics $out/better-lyrics-shaders
    unzip -q ${betterLyrics} -d $out/better-lyrics
    unzip -q ${betterLyricsShaders} -d $out/better-lyrics-shaders

    find $out/better-lyrics -type f -name '*.js' -exec sed -i \
      -e 's/chrome\\.windows\\.onRemoved\\.addListener/chrome.windows?.onRemoved?.addListener/g' \
      -e 's/chrome\\.windows\\.remove(/chrome.windows?.remove?.(/g' \
      -e 's/chrome\\.windows\\.create(/chrome.windows?.create?.(/g' \
      {} +

    if rg -q 'chrome\\.windows\\.onRemoved\\.addListener' $out/better-lyrics; then
      echo "Better Lyrics: no se pudo aplicar la compatibilidad con chrome.windows" >&2
      exit 1
    fi
  '';

  pearDesktop = pkgs.pear-desktop.overrideAttrs (old: {
    postPatch = (old.postPatch or "") + ''
      sed -i \
        '/const win = new BrowserWindow(electronWindowSettings);/a\
  await session.defaultSession.loadExtension("${betterLyricsExtensions}/better-lyrics", {allowFileAccess: true});\
  await session.defaultSession.loadExtension("${betterLyricsExtensions}/better-lyrics-shaders", {allowFileAccess: true});' \
        src/index.ts

      substituteInPlace src/plugins/do-not-track/index.ts \
        --replace-fail \
          "    enabled: false," \
          "    enabled: true,"

      substituteInPlace src/plugins/discord/index.ts \
        --replace-fail \
          "    'enabled': false," \
          "    'enabled': true,"
    '';
  });
in {
  environment.systemPackages = [pearDesktop];
}

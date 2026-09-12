{
  lib,
  pkgs,
  inputs,
  ...
}: let
  spicePkgs = inputs.spicetify-nix.legacyPackages.${pkgs.stdenv.hostPlatform.system};
  extensions = spicePkgs.extensions;

  # Cambia solamente esta línea para elegir cualquier tema del catálogo.
  theme = spicePkgs.themes.defaultDynamic;

  enabledExtensions = [
    extensions.adblock
    extensions.spicyLyrics
    extensions.oneko
  ] ++ (theme.requiredExtensions or []);
  asFlag = enabled: if enabled then "1" else "0";
  extensionManifest = pkgs.writeText "korunix-spicetify-extensions" (
    lib.concatMapStringsSep "\n"
      (extension: "${toString extension.src}/${extension.name}\t${extension.name}")
      enabledExtensions
    + "\n"
  );
  experimentalFeatures = lib.any (extension: extension.experimentalFeatures or false) enabledExtensions;
  themePatches = pkgs.writeText "korunix-spicetify-patches.ini" (
    lib.generators.toINI {} {Patch = theme.patches or {};}
  );
  themeSetup = pkgs.writeShellScript "korunix-spicetify-theme-setup" (theme.extraCommands or "");
  themeAdditionalCss = pkgs.writeText "korunix-spicetify-additional.css" (theme.additionalCss or "");
  paletteFallback = spicePkgs.themes.default.src;

  runtime = pkgs.writeShellApplication {
    name = "korunix-spotify-runtime";
    runtimeInputs = with pkgs; [coreutils gawk gnugrep gnused psmisc rsync util-linux];
    text = ''
      export KORUNIX_SPOTIFY_SOURCE=${lib.escapeShellArg "${pkgs.spotify}/share/spotify"}
      export KORUNIX_THEME_NAME=${lib.escapeShellArg theme.name}
      export KORUNIX_THEME_SOURCE=${lib.escapeShellArg (toString theme.src)}
      export KORUNIX_THEME_INJECT_CSS=${asFlag (theme.injectCss or true)}
      export KORUNIX_THEME_INJECT_JS=${asFlag (theme.injectThemeJs or true)}
      export KORUNIX_THEME_REPLACE_COLORS=${asFlag (theme.replaceColors or true)}
      export KORUNIX_THEME_OVERWRITE_ASSETS=${asFlag (theme.overwriteAssets or false)}
      export KORUNIX_THEME_HOME_CONFIG=${asFlag (theme.homeConfig or true)}
      export KORUNIX_THEME_EXPERIMENTAL_FEATURES=${asFlag experimentalFeatures}
      export KORUNIX_THEME_EXTENSIONS=${lib.escapeShellArg extensionManifest}
      export KORUNIX_THEME_PATCHES=${lib.escapeShellArg themePatches}
      export KORUNIX_THEME_SETUP=${lib.escapeShellArg themeSetup}
      export KORUNIX_THEME_ADDITIONAL_CSS=${lib.escapeShellArg themeAdditionalCss}
      export KORUNIX_PALETTE_FALLBACK=${lib.escapeShellArg (toString paletteFallback)}
      export KORUNIX_SPICETIFY_CLI=${lib.escapeShellArg "${pkgs.spicetify-cli}/bin/spicetify"}
      ${builtins.readFile ./spicetify-runtime.sh}
    '';
  };

  spotifyLauncher = pkgs.writeShellScriptBin "spotify" ''
    exec ${runtime}/bin/korunix-spotify-runtime --launch "$@"
  '';
  spicetifyLauncher = pkgs.writeShellScriptBin "spicetify" ''
    ${runtime}/bin/korunix-spotify-runtime --prepare || exit $?
    export SPICETIFY_CONFIG="''${XDG_CONFIG_HOME:-$HOME/.config}/spicetify"
    exec ${pkgs.spicetify-cli}/bin/spicetify "$@"
  '';
  desktopItem = pkgs.makeDesktopItem {
    name = "spotify";
    desktopName = "Spotify";
    exec = "spotify %U";
    icon = "spotify-client";
    categories = ["Audio" "Music" "Player"];
    mimeTypes = ["x-scheme-handler/spotify"];
    startupWMClass = "spotify";
  };
  application = pkgs.runCommand "korunix-spotify" {} ''
    mkdir -p "$out/bin" "$out/share"
    ln -s ${spotifyLauncher}/bin/spotify "$out/bin/spotify"
    ln -s ${spicetifyLauncher}/bin/spicetify "$out/bin/spicetify"
    ln -s ${desktopItem}/share/applications "$out/share/applications"
    ln -s ${pkgs.spotify}/share/icons "$out/share/icons"
  '';
in {
  environment.systemPackages = [application] ++ (theme.extraPkgs or []);
  systemd.user.services.korunix-spotify-prepare = {
    description = "Preparar Spotify con ${theme.name} y los colores de Noctalia";
    wantedBy = ["default.target"];
    serviceConfig = {
      Type = "oneshot";
      ExecStart = "${runtime}/bin/korunix-spotify-runtime --prepare";
    };
  };
}

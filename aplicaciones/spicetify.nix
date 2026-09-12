{
  lib,
  pkgs,
  inputs,
  ...
}: let
  spicePkgs = inputs.spicetify-nix.legacyPackages.${pkgs.stdenv.hostPlatform.system};
  extensions = spicePkgs.extensions;
  defaultTheme = spicePkgs.themes.default;

  runtime = pkgs.writeShellApplication {
    name = "korunix-spotify-runtime";
    runtimeInputs = with pkgs; [coreutils gnugrep gnused psmisc rsync util-linux];
    text = ''
      export KORUNIX_SPOTIFY_SOURCE=${lib.escapeShellArg "${pkgs.spotify}/share/spotify"}
      export KORUNIX_DEFAULT_SOURCE=${lib.escapeShellArg (toString defaultTheme.src)}
      export KORUNIX_ADBLOCK_SOURCE=${lib.escapeShellArg "${extensions.adblock.src}/${extensions.adblock.name}"}
      export KORUNIX_LYRICS_SOURCE=${lib.escapeShellArg "${extensions.spicyLyrics.src}/${extensions.spicyLyrics.name}"}
      export KORUNIX_ONEKO_SOURCE=${lib.escapeShellArg "${extensions.oneko.src}/${extensions.oneko.name}"}
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
  # Noctalia escribe color.ini en el hogar del usuario; Spicetify lo aplica
  # a una copia modificable de Spotify preparada desde el paquete Nix.
  environment.systemPackages = [application];
  systemd.user.services.korunix-spotify-prepare = {
    description = "Preparar Spotify con Default y los colores de Noctalia";
    wantedBy = ["default.target"];
    serviceConfig = {
      Type = "oneshot";
      ExecStart = "${runtime}/bin/korunix-spotify-runtime --prepare";
    };
  };
}

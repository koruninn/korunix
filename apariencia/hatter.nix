{
  config,
  pkgs,
  lib,
  ...
}: let
  hatter = pkgs.stdenvNoCC.mkDerivation {
    pname = "hatter-icon-theme";
    version = "3";

    src = pkgs.fetchFromGitHub {
      owner = "Mibea";
      repo = "Hatter";
      rev = "2d7c78276adf11613611733563b008fe021e3ecf";
      hash = "sha256-axFx8DEdzY3XVCQcOWmr5tocNUhIAJqcARgxBWwg0aY=";
    };

    installPhase = ''
      runHook preInstall
      mkdir -p $out/share/icons
      cp -r Hatter $out/share/icons/
      cp -r Hatter-Slate $out/share/icons/

      # Vincular los iconos con los nombres que usan los lanzadores y PolyGlot.
      for theme in Hatter Hatter-Slate; do
        apps="$out/share/icons/$theme/scalable/apps"
        mkdir -p "$apps"
        for pair in \
          anime-game-launcher:an-anime-game-launcher \
          moe.launcher.an-anime-game-launcher:an-anime-game-launcher \
          honkers-railway-launcher:the-honkers-railway-launcher \
          moe.launcher.the-honkers-railway-launcher:the-honkers-railway-launcher \
          io.github.DraqueT.PolyGlot:PolyGlot \
          polyglot:PolyGlot; do
          alias="''${pair%%:*}"
          original="''${pair#*:}"
          # Slate hereda estos iconos de Hatter.
          if [ "$theme" = Hatter-Slate ]; then
            ln -sfn "../../../Hatter/scalable/apps/$original.svg" "$apps/$original.svg"
          fi
          ln -sfn "$original.svg" "$apps/$alias.svg"
        done
      done
      runHook postInstall
    '';

    meta = {
      description = "Tema de iconos Hatter con variante Slate";
      homepage = "https://github.com/Mibea/Hatter";
      license = lib.licenses.gpl3Only;
      platforms = lib.platforms.linux;
    };
  };
  # Usar los SVG de Hatter sin cambiar los identificadores de las aplicaciones.
  launchers = [
    { id = "anime-game-launcher"; icon = "an-anime-game-launcher"; }
    { id = "honkers-railway-launcher"; icon = "the-honkers-railway-launcher"; }
  ];
  enabledLaunchers = lib.filter (launcher: lib.attrByPath ["programs" launcher.id "enable"] false config) launchers;
  launcherIcons = pkgs.runCommand "korunix-hatter-launcher-icons" {} ''
    mkdir -p "$out/share/applications"
    ${lib.concatMapStringsSep "\n" (launcher: let
      package = config.programs.${launcher.id}.package;
      entry = "${launcher.id}.desktop";
    in ''
      install -m 0644 ${package}/share/applications/${entry} "$out/share/applications/${entry}"
      substituteInPlace "$out/share/applications/${entry}" \
        --replace-fail 'Icon=${if launcher.id == "anime-game-launcher" then "moe.launcher.an-anime-game-launcher" else "moe.launcher.the-honkers-railway-launcher"}' 'Icon=${hatter}/share/icons/Hatter/scalable/apps/${launcher.icon}.svg'
    '') enabledLaunchers}
  '';
in {
  environment.systemPackages = [hatter] ++ lib.optional (enabledLaunchers != []) (lib.hiPrio launcherIcons);
}

{ pkgs, lib, ... }:

let
  hatter = pkgs.stdenvNoCC.mkDerivation {
    pname = "hatter-icon-theme";
    version = "3";

    src = pkgs.fetchFromGitHub {
      owner = "Mibea";
      repo = "Hatter";
      rev = "2d7c78276adf11611733563b008fe021e3ecf";
      hash = "sha256-axFx8DEdzY3XVCQcOWmr5tocNUhIAJqcARgxBWwg0aY=";
    };

    installPhase = ''
      runHook preInstall
      mkdir -p $out/share/icons
      cp -r Hatter $out/share/icons/
      cp -r Hatter-Slate $out/share/icons/
      runHook postInstall
    '';

    meta = {
      description = "Tema de iconos Hatter con variante Slate";
      homepage = "https://github.com/Mibea/Hatter";
      license = lib.licenses.gpl3Only;
      platforms = lib.platforms.linux;
    };
  };
in
{
  environment.systemPackages = [ hatter ];
}

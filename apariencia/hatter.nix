{
  inputs,
  lib,
  pkgs,
  ...
}: let
  hatter = pkgs.stdenvNoCC.mkDerivation {
    pname = "hatter-icon-theme";
    version = "git";
    src = inputs.hatter;

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
in {
  environment.systemPackages = [hatter];
}

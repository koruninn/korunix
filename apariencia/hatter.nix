{ pkgs, lib, ... }:

let
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
      cp -r Hatter Hatter-Green $out/share/icons/
      runHook postInstall
    '';

    meta = {
      description = "Tema de iconos Hatter";
      homepage = "https://github.com/Mibea/Hatter";
      license = lib.licenses.gpl3Only;
      platforms = lib.platforms.linux;
    };
  };
in
{
  environment.systemPackages = [ hatter ];

  environment.etc = {
    "gtk-2.0/gtkrc".text = ''
      gtk-icon-theme-name="Hatter-Green"
    '';

    "gtk-3.0/settings.ini".text = ''
      [Settings]
      gtk-icon-theme-name=Hatter-Green
    '';

    "gtk-4.0/settings.ini".text = ''
      [Settings]
      gtk-icon-theme-name=Hatter-Green
    '';

    "xdg/kdeglobals".text = ''
      [Icons]
      Theme=Hatter-Green
    '';
  };
}

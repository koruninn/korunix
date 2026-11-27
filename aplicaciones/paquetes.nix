{
  equipo,
  inputs,
  lib,
  pkgs,
  ...
}: let
  lanzadoresOcultos = pkgs.runCommand "korunix-lanzadores-ocultos" {} ''
        mkdir -p "$out/share/applications"

        ocultar() {
          id="$1"
          nombre="$2"
          cat > "$out/share/applications/$id" <<EOF
    [Desktop Entry]
    Type=Application
    Name=$nombre
    NoDisplay=true
    Hidden=true
    EOF
        }

        ocultar qt5ct.desktop "Ajustes de Qt5"
        ocultar qt6ct.desktop "Ajustes de Qt6"
        ocultar nixos-manual.desktop "Manual de NixOS"
  '';
in {
  services.gvfs.enable = true;
  services.udisks2.enable = true;

  environment.gnome.excludePackages = with pkgs; [
    epiphany
    gnome-tour
    gnome-user-docs
    yelp
  ];

  environment.systemPackages = with pkgs; [
    (lib.hiPrio lanzadoresOcultos)
    android-tools
    birdfont
    blender
    convey
    darktable
    davinci-resolve
    fastfetch
    file-roller
    fontforge
    gimp
    heroic
    inkscape
    kdePackages.kdenlive
    libreoffice
    loupe
    lutris
    mprisence
    nautilus
    obsidian
    onlyoffice-desktopeditors
    papers
    prismlauncher
    protonplus
    rar
    resources
    vesktop
    vlc
    (vscode.overrideAttrs (old: {
          nativeBuildInputs = (old.nativeBuildInputs or []) ++ [pkgs.makeWrapper];
          postInstall = (old.postInstall or "") + "
wrapProgram $out/bin/code --add-flags --password-store=basic
";
        }))
    zoom-us
  ];
}

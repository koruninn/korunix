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
        ocultar scrcpy-console.desktop "scrcpy (consola)"
        ocultar nixos-manual.desktop "Manual de NixOS"
  '';

  usaGnomeKeyring = (equipo.secretService or null) == "gnome-keyring";
  mailspringEquipo =
    if usaGnomeKeyring
    then pkgs.mailspring.override {commandLineArgs = "--password-store=gnome-libsecret";}
    else pkgs.mailspring;
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
    alacritty
    android-tools
    birdfont
    blender
    darktable
    fastfetch
    file-roller
    fontforge
    gimp
    google-chrome
    heroic
    inkscape
    kdePackages.kdenlive
    libreoffice
    loupe
    lutris
    nautilus
    nuclear
    obsidian
    onlyoffice-desktopeditors
    papers
    pear-desktop
    polyglot
    prismlauncher
    protonplus
    pywalfox-native
    rar
    resources
    scrcpy
    stirling-pdf-desktop
    mailspringEquipo
    vesktop
    vlc
    (vscode.overrideAttrs (old: {
          nativeBuildInputs = (old.nativeBuildInputs or []) ++ [pkgs.makeWrapper];
          postInstall = (old.postInstall or "") + "
wrapProgram $out/bin/code --add-flags --password-store=gnome-libsecret
";
        }))
    zoom-us
  ];
}

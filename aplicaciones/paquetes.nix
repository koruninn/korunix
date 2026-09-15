{
  equipo,
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

  # Umbriel no es identificado por Chromium/Electron como GNOME o KDE, así
  # que Mailspring no selecciona libsecret automáticamente aunque el Secret
  # Service esté disponible. Solo Korunix fuerza GNOME Keyring; Optiplex
  # conserva el comportamiento normal del paquete.
  mailspringEquipo =
    if equipo.persona == "koru"
    then pkgs.mailspring.override {commandLineArgs = "--password-store=gnome-libsecret";}
    else pkgs.mailspring;
in {
  services.gvfs.enable = true;
  services.udisks2.enable = true;

  # GNOME sigue completo, pero sin aplicaciones auxiliares que Korunix no usa.
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
    vscode
    zoom-us
  ];
}

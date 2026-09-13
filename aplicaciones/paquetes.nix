{
  inputs,
  pkgs,
  ...
}: {
  # Acceso a dispositivos y medios extraíbles desde aplicaciones GTK.
  services.gvfs.enable = true;
  services.udisks2.enable = true;

  # Lista de paquetes instalados en el perfil del sistema
  environment.systemPackages = with pkgs; [
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
    inputs.zen-browser.packages.${pkgs.stdenv.hostPlatform.system}.default
    kdePackages.kate
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
    rar
    scrcpy
    sunshine
    thunderbird
    unrar
    valent
    vesktop
    vlc
    vscode
    xwayland-satellite
    zoom-us
  ];
}

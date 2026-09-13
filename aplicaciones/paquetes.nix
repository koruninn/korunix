{
  config,
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
    eog
    fastfetch
    figma-linux
    fontforge
    gimp
    google-chrome
    heroic
    inkscape
    inputs.zen-browser.packages.${pkgs.stdenv.hostPlatform.system}.default
    kdePackages.kate
    kdePackages.kdenlive
    libreoffice
    lutris
    nautilus
    nautilus-open-any-terminal
    obsidian
    onlyoffice-desktopeditors
    peazip
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

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
    eog
    fastfetch
    figma-linux
    fontforge
    google-chrome
    heroic
    inputs.zen-browser.packages.${pkgs.stdenv.hostPlatform.system}.default
    kdePackages.kate
    kdePackages.kdenlive
    lutris
    nautilus
    nautilus-open-any-terminal
    onlyoffice-desktopeditors
    peazip
    pear-desktop
    polyglot
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

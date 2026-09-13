{
  equipo,
  lib,
  pkgs,
  ...
}: let
  cinnamonWayland = pkgs.runCommand "optiplex-cinnamon-wayland-session" {
    passthru.providedSessions = ["cinnamon-wayland"];
  } ''
    origen=${pkgs.cinnamon}/share/wayland-sessions/cinnamon-wayland.desktop

    if [ ! -f "$origen" ]; then
      echo "No existe cinnamon-wayland.desktop en ${pkgs.cinnamon}" >&2
      exit 1
    fi

    mkdir -p "$out/share/wayland-sessions"
    cp "$origen" "$out/share/wayland-sessions/cinnamon-wayland.desktop"
  '';
in {
  time.timeZone = "America/Lima";
  i18n.defaultLocale = "es_PE.UTF-8";

  # LightDM puede lanzar sesiones Wayland. Se conserva su servidor X únicamente
  # para el propio greeter; Cinnamon X11 y Cinnamon 2D ya no se ofrecen.
  services.xserver.enable = true;
  services.xserver.displayManager.lightdm.enable = true;
  services.displayManager.sessionPackages = lib.mkForce [cinnamonWayland];
  services.displayManager.defaultSession = "cinnamon-wayland";
  services.displayManager.autoLogin = {
    enable = true;
    user = equipo.persona;
  };

  services.xserver.desktopManager.cinnamon.enable = true;

  # Las aplicaciones X11 siguen funcionando dentro de Cinnamon Wayland mediante
  # XWayland; esto no vuelve a habilitar una sesión de escritorio X11.
  programs.xwayland.enable = true;

  services.xserver.xkb = {
    layout = "es";
    variant = "deadtilde";
  };

  console.keyMap = "es";

  programs.firefox.enable = false;
  nixpkgs.config.allowUnfree = true;

  environment.systemPackages = with pkgs; [
    (google-chrome.override {
      commandLineArgs = "--password-store=basic";
    })
  ];

  system.stateVersion = "26.05";
}

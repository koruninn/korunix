{
  aplicaciones,
  escritorio,
  lib,
  noctaliaPackage,
  nombre,
  personas,
  pkgs,
  programa,
  ...
}: let
  cuentas = map (persona: persona.cuenta) personas;

  personasValidas =
    if personas == []
    then throw "Falta al menos una cuenta en [[personas]]."
    else if builtins.length cuentas != builtins.length (lib.unique cuentas)
    then throw "Hay una cuenta repetida en [[personas]]."
    else personas;

  usuarios = builtins.listToAttrs (map (persona: {
      name = persona.cuenta;
      value = {
        isNormalUser = true;
        description = persona.nombre;
        shell = pkgs.fish;
        extraGroups =
          ["networkmanager"]
          ++ lib.optionals (persona.administrador or false) ["wheel"];
      };
    })
    personasValidas);

  sesion =
    if escritorio == "niri"
    then "niri"
    else if escritorio == "hyprland"
    then "hyprland-uwsm"
    else if escritorio == "cinnamon"
    then "cinnamon-wayland"
    else if escritorio == "plasma"
    then "plasma"
    else throw "No conozco el escritorio «${escritorio}».";

  noctaliaActivo = escritorio == "niri" || escritorio == "hyprland";
in {
  imports = [./hardware.nix];

  networking.hostName = nombre;
  networking.networkmanager.enable = true;

  nix.settings.experimental-features = ["nix-command" "flakes"];
  nixpkgs.config.allowUnfree = true;

  programs.fish.enable = true;

  # La contraseña no se guarda aquí. La cuenta que ya existe conserva la suya.
  users.mutableUsers = true;
  users.users = usuarios;

  # GDM muestra las sesiones; GNOME no se instala como escritorio.
  services.xserver.enable = true;
  services.displayManager = {
    gdm.enable = true;
    defaultSession = sesion;
  };

  programs.niri.enable = escritorio == "niri";

  programs.hyprland = lib.mkIf (escritorio == "hyprland") {
    enable = true;
    withUWSM = true;
    xwayland.enable = true;
  };

  services.xserver.desktopManager.cinnamon.enable = escritorio == "cinnamon";
  services.desktopManager.plasma6.enable = escritorio == "plasma";

  environment.sessionVariables = lib.mkIf (escritorio == "niri") {
    NIRI_CONFIG = "/etc/niri/config.kdl";
  };

  environment.etc."niri/config.kdl" = lib.mkIf (escritorio == "niri") {
    source = ./niri.kdl;
  };

  environment.etc."korunix/noctalia.toml" = lib.mkIf noctaliaActivo {
    source = ./noctalia.toml;
  };

  # Noctalia necesita estas piezas para sus controles. No se añaden applets
  # gráficos de NetworkManager ni Blueman.
  hardware.bluetooth = lib.mkIf noctaliaActivo {
    enable = true;
    powerOnBoot = true;
  };

  services.upower.enable = lib.mkIf noctaliaActivo true;
  services.power-profiles-daemon.enable = lib.mkIf noctaliaActivo true;

  security.polkit.enable = true;
  security.rtkit.enable = lib.mkIf noctaliaActivo true;

  services.pipewire = lib.mkIf noctaliaActivo {
    enable = true;
    alsa.enable = true;
    alsa.support32Bit = true;
    pulse.enable = true;
  };

  # Antes de abrir Noctalia, Korunix prepara únicamente su política de capturas.
  systemd.user.services.korunix-sesion = lib.mkIf noctaliaActivo {
    description = "Prepara la sesión de Korunix";
    wantedBy = ["graphical-session.target"];
    before = ["noctalia.service"];

    environment.KORUNIX_NOCTALIA_BASE = "/etc/korunix/noctalia.toml";

    serviceConfig = {
      Type = "oneshot";
      ExecStart = "${programa}/bin/korunix sesion preparar";
    };
  };

  # Noctalia solo pertenece a Niri y Hyprland.
  systemd.user.services.noctalia = lib.mkIf noctaliaActivo {
    description = "Noctalia";
    partOf = ["graphical-session.target"];
    wantedBy = ["graphical-session.target"];
    after = [
      "graphical-session.target"
      "korunix-sesion.service"
    ];
    requires = ["korunix-sesion.service"];

    enableDefaultPath = false;

    serviceConfig = {
      ExecStart = lib.getExe noctaliaPackage;
      Restart = "on-failure";
    };
  };

  environment.systemPackages =
    aplicaciones
    ++ [programa]
    ++ lib.optionals noctaliaActivo [
      noctaliaPackage
      pkgs.alacritty
      pkgs.nautilus
    ]
    ++ lib.optionals (escritorio == "niri") [
      pkgs.xwayland-satellite
    ];

  # Conserva la compatibilidad de la instalación actual.
  system.stateVersion = "26.05";
}

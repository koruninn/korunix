{
  aplicaciones,
  config,
  escritorio,
  idioma,
  lib,
  monitor,
  noctaliaPackage,
  nombre,
  personas,
  pkgs,
  programa,
  salidaMonitor,
  teclado,
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

  idiomaCodigo =
    if idioma.sistema == "español"
    then "es"
    else throw "Todavía no conozco el idioma «${idioma.sistema}».";

  region =
    if idioma.region == "Perú"
    then {
      codigo = "PE";
      zonaHoraria = "America/Lima";
    }
    else throw "Todavía no conozco la región «${idioma.region}».";

  locale = "${idiomaCodigo}_${region.codigo}.UTF-8";

  teclados = {
    "españa" = {
      xkb = "es";
      variante = "deadtilde";
      consola = "es";
    };
    "latinoamérica" = {
      xkb = "latam";
      variante = "";
      consola = "la-latin1";
    };
  };

  resolverTeclado = nombreTeclado:
    teclados.${nombreTeclado}
    or (throw "Todavía no conozco el teclado «${nombreTeclado}».");

  tecladosResueltos = map resolverTeclado teclado.distribuciones;
  xkbLayouts = map (valor: valor.xkb) tecladosResueltos;
  xkbVariantes = map (valor: valor.variante) tecladosResueltos;

  cambioXkb =
    if teclado.cambio == "alt+shift"
    then "grp:alt_shift_toggle"
    else throw "No conozco la combinación «${teclado.cambio}».";

  tecladoPrincipal =
    if tecladosResueltos == []
    then throw "Elige al menos una distribución de teclado."
    else builtins.head tecladosResueltos;

  modoMonitor = "${monitor.resolucion}@${toString monitor.hz}.000";

  ibusPackage = pkgs.ibus-with-plugins.override {
    plugins = config.i18n.inputMethod.ibus.engines;
  };
in {
  imports = [./hardware.nix];

  networking.hostName = nombre;
  networking.networkmanager.enable = true;

  nix.settings.experimental-features = ["nix-command" "flakes"];
  nixpkgs.config.allowUnfree = true;

  programs.fish.enable = true;

  users.mutableUsers = true;
  users.users = usuarios;

  i18n.defaultLocale = locale;

  i18n.extraLocaleSettings = {
    LC_ADDRESS = locale;
    LC_IDENTIFICATION = locale;
    LC_MEASUREMENT = locale;
    LC_MONETARY = locale;
    LC_NAME = locale;
    LC_NUMERIC = locale;
    LC_PAPER = locale;
    LC_TELEPHONE = locale;
    LC_TIME = locale;
  };

  time.timeZone = region.zonaHoraria;

  # XKB sigue siendo dueño de las distribuciones normales.
  services.xserver.xkb = {
    layout = lib.concatStringsSep "," xkbLayouts;
    variant = lib.concatStringsSep "," xkbVariantes;
    options = cambioXkb;
  };

  console.keyMap = tecladoPrincipal.consola;

  # IBus aporta composición y diacríticos sin reemplazar XKB.
  i18n.inputMethod = {
    enable = true;
    type = "ibus";
    ibus.waylandFrontend = true;
  };

  # Niri y Hyprland usan el frontend Wayland de IBus.
  environment.etc."xdg/autostart/ibus-daemon.desktop" =
    lib.mkIf (escritorio == "niri" || escritorio == "hyprland")
    {
      text = ''
        [Desktop Entry]
        Name=IBus
        Type=Application
        Exec=${ibusPackage}/bin/ibus start --type wayland
        NotShowIn=KDE;
      '';
    };

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

  environment.etc."korunix/niri-input.kdl" = lib.mkIf (escritorio == "niri") {
    text = ''
      // El teclado viene de configuracion.toml.
      input {
          keyboard {
              xkb {
                  layout ${builtins.toJSON (lib.concatStringsSep "," xkbLayouts)}
                  variant ${builtins.toJSON (lib.concatStringsSep "," xkbVariantes)}
                  options ${builtins.toJSON cambioXkb}
              }

              numlock
          }
      }
    '';
  };

  environment.etc."korunix/niri-output.kdl" = lib.mkIf (escritorio == "niri") {
    text = ''
      // La salida es un hecho detectado; resolución y Hz vienen de configuracion.toml.
      output ${builtins.toJSON salidaMonitor} {
          mode ${builtins.toJSON modoMonitor}
      }
    '';
  };

  environment.etc."korunix/noctalia.toml" = lib.mkIf noctaliaActivo {
    source = ./noctalia.toml;
  };

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

  system.stateVersion = "26.05";
}

{
  almacenamiento,
  aplicaciones,
  aplicacionesElegidas,
  config,
  escritorio,
  escritorios,
  idioma,
  impresion,
  lib,
  monitor,
  noctaliaPackage,
  nombre,
  personas,
  pkgs,
  programa,
  salidaMonitor,
  sunshine,
  steam,
  teclado,
  unidadesDetectadas,
  virtualizacion,
  ...
}: let
  escritoriosValidos =
    if !(builtins.elem escritorio escritorios)
    then throw "El escritorio principal también tiene que estar en «instalados»."
    else if builtins.length escritorios != builtins.length (lib.unique escritorios)
    then throw "Hay un escritorio repetido en «instalados»."
    else escritorios;

  niriActivo = builtins.elem "niri" escritoriosValidos;
  hyprlandActivo = builtins.elem "hyprland" escritoriosValidos;
  cinnamonActivo = builtins.elem "cinnamon" escritoriosValidos;
  plasmaActivo = builtins.elem "plasma" escritoriosValidos;
  noctaliaActivo = niriActivo || hyprlandActivo;

  localsendActivo = builtins.elem "localsend" aplicacionesElegidas;
  obsActivo = builtins.elem "obs-studio" aplicacionesElegidas;
  scrcpyActivo = builtins.elem "scrcpy" aplicacionesElegidas;

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
          ++ lib.optionals (persona.administrador or false) ["wheel"]
          ++ lib.optionals (sunshine.activo or false) ["input" "uinput"]
          ++ lib.optionals (virtualizacion.activa or false) ["libvirtd" "kvm"]
          ++ lib.optionals (impresion.activa or false) ["lp" "scanner"];
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

  unidadesSolicitadas = almacenamiento.disponibles or [];

  resolverUnidad = unidad:
    if builtins.hasAttr unidad unidadesDetectadas
    then
      {nombre = unidad;}
      // builtins.getAttr unidad unidadesDetectadas
    else throw "No conozco la unidad «${unidad}» en este equipo.";

  unidades = map resolverUnidad unidadesSolicitadas;

  opcionesUnidad = unidad:
    [
      "nofail"
      "x-systemd.automount"
      "x-systemd.device-timeout=5s"
      "x-gvfs-show"
    ]
    ++ lib.optionals
    (builtins.elem unidad.sistemaArchivos ["ntfs" "exfat" "vfat"])
    [
      "uid=${toString unidad.uid}"
      "gid=${toString unidad.gid}"
      "umask=0077"
    ]
    ++ lib.optionals (unidad.sistemaArchivos == "ntfs") [
      "windows_names"
    ];

  sistemasDeArchivos = builtins.listToAttrs (map (unidad: {
      name = "/mnt/${unidad.nombre}";
      value = {
        device = "/dev/disk/by-uuid/${unidad.uuid}";
        fsType =
          if unidad.sistemaArchivos == "ntfs"
          then "ntfs3"
          else unidad.sistemaArchivos;
        options = opcionesUnidad unidad;
      };
    })
    unidades);

  sesionNoctalia = pkgs.writeShellScript "korunix-es-sesion-noctalia" ''
    escritorio="''${XDG_CURRENT_DESKTOP:-}"

    # Si la sesión actual está identificada, esa señal manda. Una variable vieja
    # de una sesión anterior no puede convertir Plasma o Cinnamon en Niri.
    if [ -n "$escritorio" ]; then
      case ":$escritorio:" in
        *:niri:*|*:Hyprland:*|*:hyprland:*)
          exit 0
          ;;
        *)
          exit 1
          ;;
      esac
    fi

    escritorio="''${XDG_SESSION_DESKTOP:-}"

    if [ -n "$escritorio" ]; then
      case ":$escritorio:" in
        *:niri:*|*:Hyprland:*|*:hyprland:*|*:hyprland-uwsm:*)
          exit 0
          ;;
        *)
          exit 1
          ;;
      esac
    fi

    escritorio="''${DESKTOP_SESSION:-}"

    if [ -n "$escritorio" ]; then
      case "$escritorio" in
        niri|hyprland|hyprland-uwsm)
          exit 0
          ;;
        *)
          exit 1
          ;;
      esac
    fi

    # Solo si la sesión no publicó ninguna de esas variables se usa el socket
    # del compositor como respaldo.
    if compgen -G "''${XDG_RUNTIME_DIR:-/run/user/$UID}/niri*.sock" >/dev/null; then
      exit 0
    fi

    if [ -d "''${XDG_RUNTIME_DIR:-/run/user/$UID}/hypr" ]; then
      exit 0
    fi

    exit 1
  '';
in {
  imports = [./hardware.nix];

  networking.hostName = nombre;
  networking.networkmanager.enable = true;

  nix.settings.experimental-features = ["nix-command" "flakes"];
  nixpkgs.config.allowUnfree = true;

  hardware.enableRedistributableFirmware = true;

  hardware.graphics = {
    enable = true;
    enable32Bit = pkgs.stdenv.hostPlatform.system == "x86_64-linux";
  };

  programs.appimage = {
    enable = true;
    binfmt = true;
  };

  services.flatpak.enable = true;

  services.openssh = {
    enable = true;
    openFirewall = true;
  };

  services.avahi = {
    enable = true;
    openFirewall = true;
  };

  services.udisks2.enable = true;
  services.gvfs.enable = true;

  services.fwupd.enable = true;
  systemd.services.fwupd-refresh.enable = false;
  systemd.timers.fwupd-refresh.enable = false;

  services.gnome.gcr-ssh-agent.enable = false;
  programs.ssh.startAgent = true;

  programs.fish.enable = true;

  users.mutableUsers = true;
  users.users = usuarios;

  fileSystems = sistemasDeArchivos;

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

  services.xserver.xkb = {
    layout = lib.concatStringsSep "," xkbLayouts;
    variant = lib.concatStringsSep "," xkbVariantes;
    options = cambioXkb;
  };

  console.keyMap = tecladoPrincipal.consola;

  i18n.inputMethod = {
    enable = true;
    type = "ibus";
    ibus.waylandFrontend = true;
  };

  environment.etc."xdg/autostart/ibus-daemon.desktop" =
    lib.mkIf (niriActivo || hyprlandActivo)
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

  programs.niri.enable = niriActivo;

  programs.hyprland = lib.mkIf hyprlandActivo {
    enable = true;
    withUWSM = true;
    xwayland.enable = true;
  };

  services.xserver.desktopManager.cinnamon.enable = cinnamonActivo;
  services.desktopManager.plasma6.enable = plasmaActivo;

  environment.sessionVariables = lib.mkIf niriActivo {
    NIRI_CONFIG = "/etc/niri/config.kdl";
  };

  environment.etc."niri/config.kdl" = lib.mkIf niriActivo {
    source = ./niri.kdl;
  };

  environment.etc."korunix/niri-input.kdl" = lib.mkIf niriActivo {
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

  environment.etc."korunix/niri-output.kdl" = lib.mkIf niriActivo {
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

  # Este servicio existe si hay Niri o Hyprland, pero no arranca en Plasma o Cinnamon.
  systemd.user.services.noctalia = lib.mkIf noctaliaActivo {
    description = "Noctalia";
    partOf = ["graphical-session.target"];
    wantedBy = ["graphical-session.target"];
    after = ["graphical-session.target"];
    enableDefaultPath = false;

    environment.KORUNIX_NOCTALIA_BASE = "/etc/korunix/noctalia.toml";

    serviceConfig = {
      ExecCondition = sesionNoctalia;
      ExecStartPre = "${programa}/bin/korunix sesion preparar";
      ExecStart = lib.getExe noctaliaPackage;
      Restart = "on-failure";
    };
  };

  # Este puerto solo se abre cuando Sunshine está encendido.
  services.sunshine = {
    enable = sunshine.activo or false;
    openFirewall = sunshine.activo or false;
    autoStart = sunshine.autoinicio or false;
    capSysAdmin = sunshine.activo or false;
  };

  programs.localsend = lib.mkIf localsendActivo {
    enable = true;
    openFirewall = true;
  };

  programs.obs-studio = lib.mkIf obsActivo {
    enable = true;

    package = pkgs.obs-studio.override {
      cudaSupport = false;
    };

    plugins = with pkgs.obs-studio-plugins; [
      wlrobs
      obs-backgroundremoval
      obs-pipewire-audio-capture
      obs-vaapi
      obs-gstreamer
      obs-vkcapture
    ];

    enableVirtualCamera = true;
  };

  boot.extraModulePackages = lib.optionals obsActivo [
    config.boot.kernelPackages.v4l2loopback
  ];

  programs.steam = lib.mkIf (steam.activo or false) {
    enable = true;
    remotePlay.openFirewall = steam.remote_play or false;
    dedicatedServer.openFirewall = steam.servidor_dedicado or false;
  };

  programs.gamemode.enable = steam.activo or false;

  services.printing = {
    enable = impresion.activa or false;

    drivers =
      lib.optionals
      ((impresion.activa or false)
        && (impresion.controlador or null) == "epson-201207w")
      [pkgs.epson_201207w];
  };

  hardware.sane.enable = impresion.activa or false;

  programs.virt-manager.enable = virtualizacion.activa or false;
  virtualisation.libvirtd.enable = virtualizacion.activa or false;

  systemd.services.libvirt-default-network = lib.mkIf (virtualizacion.activa or false) {
    description = "Activa la red predeterminada de las máquinas virtuales";
    wantedBy = ["multi-user.target"];
    after = ["libvirtd.service"];

    serviceConfig = {
      Type = "oneshot";
      RemainAfterExit = true;
    };

    script = ''
      ${pkgs.libvirt}/bin/virsh net-autostart default

      if ! ${pkgs.libvirt}/bin/virsh net-info default | grep -q "Active:.*yes"; then
        ${pkgs.libvirt}/bin/virsh net-start default
      fi
    '';
  };

  environment.systemPackages =
    aplicaciones
    ++ [
      programa
      pkgs.git
      pkgs.just
      pkgs.tree
      pkgs.wget
      pkgs.udisks2
      pkgs.fwupd
    ]
    ++ lib.optionals scrcpyActivo [
      # scrcpy necesita adb, pero la persona no tiene que elegirlo dos veces.
      pkgs.android-tools
    ]
    ++ lib.optionals noctaliaActivo [
      noctaliaPackage
      pkgs.alacritty
      pkgs.nautilus
    ]
    ++ lib.optionals niriActivo [
      pkgs.xwayland-satellite
    ];

  system.stateVersion = "26.05";
}

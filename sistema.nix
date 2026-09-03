{
  ajustesAagl,
  almacenamiento,
  apariencia,
  aparienciaNoctalia,
  aplicaciones,
  aplicacionesElegidas,
  bluetooth,
  config,
  escritorio,
  escritorios,
  idioma,
  impresion,
  lib,
  monitor,
  noctaliaPackage,
  nombre,
  paquetesSpicetify,
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

  # Los escritorios traen sus herramientas normales sin convertir cada pieza
  # en otra pregunta de configuracion.toml.
  valentActivo = niriActivo || hyprlandActivo || cinnamonActivo;
  kdeConnectActivo = plasmaActivo;
  dispositivosActivos = valentActivo || kdeConnectActivo;

  aplicacionesNoctalia = with pkgs; [
    nautilus
    baobab
    gnome-characters
    gnome-clocks
    gnome-font-viewer
    gnome-maps
    gnome-text-editor
    gnome-weather
    loupe
    papers
    simple-scan
    snapshot
  ];

  aplicacionesNoctaliaCinnamon = with pkgs; [
    gnome-calculator
    gnome-calendar
    gnome-disk-utility
  ];

  aplicacionesCinnamon = with pkgs; [
    bulky
    warpinator
    xviewer
    xreader
    xed-editor
    pix
    celluloid
    gnome-screenshot
    file-roller
    gucharmap
    nemo-with-extensions
    cinnamon-control-center
    cinnamon-screensaver
    gnome-online-accounts-gtk
    onboard
  ];

  aplicacionesPlasma =
    (with pkgs.kdePackages; [
      ark
      dolphin
      elisa
      gwenview
      khelpcenter
      kinfocenter
      kmenuedit
      krdp
      kwalletmanager
      okular
      plasma-systemmonitor
      spectacle
      systemsettings
      qrca
      discover
    ])
    ++ lib.optionals (impresion.activa or false) [
      pkgs.kdePackages.print-manager
      pkgs.kdePackages.skanpage
    ];

  # Los paquetes pueden estar instalados a la vez, pero cada menú enseña solo
  # lo que pertenece a esa familia de escritorio.
  visibilidadEscritorios = pkgs.runCommand "korunix-visibilidad-escritorios" {} ''
        set -eu

        mkdir -p "$out/share/applications" "$out/etc/xdg/autostart"

        limitar_archivo() {
          origen="$1"
          destino="$2"
          escritorios="$3"

          [ -f "$origen" ] || return 0

          ${pkgs.gawk}/bin/awk \
            -v escritorios="$escritorios" \
            '
              /^\[Desktop Entry\]$/ {
                print
                print "OnlyShowIn=" escritorios ";"
                dentro = 1
                next
              }

              dentro && /^(OnlyShowIn|NotShowIn)=/ {
                next
              }

              /^\[/ {
                dentro = 0
              }

              {
                print
              }
            ' \
            "$origen" > "$destino"
        }

        limitar_paquete() {
          paquete="$1"
          escritorios="$2"
          directorio="$paquete/share/applications"

          [ -d "$directorio" ] || return 0

          for origen in "$directorio"/*.desktop; do
            [ -f "$origen" ] || continue
            limitar_archivo \
              "$origen" \
              "$out/share/applications/$(basename "$origen")" \
              "$escritorios"
          done
        }

        limitar_autostart() {
          paquete="$1"
          escritorios="$2"
          directorio="$paquete/etc/xdg/autostart"

          [ -d "$directorio" ] || return 0

          for origen in "$directorio"/*.desktop; do
            [ -f "$origen" ] || continue
            limitar_archivo \
              "$origen" \
              "$out/etc/xdg/autostart/$(basename "$origen")" \
              "$escritorios"
          done
        }

        for paquete in ${lib.escapeShellArgs (
      map toString (lib.optionals noctaliaActivo aplicacionesNoctalia)
    )}; do
          limitar_paquete "$paquete" "niri;Hyprland"
        done

        for paquete in ${lib.escapeShellArgs (
      map toString (
        lib.optionals
        (noctaliaActivo || cinnamonActivo)
        aplicacionesNoctaliaCinnamon
      )
    )}; do
          limitar_paquete "$paquete" "niri;Hyprland;X-Cinnamon"
        done

        for paquete in ${lib.escapeShellArgs (
      map toString (lib.optionals cinnamonActivo aplicacionesCinnamon)
    )}; do
          limitar_paquete "$paquete" "X-Cinnamon"
        done

        for paquete in ${lib.escapeShellArgs (
      map toString (lib.optionals plasmaActivo aplicacionesPlasma)
    )}; do
          limitar_paquete "$paquete" "KDE"
        done

        for paquete in ${lib.escapeShellArgs (
      map toString (
        lib.optionals cinnamonActivo [
          pkgs.blueman
          pkgs.networkmanagerapplet
        ]
      )
    )}; do
          limitar_autostart "$paquete" "X-Cinnamon"
        done

        for paquete in ${lib.escapeShellArgs (
      map toString (lib.optionals valentActivo [pkgs.valent])
    )}; do
          limitar_paquete "$paquete" "niri;Hyprland;X-Cinnamon"
          limitar_autostart "$paquete" "niri;Hyprland;X-Cinnamon"
        done

        for paquete in ${lib.escapeShellArgs (
      map toString (
        lib.optionals kdeConnectActivo [
          pkgs.kdePackages.kdeconnect-kde
        ]
      )
    )}; do
          limitar_paquete "$paquete" "KDE"
          limitar_autostart "$paquete" "KDE"
        done

        # El indicador pensado para escritorios no Plasma no se usa: fuera de KDE
        # la misma función la cubre Valent.
        indicador="$out/share/applications/org.kde.kdeconnect.nonplasma.desktop"

        if [ -f "$indicador" ]; then
          cat > "$indicador" <<'EOF'
    [Desktop Entry]
    Type=Application
    Name=KDE Connect Indicator
    Exec=/run/current-system/sw/bin/false
    Hidden=true
    NoDisplay=true
    EOF
        fi
  '';

  cohesionActivo = builtins.elem "cohesion" aplicacionesElegidas;
  figmaActivo = builtins.elem "figma-linux-next" aplicacionesElegidas;
  genshinActivo = builtins.elem "genshin-impact" aplicacionesElegidas;
  honkaiActivo = builtins.elem "honkai-star-rail" aplicacionesElegidas;
  localsendActivo = builtins.elem "localsend" aplicacionesElegidas;
  obsActivo = builtins.elem "obs-studio" aplicacionesElegidas;
  scrcpyActivo = builtins.elem "scrcpy" aplicacionesElegidas;
  spotifyActivo = builtins.elem "spotify" aplicacionesElegidas;

  cuentas = map (persona: persona.cuenta) personas;

  rutaHumanaValida = valor:
    valor
    != ""
    && !(lib.hasPrefix "/" valor)
    && !(builtins.elem ".." (lib.splitString "/" valor));

  avatarDe = persona: let
    nombreAvatar = persona.avatar or null;
    rutaAvatar =
      if nombreAvatar == null
      then null
      else ./. + "/${nombreAvatar}";
  in
    if nombreAvatar == null
    then null
    else if !rutaHumanaValida nombreAvatar
    then throw "El avatar de «${persona.cuenta}» tiene una ruta que no es segura."
    else if !builtins.pathExists rutaAvatar
    then throw "No encontré el avatar «${nombreAvatar}» de «${persona.cuenta}»."
    else rutaAvatar;

  clavesGithub = lib.concatMapStringsSep "\n" (persona: let
    clave = persona.clave_github or null;
  in
    lib.optionalString (clave != null) (
      if !rutaHumanaValida clave
      then throw "La clave de GitHub de «${persona.cuenta}» tiene una ruta que no es segura."
      else ''
        Match host github.com localuser ${persona.cuenta}
          IdentityFile /home/${persona.cuenta}/${clave}
          IdentitiesOnly yes
          AddKeysToAgent yes
      ''
    ))
  personas;

  hayAvatar = builtins.any (persona: avatarDe persona != null) personas;

  casosAvatar = lib.concatMapStrings (persona: let
    avatar = avatarDe persona;
  in
    lib.optionalString (avatar != null) ''
      ${persona.cuenta})
        avatar=${lib.escapeShellArg (toString avatar)}
        ;;
    '')
  personas;

  prepararPersona = pkgs.writeShellScript "korunix-prepara-persona" ''
    set -eu

    avatar=""

    case "''${USER:-}" in
    ${casosAvatar}
      *)
        exit 0
        ;;
    esac

    if [ -L "$HOME/.face" ]; then
      ${pkgs.coreutils}/bin/ln -sfn "$avatar" "$HOME/.face"
    elif [ ! -e "$HOME/.face" ]; then
      ${pkgs.coreutils}/bin/ln -s "$avatar" "$HOME/.face"
    fi
  '';

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

  nix.settings =
    {
      experimental-features = ["nix-command" "flakes"];
    }
    // lib.optionalAttrs (genshinActivo || honkaiActivo) ajustesAagl;
  nixpkgs.config.allowUnfree = true;

  hardware.enableRedistributableFirmware = true;

  # El arranque comprobado usa el kernel más reciente de la rama elegida,
  # una pantalla limpia y cinco segundos para entrar a recuperación.
  boot.kernelPackages = pkgs.linuxPackages_latest;
  boot.kernelParams = [
    "quiet"
    "splash"
    "boot.shell_on_fail"
  ];
  boot.plymouth.enable = true;
  boot.loader.timeout = 5;

  hardware.graphics = {
    enable = true;
    enable32Bit = pkgs.stdenv.hostPlatform.system == "x86_64-linux";
  };

  programs.appimage = {
    enable = true;
    binfmt = true;
  };

  services.flatpak = {
    enable = true;

    packages = lib.optionals cohesionActivo [
      {
        appId = "io.github.brunofin.Cohesion";
        commit = "a476d7d1dbee231266f9e904d878ec931bcafe6e37b14191430b5feb1d3da21e";
      }
    ];

    # Una generación revisada no cambia Flatpaks por detrás.
    update = {
      onActivation = false;
      auto.enable = false;
    };

    # Los Flatpaks instalados fuera de Korunix se conservan.
    uninstallUnmanaged = false;
  };

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
  programs.ssh = {
    startAgent = true;
    extraConfig = clavesGithub;
  };

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

  environment.sessionVariables =
    {
      # IBus mantiene la compatibilidad XIM sin forzar módulos de GTK ni Qt.
      XMODIFIERS = "@im=ibus";
    }
    // lib.optionalAttrs niriActivo {
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

  # Bluetooth es una decisión del equipo, no una consecuencia de usar Noctalia.
  hardware.bluetooth = {
    enable = bluetooth.activo or false;
    powerOnBoot = bluetooth.activo or false;
  };

  # Si Bluetooth está encendido, un mando Xbox compatible queda preparado sin
  # pedir otra opción técnica.
  hardware.xpadneo.enable = bluetooth.activo or false;

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
  systemd.user.services.korunix-persona = lib.mkIf hayAvatar {
    description = "Prepara las preferencias personales de Korunix";
    wantedBy = ["default.target"];

    serviceConfig = {
      Type = "oneshot";
      RemainAfterExit = true;
      ExecStart = prepararPersona;
    };
  };

  systemd.user.services.noctalia = lib.mkIf noctaliaActivo {
    description = "Noctalia";
    partOf = ["graphical-session.target"];
    wantedBy = ["graphical-session.target"];
    wants = lib.optionals hayAvatar ["korunix-persona.service"];
    after =
      ["graphical-session.target"]
      ++ lib.optionals hayAvatar ["korunix-persona.service"];
    enableDefaultPath = false;

    environment = {
      KORUNIX_NOCTALIA_BASE = "/etc/korunix/noctalia.toml";
      KORUNIX_NOCTALIA_SOURCE = aparienciaNoctalia.source;
      KORUNIX_NOCTALIA_MODE = aparienciaNoctalia.mode;
    };

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

  # Plasma usa KDE Connect. Niri, Hyprland y Cinnamon usan Valent.
  # Ambos implementan la misma función y Korunix abre sus puertos una sola vez.
  programs.kdeconnect = lib.mkIf dispositivosActivos {
    enable = true;

    # Plasma ya fija su propio paquete. Korunix solo aporta un valor
    # predeterminado para los demás escritorios.
    package = lib.mkDefault (
      if kdeConnectActivo
      then pkgs.kdePackages.kdeconnect-kde
      else pkgs.valent
    );
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

  programs.figma-linux-next.enable = figmaActivo;

  programs.anime-game-launcher.enable = genshinActivo;
  programs.honkers-railway-launcher.enable = honkaiActivo;

  programs.spicetify = lib.mkIf spotifyActivo {
    enable = true;
    wayland = true;
    spotifyLaunchFlags = "--enable-features=WaylandWindowDecorations,UseOzonePlatform";

    enabledExtensions = with paquetesSpicetify.extensions; [
      adblock
      spicyLyrics
      oneko
    ];
  };

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

      # Esta capa solo decide qué aparece en cada menú; no duplica decisiones.
      (lib.hiPrio visibilidadEscritorios)
    ]
    ++ lib.optionals scrcpyActivo [
      # scrcpy necesita adb, pero la persona no tiene que elegirlo dos veces.
      pkgs.android-tools
    ]
    ++ lib.optionals noctaliaActivo (
      [
        noctaliaPackage
        pkgs.alacritty
      ]
      ++ aplicacionesNoctalia
    )
    ++ lib.optionals
    (noctaliaActivo || cinnamonActivo)
    aplicacionesNoctaliaCinnamon
    ++ lib.optionals cinnamonActivo aplicacionesCinnamon
    ++ lib.optionals plasmaActivo aplicacionesPlasma
    ++ lib.optionals (kdeConnectActivo && valentActivo) [
      # programs.kdeconnect instala KDE Connect para Plasma; Valent queda
      # disponible aparte para las otras sesiones instaladas.
      pkgs.valent
    ]
    ++ lib.optionals niriActivo [
      pkgs.xwayland-satellite
    ];

  system.stateVersion = "26.05";
}

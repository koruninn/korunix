{
  description = "Korunix";

  inputs = {
    nixpkgs-estable.url = "github:NixOS/nixpkgs/nixos-26.05";
    nixpkgs-inestable.url = "github:NixOS/nixpkgs/nixos-unstable";

    millennium.url = "github:SteamClientHomebrew/Millennium?dir=packages/nix";

    figma-linux-next.url = "github:arximus88/figma-linux-next";

    hatter = {
      url = "github:Mibea/Hatter";
      flake = false;
    };

    aagl-inestable = {
      url = "github:ezKEa/aagl-gtk-on-nix";
      inputs.nixpkgs.follows = "nixpkgs-inestable";
    };

    aagl-estable = {
      url = "github:ezKEa/aagl-gtk-on-nix/release-26.05";
      inputs.nixpkgs.follows = "nixpkgs-estable";
    };

    nix-flatpak.url = "github:gmodena/nix-flatpak?ref=latest";

    spicetify-nix = {
      url = "github:Gerg-L/spicetify-nix";
      inputs.nixpkgs.follows = "nixpkgs-inestable";
    };
  };

  outputs = {
    aagl-estable,
    aagl-inestable,
    figma-linux-next,
    hatter,
    nix-flatpak,
    nixpkgs-estable,
    nixpkgs-inestable,
    millennium,
    spicetify-nix,
    ...
  }: let
    configuracion = builtins.fromTOML (builtins.readFile ./configuracion.toml);
    nombre = configuracion.nombre or "nixos";
    canal = configuracion.canal or "estable";
    personas = configuracion.personas or [];

    escritorioConfiguracion = configuracion.escritorio or {};
    escritorio = escritorioConfiguracion.principal or "niri";
    escritoriosDeclarados = escritorioConfiguracion.instalados or [];
    escritorios =
      if escritoriosDeclarados == []
      then [escritorio]
      else escritoriosDeclarados;

    apariencia =
      configuracion.apariencia or {
        estilo = "predeterminado";
        modo = "automatico";
      };

    aparienciaNoctalia = {
      source =
        if apariencia.estilo == "dinamico"
        then "wallpaper"
        else if apariencia.estilo == "everforest"
        then "community"
        else "builtin";

      mode =
        if apariencia.modo == "claro"
        then "light"
        else if apariencia.modo == "oscuro"
        then "dark"
        else "auto";
    };

    idioma =
      configuracion.idioma or {
        sistema = "español";
        region = "Perú";
      };

    teclado =
      configuracion.teclado or {
        distribuciones = ["españa" "latinoamérica"];
        cambio = "alt+shift";
      };

    monitor =
      configuracion.monitor or {
        resolucion = "1920x1080";
        hz = 120;
      };

    almacenamiento =
      configuracion.almacenamiento or {
        disponibles = [];
      };

    bluetooth =
      configuracion.bluetooth or {
        activo = false;
      };

    sunshine =
      configuracion.sunshine or {
        activo = false;
        autoinicio = false;
      };

    steam =
      configuracion.steam or {
        activo = false;
        remote_play = false;
        servidor_dedicado = false;
      };

    impresion =
      configuracion.impresion or {
        activa = false;
        controlador = null;
      };

    virtualizacion =
      configuracion.virtualizacion or {
        activa = false;
      };

    noctaliaActivo =
      builtins.any
      (nombreEscritorio: nombreEscritorio == "niri" || nombreEscritorio == "hyprland")
      escritorios;

    nixpkgs =
      if canal == "estable"
      then nixpkgs-estable
      else if canal == "inestable"
      then nixpkgs-inestable
      else throw "No conozco el canal «${canal}».";

    aagl =
      if canal == "estable"
      then aagl-estable
      else aagl-inestable;

    ajustesAagl = aagl.nixConfig;

    sistema = "x86_64-linux";

    pkgs = import nixpkgs {
      system = sistema;
      config.allowUnfree = true;
    };

    paquetesSpicetify = spicetify-nix.legacyPackages.${sistema};

    noctaliaPackage =
      if builtins.hasAttr "noctalia" pkgs
      then pkgs.noctalia
      else nixpkgs-inestable.legacyPackages.${sistema}.noctalia;

    aplicacionesElegidas = configuracion.aplicaciones.instaladas or [];

    # Estas aplicaciones siguen siendo elecciones normales. Solo cambia la forma
    # técnica de obtenerlas.
    aplicacionesEspeciales = {
      cohesion = {
        nombre = "Cohesion (Flatpak)";
        version = "";
      };

      "figma-linux-next" = {
        nombre = "Figma";
        version = figma-linux-next.packages.${sistema}.default.version or "";
      };

      "genshin-impact" = {
        nombre = "Genshin Impact";
        version = "";
      };

      "honkai-star-rail" = {
        nombre = "Honkai: Star Rail";
        version = "";
      };

      spotify = {
        nombre = "Spotify con Spicetify";
        version = "";
      };

      whatsapp = {
        nombre = "WhatsApp Web";
        version = "PWA";
      };
    };

    paquetePorNombre = elegida:
      if elegida == "kate"
      then pkgs.kdePackages.kate
      else if elegida == "kdenlive"
      then pkgs.kdePackages.kdenlive
      else if builtins.hasAttr elegida pkgs
      then builtins.getAttr elegida pkgs
      else null;

    resolver = elegida:
      if builtins.hasAttr elegida aplicacionesEspeciales
      then
        {
          inherit elegida;
          paquete = null;
        }
        // builtins.getAttr elegida aplicacionesEspeciales
      else let
        paquete = paquetePorNombre elegida;
      in
        if paquete != null
        then {
          inherit elegida paquete;
          nombre =
            if paquete ? pname && paquete.pname != null
            then builtins.toString paquete.pname
            else elegida;
          version =
            if paquete ? version && paquete.version != null
            then builtins.toString paquete.version
            else "";
        }
        else throw "No encontré «${elegida}» en Nixpkgs.";

    resueltas = map resolver aplicacionesElegidas;

    # LocalSend y OBS usan sus módulos de NixOS. El plan sí los resuelve para
    # enseñar nombre y versión, pero no se instalan dos veces.
    aplicacionesConModulo = ["localsend" "obs-studio"];

    aplicaciones =
      map
      (aplicacion: aplicacion.paquete)
      (builtins.filter
        (aplicacion:
          aplicacion.paquete
          != null
          && !(builtins.elem aplicacion.elegida aplicacionesConModulo))
        resueltas);

    planAplicaciones =
      map
      (aplicacion: builtins.removeAttrs aplicacion ["paquete"])
      resueltas;

    planPersonas =
      map (persona: {
        inherit (persona) cuenta;
        administrador = persona.administrador or false;
        avatar = persona.avatar or null;
        clave_github = persona.clave_github or null;
      })
      personas;

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
      };
      "latinoamérica" = {
        xkb = "latam";
        variante = "";
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

    programa = pkgs.rustPlatform.buildRustPackage {
      pname = "korunix";
      version = "0.1.0";
      src = ./.;
      cargoLock.lockFile = ./Cargo.lock;

      passthru.plan = {
        inherit nombre canal escritorio escritorios;
        personas = planPersonas;
        revision = nixpkgs.rev or "";
        aplicaciones = planAplicaciones;
        noctalia = noctaliaActivo;
        noctalia_version =
          if noctaliaActivo
          then noctaliaPackage.version or ""
          else "";

        apariencia = {
          estilo = apariencia.estilo;
          modo = apariencia.modo;
          noctalia_source = aparienciaNoctalia.source;
          noctalia_mode = aparienciaNoctalia.mode;
        };

        idioma = {
          sistema = idioma.sistema;
          region = idioma.region;
          inherit locale;
          zona_horaria = region.zonaHoraria;
        };

        teclado = {
          distribuciones = teclado.distribuciones;
          cambio = teclado.cambio;
          xkb = xkbLayouts;
          variantes = xkbVariantes;
        };

        monitor = {
          resolucion = monitor.resolucion;
          hz = monitor.hz;
        };

        entrada = {
          backend = "ibus";
          wayland = true;
        };

        almacenamiento =
          map
          (unidad: {
            nombre = unidad;
            ruta = "/mnt/${unidad}";
          })
          almacenamiento.disponibles;

        bluetooth = bluetooth.activo or false;

        sunshine = {
          activo = sunshine.activo or false;
          autoinicio = sunshine.autoinicio or false;
        };

        steam = {
          activo = steam.activo or false;
          remote_play = steam.remote_play or false;
          servidor_dedicado = steam.servidor_dedicado or false;
        };

        impresion = {
          activa = impresion.activa or false;
          controlador = impresion.controlador or null;
        };

        virtualizacion = virtualizacion.activa or false;
      };
    };

    programaInterfaz = pkgs.rustPlatform.buildRustPackage {
      pname = "korunix-interfaz";
      version = "0.1.0";
      src = ./.;
      cargoLock.lockFile = ./Cargo.lock;

      nativeBuildInputs = [
        pkgs.pkg-config
        pkgs.wrapGAppsHook4
      ];

      buildInputs = [
        pkgs.gtk4
        pkgs.libadwaita
      ];

      cargoBuildFlags = [
        "--features"
        "interfaz"
        "--bin"
        "korunix-interfaz"
      ];

      cargoInstallFlags = [
        "--features"
        "interfaz"
        "--bin"
        "korunix-interfaz"
      ];

      preFixup = ''
        gappsWrapperArgs+=(
          --set KORUNIX_MOTOR_BIN ${programa}/bin/korunix
        )
      '';

      postInstall = ''
        mkdir -p "$out/share/applications"

        cat > "$out/share/applications/io.github.koruninn.Korunix.desktop" <<'EOF'
        [Desktop Entry]
        Type=Application
        Name=Korunix
        Comment=Configura y mantiene NixOS
        Exec=korunix-interfaz
        Icon=preferences-system
        Terminal=false
        Categories=Settings;System;
        StartupNotify=true
        EOF
      '';
    };
  in {
    packages.${sistema} = {
      default = programa;
      korunix = programa;
      interfaz = programaInterfaz;

      aplicaciones = pkgs.buildEnv {
        name = "korunix-aplicaciones";
        paths = aplicaciones;
      };
    };

    nixosConfigurations.korunix = nixpkgs.lib.nixosSystem {
      system = sistema;

      specialArgs = {
        hatterSource = hatter;
        inherit
          ajustesAagl
          almacenamiento
          apariencia
          aparienciaNoctalia
          aplicaciones
          aplicacionesElegidas
          bluetooth
          escritorio
          escritorios
          idioma
          impresion
          monitor
          noctaliaPackage
          nombre
          paquetesSpicetify
          personas
          programa
          programaInterfaz
          steam
          sunshine
          teclado
          virtualizacion
          ;
      };

      modules = [
        {
          # Steam usa Millennium por debajo. El overlay se aplica al mismo
          # conjunto de paquetes que recibe sistema.nix.
          nixpkgs.overlays = [millennium.overlays.default];
        }
        aagl.nixosModules.default
        figma-linux-next.nixosModules.default
        nix-flatpak.nixosModules.nix-flatpak
        spicetify-nix.nixosModules.default
        ./sistema.nix
        ./apariencia.nix
      ];
    };

    formatter.${sistema} = pkgs.alejandra;
  };
}

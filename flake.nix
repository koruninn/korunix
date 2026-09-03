{
  description = "Korunix";

  inputs = {
    nixpkgs-estable.url = "github:NixOS/nixpkgs/nixos-26.05";
    nixpkgs-inestable.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = {
    nixpkgs-estable,
    nixpkgs-inestable,
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

    sunshine =
      configuracion.sunshine or {
        activo = false;
        autoinicio = false;
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

    sistema = "x86_64-linux";

    pkgs = import nixpkgs {
      system = sistema;
      config.allowUnfree = true;
    };

    noctaliaPackage =
      if builtins.hasAttr "noctalia" pkgs
      then pkgs.noctalia
      else nixpkgs-inestable.legacyPackages.${sistema}.noctalia;

    resolver = elegida:
      if builtins.hasAttr elegida pkgs
      then let
        paquete = builtins.getAttr elegida pkgs;
      in {
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

    resueltas = map resolver (configuracion.aplicaciones.instaladas or []);
    aplicaciones = map (aplicacion: aplicacion.paquete) resueltas;

    planAplicaciones =
      map
      (aplicacion: builtins.removeAttrs aplicacion ["paquete"])
      resueltas;

    planPersonas =
      map (persona: {
        inherit (persona) cuenta;
        administrador = persona.administrador or false;
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

        sunshine = {
          activo = sunshine.activo or false;
          autoinicio = sunshine.autoinicio or false;
        };

        impresion = {
          activa = impresion.activa or false;
          controlador = impresion.controlador or null;
        };

        virtualizacion = virtualizacion.activa or false;
      };
    };
  in {
    packages.${sistema} = {
      default = programa;
      korunix = programa;

      aplicaciones = pkgs.buildEnv {
        name = "korunix-aplicaciones";
        paths = aplicaciones;
      };
    };

    nixosConfigurations.korunix = nixpkgs.lib.nixosSystem {
      system = sistema;

      specialArgs = {
        inherit
          almacenamiento
          aplicaciones
          escritorio
          escritorios
          idioma
          impresion
          monitor
          noctaliaPackage
          nombre
          personas
          programa
          sunshine
          teclado
          virtualizacion
          ;
      };

      modules = [./sistema.nix];
    };

    formatter.${sistema} = pkgs.alejandra;
  };
}

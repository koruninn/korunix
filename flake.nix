{
  description = "Korunix";

  inputs = {
    # Este es el canal de NixOS que usan los equipos que siguen la versión en desarrollo.
    nixpkgs.url = "nixpkgs/nixos-unstable";

    # Este canal se usa para los equipos que prefieren una base más tranquila y estable.
    nixpkgs-stable.url = "nixpkgs/nixos-26.05";

    # Perfiles de hardware mantenidos por la comunidad de NixOS, incluido el de la ROG Ally RC71L.
    nixos-hardware = {
      url = "github:NixOS/nixos-hardware";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # Noctalia: panel, widgets, temas y ajustes de escritorio.
    noctalia = {
      url = "github:noctalia-dev/noctalia/";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # Código fuente de Umbriel, fijado por flake.lock.
    umbriel = { url = "github:noctalia-dev/umbriel/e65cc65ecec383528715d4bb50747b9f46199ab7"; };
    umbriel-src = {
      url = "github:noctalia-dev/umbriel";
      flake = false;
    };

    # Proporciona Steam Deck UI, Gamescope Session y el gestor real de sesiones.
    jovian = {
      url = "github:Jovian-Experiments/Jovian-NixOS";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # Código fuente del teclado virtual Sysboard, fijado por flake.lock.
    sysboard-src = {
      url = "github:System64fumo/sysboard";
      flake = false;
    };

    # libfprint con soporte SDCP para el lector LighTuning de la Ally.
    libfprint-egismoc-sdcp = {
      url = "github:TenSeventy7/libfprint-egismoc-sdcp/4d128d4f6f0b46182572126e84df88a73ac27859";
      flake = false;
    };

    # Parches específicos para el sensor 1c7a:0588.
    ally-fingerprint-patches = {
      url = "github:podothecat/decky-fingerprint-unlock/d5b077c3cfad098909a2849529a27aff55e5354f";
      flake = false;
    };

    # Añade los lanzadores que Korunix usa para algunos juegos de HoYo.
    aagl.url = "github:ezKEa/aagl-gtk-on-nix";
    aagl.inputs.nixpkgs.follows = "nixpkgs";

    # Deja instalar apps de Flatpak desde la configuración de NixOS.
    nix-flatpak.url = "github:gmodena/nix-flatpak?ref=latest";

    # Deja instalar Figma como una app integrada en Korunix.
    figma-linux-next.url = "github:arximus88/figma-linux-next";

    # Añade Millennium a Steam para usar la versión personalizada que Korunix deja lista.
    millennium.url = "github:SteamClientHomebrew/Millennium?dir=packages/nix";

    # Añade Spicetify a Spotify para poder cambiar su apariencia y funciones.
    spicetify-nix = {
      url = "github:Gerg-L/spicetify-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    nixpkgs,
    nixpkgs-stable,
    ...
  } @ inputs: let
    directorios = builtins.readDir ./equipos;
    equipos = builtins.filter (
      nombre: directorios.${nombre} == "directory"
    ) (builtins.attrNames directorios);

    crearEquipo = nombre: let
      equipoOriginal = import ./equipos/${nombre}/equipo.nix;

      cadenaObligatoria = campo:
        if !(builtins.hasAttr campo equipoOriginal)
        then throw "Falta ${campo} en equipos/${nombre}/equipo.nix."
        else let
          valor = builtins.getAttr campo equipoOriginal;
        in
          if builtins.isString valor && valor != ""
          then valor
          else throw "${campo} debe ser una cadena no vacía en equipos/${nombre}/equipo.nix.";

      canalEquipo = cadenaObligatoria "canal";
      arquitecturaEquipo = cadenaObligatoria "arquitectura";
      personaEquipo = cadenaObligatoria "persona";

      pantallaEquipo =
        if !(equipoOriginal ? pantalla)
        then null
        else let
          pantalla = equipoOriginal.pantalla;
          campos = ["nombre" "modo" "escala"];
          faltantes = builtins.filter (campo: !(builtins.hasAttr campo pantalla)) campos;
          nombreValido = builtins.isString pantalla.nombre && pantalla.nombre != "";
          modoValido = builtins.isString pantalla.modo && pantalla.modo != "";
          escalaValida =
            (builtins.isInt pantalla.escala || builtins.isFloat pantalla.escala)
            && pantalla.escala > 0;
        in
          if faltantes != []
          then throw "Pantalla incompleta en equipos/${nombre}/equipo.nix. Faltan: ${builtins.concatStringsSep ", " faltantes}."
          else if !nombreValido
          then throw "pantalla.nombre debe ser una cadena no vacía en equipos/${nombre}/equipo.nix."
          else if !modoValido
          then throw "pantalla.modo debe ser una cadena no vacía en equipos/${nombre}/equipo.nix."
          else if !escalaValida
          then throw "pantalla.escala debe ser un número mayor que cero en equipos/${nombre}/equipo.nix."
          else pantalla;

      equipo =
        equipoOriginal
        // {
          canal = canalEquipo;
          arquitectura = arquitecturaEquipo;
          persona = personaEquipo;
        }
        // (
          if pantallaEquipo == null
          then {}
          else {pantalla = pantallaEquipo;}
        );

      nixpkgsSeleccionado =
        if canalEquipo == "stable"
        then nixpkgs-stable
        else if canalEquipo == "unstable"
        then nixpkgs
        else throw "Canal no válido en equipos/${nombre}/equipo.nix: ${canalEquipo}. Usa stable o unstable.";
      system = arquitecturaEquipo;
      lib = nixpkgsSeleccionado.lib;
    in {
      name = nombre;
      value = lib.nixosSystem {
        inherit system;
        specialArgs = {
          inherit inputs equipo nombre;
          canal = canalEquipo;
        };
        modules = [
          ./equipos/${nombre}
          ./modulos/base
          {
            networking.hostName = nombre;
          }
        ];
      };
    };
  in {
    formatter = nixpkgs.lib.genAttrs ["x86_64-linux" "aarch64-linux"] (
      system: nixpkgs.legacyPackages.${system}.alejandra
    );

    nixosConfigurations = builtins.listToAttrs (map crearEquipo equipos);
  };
}

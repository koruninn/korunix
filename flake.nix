{
  description = "Korunix";

  inputs = {
    # Este es el canal de NixOS que usan los equipos que siguen la versión en desarrollo.
    nixpkgs.url = "nixpkgs/nixos-unstable";

    # Este canal se usa para los equipos que prefieren una base más tranquila y estable.
    nixpkgs-stable.url = "nixpkgs/nixos-26.05";

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

    # Añade Noctalia, que Korunix usa para el panel, widgets, temas y otras partes de la interfaz.
    noctalia = {
      url = "github:noctalia-dev/noctalia/";
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

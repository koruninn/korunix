{
  description = "Korunix";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    nixpkgs-stable.url = "nixpkgs/nixos-26.05";

    # AAGL acompaña automáticamente al canal de NixOS de cada equipo.
    aagl-unstable.url = "github:ezKEa/aagl-gtk-on-nix";
    aagl-unstable.inputs.nixpkgs.follows = "nixpkgs";
    aagl-stable.url = "github:ezKEa/aagl-gtk-on-nix/release-26.05";
    aagl-stable.inputs.nixpkgs.follows = "nixpkgs-stable";

    nix-flatpak.url = "github:gmodena/nix-flatpak?ref=latest";

    figma-linux-next.url = "github:arximus88/figma-linux-next";
    figma-linux-next.inputs.nixpkgs.follows = "nixpkgs";

    millennium.url = "github:SteamClientHomebrew/Millennium?dir=packages/nix";

    spicetify-nix = {
      url = "github:Gerg-L/spicetify-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

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

      personaOriginal =
        if !(equipoOriginal ? persona) || !builtins.isAttrs equipoOriginal.persona
        then throw "persona debe ser una ficha con al menos usuario en equipos/${nombre}/equipo.nix."
        else equipoOriginal.persona;

      usuarioPersona =
        if !(personaOriginal ? usuario) || !builtins.isString personaOriginal.usuario || personaOriginal.usuario == ""
        then throw "persona.usuario debe ser una cadena no vacía en equipos/${nombre}/equipo.nix."
        else personaOriginal.usuario;

      nombrePersona =
        if personaOriginal ? nombre
        then
          if builtins.isString personaOriginal.nombre && personaOriginal.nombre != ""
          then personaOriginal.nombre
          else throw "persona.nombre debe ser una cadena no vacía en equipos/${nombre}/equipo.nix."
        else usuarioPersona;

      perfilPersona = personaOriginal // {
        usuario = usuarioPersona;
        nombre = nombrePersona;
      };

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

      # Los archivos de equipo usan una ficha humana. El motor expone además
      # los nombres internos antiguos para que el resto de módulos no necesite
      # saber cómo está escrita esa ficha.
      equipo =
        (builtins.removeAttrs equipoOriginal ["persona"])
        // {
          canal = canalEquipo;
          arquitectura = arquitecturaEquipo;
          persona = usuarioPersona;
          nombre = nombrePersona;
          perfil = perfilPersona;
        }
        // (if personaOriginal ? foto then {foto = personaOriginal.foto;} else {})
        // (if pantallaEquipo == null then {} else {pantalla = pantallaEquipo;});

      nixpkgsSeleccionado =
        if canalEquipo == "stable"
        then nixpkgs-stable
        else if canalEquipo == "unstable"
        then nixpkgs
        else throw "Canal no válido en equipos/${nombre}/equipo.nix: ${canalEquipo}. Usa stable o unstable.";

      aaglSeleccionado =
        if canalEquipo == "stable"
        then inputs.aagl-stable
        else inputs.aagl-unstable;

      system = arquitecturaEquipo;
      lib = nixpkgsSeleccionado.lib;
    in {
      name = nombre;
      value = lib.nixosSystem {
        inherit system;
        specialArgs = {
          inherit inputs equipo nombre;
          aagl = aaglSeleccionado;
          canal = canalEquipo;
        };
        modules = [
          ./equipos/${nombre}
          ./modulos/base
          {networking.hostName = nombre;}
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

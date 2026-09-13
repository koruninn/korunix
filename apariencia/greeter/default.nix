{
  config,
  equipo,
  pkgs,
  ...
}: let
  # La revisión de nixpkgs usada actualmente por Korunix todavía trae
  # noctalia-greeter 1.3.1. La sincronización restringida sin contraseña
  # requiere 1.5.0 o superior, así que actualizamos solo este paquete sin
  # mover el resto del sistema.
  greeterPackage = pkgs.noctalia-greeter.overrideAttrs (old: {
    version = "1.5.0";

    src = pkgs.fetchFromGitHub {
      owner = "noctalia-dev";
      repo = "noctalia-greeter";
      tag = "v1.5.0";
      hash = "sha256-JgPgbmlUOKlgCX/KDfRF+z9ID80+Q7CcdaJFh5eaFjU=";
    };

    buildInputs = (old.buildInputs or []) ++ [
      pkgs.libxml2
    ];
  });
in {
  # Korunix usa Noctalia Greeter sobre greetd como pantalla de inicio de sesión.
  # El módulo oficial de NixOS habilita greetd, Polkit y AccountsService.
  services.displayManager.noctalia-greeter = {
    enable = true;
    package = greeterPackage;

    settings = {
      keyboard = {
        layout = "es";
        variant = "deadtilde";
      };

      cursor.size = 24;
    };

    cursorTheme = {
      package = pkgs.bibata-cursors;
      name = "Bibata-Modern-Ice";
    };
  };

  # Noctalia sincroniza fondo, paleta y disposición de monitores mediante un
  # helper muy limitado. Autorizamos sin contraseña únicamente esa operación
  # concreta para la persona principal del equipo; no concede privilegios
  # generales ni acceso sin contraseña a otros comandos administrativos.
  security.polkit = {
    enable = true;
    enablePkexecWrapper = true;
    extraConfig = ''
      polkit.addRule(function(action, subject) {
        var allowedUsers = ["${equipo.persona}"];

        if (action.id == "org.noctalia.greeter.sync-appearance" &&
            action.lookup("program") == "${greeterPackage}/bin/noctalia-greeter-apply-appearance" &&
            action.lookup("user") == "root" &&
            subject.local && subject.active &&
            allowedUsers.indexOf(subject.user) >= 0) {
          return polkit.Result.YES;
        }
      });
    '';
  };
}

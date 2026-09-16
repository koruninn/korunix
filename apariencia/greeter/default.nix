{
  equipo,
  inputs,
  pkgs,
  ...
}: let
  system = pkgs.stdenv.hostPlatform.system;
  greeterPackage = inputs.noctalia-greeter.packages.${system}.default;
in {
  # Korunix usa Noctalia Greeter sobre greetd como pantalla de inicio de sesión.
  # La versión exacta queda registrada en flake.lock y se actualiza con Korunix.
  services.displayManager.noctalia-greeter = {
    enable = true;
    package = greeterPackage;

    settings = {
      keyboard = {
        layout = "es";
        variant = "deadtilde";
        numlock = true;
      };

      cursor.size = 24;
    };

    cursorTheme = {
      package = pkgs.bibata-cursors;
      name = "Bibata-Modern-Classic";
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

{
  config,
  equipo,
  pkgs,
  ...
}: let
  greeterPackage = config.services.displayManager.noctalia-greeter.package;
in {
  # Korunix usa Noctalia Greeter sobre greetd como pantalla de inicio de sesión.
  # El módulo oficial de NixOS habilita greetd, Polkit y AccountsService.
  services.displayManager.noctalia-greeter = {
    enable = true;

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

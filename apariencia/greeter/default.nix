{
  pkgs,
  ...
}: {
  # Korunix usa Noctalia Greeter sobre greetd como pantalla de inicio de sesión.
  # El módulo oficial de NixOS habilita también Polkit y AccountsService.
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
}

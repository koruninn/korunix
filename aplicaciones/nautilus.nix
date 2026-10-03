{...}: {
  # Esto integra "Abrir en terminal" en Nautilus sin depender de una sesión GNOME.
  # El bloque de NixOS instala esta extensión, deja la ruta correcta para Nautilus 4
  # y pone Alacritty como terminal de Korunix.
  programs.nautilus-open-any-terminal = {
    enable = true;
    terminal = "alacritty";
  };
}

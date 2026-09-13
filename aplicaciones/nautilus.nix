{...}: {
  # Integra "Abrir en terminal" en Nautilus sin depender de una sesión GNOME.
  # El módulo de NixOS instala la extensión, expone la ruta correcta para
  # Nautilus 4 y fija Alacritty como terminal de Korunix.
  programs.nautilus-open-any-terminal = {
    enable = true;
    terminal = "alacritty";
  };
}

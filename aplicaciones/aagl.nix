{inputs, ...}: {
  # Este bloque deja instalado lo que necesitan los juegos de HoYo y solo
  # activa los lanzadores que este equipo usa de verdad.
  imports = [
    inputs.aagl.nixosModules.default
  ];

  programs.anime-game-launcher.enable = true; # Launcher de Genshin Impact.
  programs.honkers-railway-launcher.enable = true; # Launcher de Honkai: Star Rail.
  # programs.honkers-launcher.enable = false; # Lanzador de Honkai Impact 3rd, actualmente apagado.
  # programs.wave-launcher.enable = false; # Lanzador de Wuthering Waves, actualmente apagado.
  # programs.sleepy-launcher.enable = false; # Launcher de Zenless Zone Zero, actualmente desactivado.
}

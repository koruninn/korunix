{inputs, ...}: {
  imports = [
    inputs.aagl.nixosModules.default
  ];

  programs.anime-game-launcher.enable = true; # Genshin Impact.
  programs.honkers-railway-launcher.enable = true; # Honkai: Star Rail.
}

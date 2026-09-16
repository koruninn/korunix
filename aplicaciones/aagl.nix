{
  aagl,
  ...
}: {
  imports = [
    aagl.nixosModules.default
  ];

  programs.anime-game-launcher.enable = true; # Genshin Impact
  programs.honkers-railway-launcher.enable = true; # Honkai: Star Rail
  # programs.honkers-launcher.enable = false;          # Honkai Impact 3rd
  # programs.wave-launcher.enable = false;             # Wuthering Waves
  # programs.sleepy-launcher.enable = false;           # Zenless Zone Zero
}

{ ... }: {
  # Habilita el shell Fish y sus integraciones nativas en NixOS.
  programs.fish = {
    enable = true;
  };
}

{ ... }: {
  # 1. Habilita el shell Fish y sus integraciones nativas en NixOS
  programs.fish = {
    enable = true;
  };

  # 2. Crea el enlace simbólico mutable hacia tus dotfiles en cada arranque
  systemd.tmpfiles.rules = [
    "L+ /home/koru/.config/fish/config.fish - - - - /home/koru/.korunix/config.fish"
  ];
}

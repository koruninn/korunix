{ ... }: {
  # Desactiva o inicializa en limpio los métodos de entrada (como Fcitx/IBus) a nivel global
  i18n.inputMethod = {
    enable = true;
    type = "none";
  };
}

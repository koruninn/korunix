{pkgs, ...}: {
  services.pulseaudio.enable = false;
  security.rtkit.enable = true;

  services.pipewire = {
    enable = true;
    alsa.enable = true;
    # Steam y otros programas x86 pueden necesitar bibliotecas de audio de
    # 32 bits. En otras arquitecturas no intentamos habilitar una compatibilidad
    # que no corresponde a la plataforma.
    alsa.support32Bit = pkgs.stdenv.hostPlatform.isx86_64;
    pulse.enable = true;
  };
}

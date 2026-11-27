{pkgs, ...}: {
  services.pulseaudio.enable = false;
  security.rtkit.enable = true;

  services.pipewire = {
    enable = true;
    alsa.enable = true;
    # Steam puede necesitar audio de 32 bits en equipos x86.
    alsa.support32Bit = pkgs.stdenv.hostPlatform.isx86_64;
    pulse.enable = true;
  };
}

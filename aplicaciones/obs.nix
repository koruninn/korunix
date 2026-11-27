{
  config,
  pkgs,
  ...
}: {
  programs.obs-studio = {
    enable = true;

    package = (
      pkgs.obs-studio.override {
        cudaSupport = false;
      }
    );

    plugins = with pkgs.obs-studio-plugins; [
      wlrobs
      obs-backgroundremoval
      obs-pipewire-audio-capture
      obs-vaapi # Aceleración de video para gráficos AMD.
      obs-gstreamer
      obs-vkcapture
    ];

    enableVirtualCamera = true;
  };

  boot.extraModulePackages = [config.boot.kernelPackages.v4l2loopback];
}

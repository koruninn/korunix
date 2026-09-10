{
  config,
  pkgs,
  ...
}: {
  programs.obs-studio = {
    enable = true;

    # optional Nvidia hardware acceleration
    package = (
      pkgs.obs-studio.override {
        cudaSupport = false;
      }
    );

    plugins = with pkgs.obs-studio-plugins; [
      wlrobs
      obs-backgroundremoval
      obs-pipewire-audio-capture
      obs-vaapi #optional AMD hardware acceleration
      obs-gstreamer
      obs-vkcapture
    ];

    enableVirtualCamera = true;
  };

  boot.extraModulePackages = [config.boot.kernelPackages.v4l2loopback];
}

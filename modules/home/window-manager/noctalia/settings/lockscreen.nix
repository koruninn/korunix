{...}: {
  programs.noctalia.settings = {
    lockscreen = {
      enabled = false;
      fingerprint = false;
      allow_empty_password = false;
      blurred_desktop = true;
      blur_intensity = 0.5;
      tint_intensity = 0.6;
      wallpaper = "~/.korunix/window-manager/noctalia/wallpapers/dark/capriccio-arco-rovinato-e-una-villa-nello-sfondo.jpg";
      monitors = [];
    };
  };
}

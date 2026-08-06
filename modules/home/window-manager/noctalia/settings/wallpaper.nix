{lib, ...}: {
  programs.noctalia.settings = {
    wallpaper = {
      enabled = true;
      fill_mode = "crop";
      fill_color = "#111111";

      transition = ["fade" "wipe" "disc" "stripes" "zoom" "honeycomb"];
      transition_duration = 1500;
      edge_smoothness = 0.3;
      transition_on_startup = false;

      # CORRECCIÓN DE RUTAS REALES (Añadido /modules/home/)
      directory = "/home/koru/.korunix/modules/home/window-manager/noctalia/wallpapers";
      directory_light = "/home/koru/.korunix/modules/home/window-manager/noctalia/wallpapers/light";
      directory_dark = "/home/koru/.korunix/modules/home/window-manager/noctalia/wallpapers/dark";
      per_monitor_directories = false;

      default = {
        path = "/home/koru/.korunix/modules/home/window-manager/noctalia/wallpapers/dark/capriccio-arco-rovinato-e-una-villa-nello-sfondo.jpg";
      };

      automation = {
        enabled = false;
        interval_seconds = 1800;
        order = "random";
        recursive = true;
      };
    };
  };
}

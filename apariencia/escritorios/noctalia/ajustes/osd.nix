{...}: {
  programs.noctalia.settings = {
    osd = {
      position = "top_center";
      position_vertical = "top_center";
      orientation = "horizontal";
      scale = 1.0;
      background_opacity = 0.50;
      offset_x = 20;
      offset_y = 8;
      # monitors = [ "DP-1" ]; # Descomenta y ajusta si quieres limitarlo a un monitor

      kinds = {
        volume = true;
        volume_output = true;
        volume_input = true;
        brightness = true;
        wifi = true;
        bluetooth = true;
        power_profile = true;
        caffeine = true;
        nightlight = true;
        dnd = true;
        lock_keys = true;
        keyboard_layout = true;
        privacy = true;
      };
    };
  };
}

{...}: {
  programs.noctalia.settings = {
    dock = {
      enabled = true; # set true to activate
      position = "bottom"; # top | bottom | left | right
      active_monitor_only = false; # when true, only show apps/windows from the active monitor
      monitors = []; # connector names to show on; empty = all outputs

      icon_size = 40;
      main_axis_padding = 16; # inner padding along the icon row (main axis)
      cross_axis_padding = 8; # inner padding perpendicular to the icon row
      item_spacing = 6; # gap between items in pixels
      background_opacity = 0.50;
      shadow = false; # cast the global [shell.shadow]

      radius = 16;
      radius_top_left = 16; # optional per-corner overrides
      radius_top_right = 16;
      radius_bottom_left = 16;
      radius_bottom_right = 16;

      margin_ends = 0; # inset from each end of the dock along its main axis
      margin_edge = 8; # distance from the nearest screen edge (positive values float the dock)

      show_running = true; # also show running apps not in the pinned list
      auto_hide = false; # fade out when pointer leaves; fade in on approach
      reserve_space = true; # reserve compositor exclusive zone / push windows away

      active_scale = 1.0; # icon scale for the focused app (clamped 0.1–1.75)
      inactive_scale = 0.85; # icon scale for non-focused apps (clamped 0.1–1.0)
      magnification = true; # magnify icons near the pointer (macOS-style)
      magnification_scale = 1.35; # max scale multiplier at the pointer center (1.0–2.0; 1.0 = off)
      active_opacity = 1.0;
      inactive_opacity = 0.85;
      show_instance_count = true; # badge with window count when an app has 2+ windows
      show_dots = true; # running-window indicator dots beside app icons

      launcher_position = "none"; # none | start | end — optional launcher button on the dock
      launcher_icon = "grid-dots"; # Tabler glyph for the launcher button

      # Desktop entry IDs, StartupWMClass, or human-readable names
      pinned = ["firefox" "org.gnome.Nautilus" "spotify" "steam" "net.lutris.Lutris" "moe.launcher.an-anime-game-launcher" "moe.launcher.the-honkers-railway-launcher" "vesktop" "affinity.exe" "localsend_app" "code" "com.obsproject.Studio" "org.kde.kdenlive" "heroic" "ONLYOFFICE" "birdfont"];
    };
  };
}

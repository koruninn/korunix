{...}: {
  programs.noctalia.settings = {
    bar = {
      order = ["default"]; # layer-shell creation order

      default = {
        position = "top"; # top | bottom | left | right
        enabled = true;
        auto_hide = false; # slide out after pointer leaves; reveal from edge trigger strip
        reserve_space = true; # reserve compositor exclusive zone / push windows away
        layer = "top"; # top | overlay; overlay appears above fullscreen apps

        thickness = 34; # bar cross-axis size in pixels (height for horizontal, width for vertical)
        background_opacity = 0.5; # 0.0 (transparent) to 1.0 (opaque)
        border = "outline"; # color role or #RRGGBB for the bar outline
        border_width = 0.0; # inside outline width in pixels; 0 disables it
        shadow = false; # cast the global [shell.shadow]
        contact_shadow = false; # dark gradient between an attached panel and the bar (depth at the seam)
        panel_overlap = 1; # logical px an attached panel overlaps the bar edge to hide the seam

        radius = 12; # global corner radius fallback
        radius_top_left = 12;
        radius_top_right = 12;
        radius_bottom_left = 12;
        radius_bottom_right = 12;

        margin_ends = 180; # inset from each end of the bar along its main axis
        margin_edge = 10; # distance from the nearest screen edge (positive values float the bar)
        margin_opposite_edge = 0; # extra reserved space on the inward side of the bar (below for top, above for bottom)
        padding = 14; # main-axis padding from bar edges to start/end widget sections
        widget_spacing = 12; # gap between widgets within a section

        scale = 1.0; # content scale multiplier for icons and text
        font_weight = "regular"; # "regular" | "bold" — primary label weight for bar widgets
        font_family = ""; # typeface for this bar's widgets; empty inherits the global font

        # Default capsule style for all widgets on this bar (see Widget Capsule section)
        capsule = false;
        capsule_fill = "surface_variant";
        capsule_thickness = 0.76; # capsule size across the bar as a fraction of bar thickness (1.0 fills the bar)
        capsule_radius = 8.0; # omit for automatic pill radius
        capsule_opacity = 1.0;
        # capsule_border   = "outline";   # omit this key for no border by default

        start = ["launcher" "wallpaper" "workspaces"];
        center = ["clock"];
        end = [
          "media"
          "tray"
          "notifications"
          "clipboard"
          "network"
          "bluetooth"
          "volume"
          "brightness"
          "battery"
          "control-center"
          "session"
        ];
      };
    };
  };
}

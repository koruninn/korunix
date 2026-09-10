{...}: {
  programs.noctalia.settings = {
    shell = {
      ui_scale = 0.9;
      corner_radius_scale = 1.0;
      font_family = "sans-serif";
      lang = "es";
      time_format = "{:%H:%M:%S}";
      date_format = "%A, %x";
      offline_mode = false;
      telemetry_enabled = false;
      setup_wizard_enabled = true;
      niri_overview_type_to_launch_enabled = false;
      polkit_agent = true;
      password_style = "default";
      avatar_path = "~/.korunix/modules/home/window-manager/noctalia/.face/avatar.jpg";
      settings_show_advanced = false;
      middle_click_opens_widget_settings = true;
      show_location = true;
      launch_apps_as_systemd_services = false;
      screen_time_enabled = false;
      app_icon_colorize = false;
      app_icon_color = "on_surface";

      clipboard_enabled = true;
      clipboard_history_max_entries = 100;
      clipboard_confirm_clear_history = true;
      clipboard_auto_paste = "auto";
      clipboard_image_action_command = "";

      shared_gl_context = true;
      disable_mipmaps = true;

      animation = {
        enabled = true;
        speed = 1.0;
      };

      shadow = {
        direction = "down";
        alpha = 0.55;
      };

      panel = {
        transparency_mode = "transparent";
        borders = true;
        shadow = false;
        launcher_placement = "centered";
        clipboard_placement = "centered";
        control_center_placement = "attached";
        wallpaper_placement = "attached";
        session_placement = "attached";
        floating_offset = 8;
        open_near_click_control_center = false;
        open_near_click_launcher = false;
        launcher_categories = true;
        launcher_show_icons = true;
        launcher_compact = false;
        launcher_session_search = false;
        launcher_sort_by_usage = true;
        open_near_click_clipboard = false;
        open_near_click_wallpaper = false;
        open_near_click_session = false;
      };

      screen_corners = {
        enabled = false;
        size = 32;
      };

      mpris = {
        blacklist = [];
      };

      screenshot = {
        save_to_file = true;
        directory = "~/Imágenes/Capturas de pantalla";
        filename_pattern = "screenshot_%Y%m%d_%H%M%S";
        copy_to_clipboard = true;
        freeze_screen = true;
        pipe_to_command = false;
        pipe_command = "";
      };
    };
  };
}

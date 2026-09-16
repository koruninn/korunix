{
  equipo,
  lib,
  plugins,
}: let
  compartido = import ../compartido.nix;
in {
  # ESTE ES EL ARCHIVO FÁCIL DE NOCTALIA.
  # Aquí viven los ajustes que Korunix quiere conservar después de un rebuild.
  # No hay que editar el "motor" noctalia.nix para mover widgets o cambiar el dock.

  accessibility = {
    ui_scale = 0.9;
    high_contrast = false;
  };

  shell = {
    corner_radius_scale = 1.0;
    font_family = compartido.tipografia;
    lang = "es";
    time_format = "{:%H:%M:%S}";
    date_format = "%A, %x";
    offline_mode = false;
    telemetry_enabled = false;
    setup_wizard_enabled = false;
    polkit_agent = true;
    password_style = "default";
    avatar_path =
      if equipo ? foto
      then "${equipo.foto}"
      else "";
    settings_show_advanced = false;
    show_location = true;
    launch_apps_as_systemd_services = false;
    screen_time_enabled = true;
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

    greeter_sync.auto_sync = true;

    panel = {
      transparency_mode = "glass";
      borders = true;
      shadow = false;
      launcher_placement = "floating";
      clipboard_placement = "floating";
      control_center_placement = "attached";
      wallpaper_placement = "attached";
      session_placement = "attached";
      floating_offset = 8;
      open_near_click_control_center = false;
      open_near_click_launcher = false;
      open_near_click_clipboard = false;
      open_near_click_wallpaper = false;
      open_near_click_session = false;
    };

    screen_corners = {
      enabled = true;
      size = 32;
    };

    mpris.blacklist = [];
    launcher.sort_by_usage = false;

    screenshot = {
      save_to_file = true;
      filename_pattern = "Captura de pantalla %Y-%m-%d %H-%M-%S";
      copy_to_clipboard = true;
      freeze_screen = true;
      confirm_region = true;
      annotate = true;
    };
  };

  location.auto_locate = true;

  control_center = {
    sidebar = "full";
    sidebar_section = "full";
    width = 700;
    show_shortcut_labels = true;
    show_session_button = true;
    hidden_tabs = [];

    shortcuts = map (type: {inherit type;}) [
      "bluetooth"
      "notification"
      "power_profile"
      "screen_recorder"
      "nightlight"
      "dark_mode"
    ];
  };

  bar = {
    order = ["default"];

    default = {
      position = "top";
      enabled = true;
      auto_hide = false;
      reserve_space = true;
      layer = "top";
      thickness = 34;
      background_opacity = 0.35;
      border = "outline";
      border_width = 0.0;
      shadow = true;
      contact_shadow = true;
      panel_overlap = 1;
      radius = 24;
      radius_top_left = 24;
      radius_top_right = 24;
      radius_bottom_left = 24;
      radius_bottom_right = 24;
      margin_ends = 15;
      margin_edge = 10;
      margin_opposite_edge = 0;
      padding = 14;
      widget_spacing = 12;
      scale = 0.9;
      font_weight = "regular";
      font_family = "";
      capsule = false;
      capsule_fill = "surface_variant";
      capsule_thickness = 0.76;
      capsule_radius = 8.0;
      capsule_opacity = 1.0;
      start = ["workspaces" "fecha" "clock" "cat" "temperatura" "wallpaper"];
      center = ["bongo_cat" "media" "audio_visualizer"];
      end = [
        "tray"
        "notifications"
        "clipboard"
        "calculator"
        "pomodoro_timer"
        "notes"
        "umbriel_companion"
        "udiskie_manager"
        "network"
        "bluetooth"
        "lock_keys"
        "volume"
        "volume_input"
      ];
    };
  };

  widget = {
    fecha = {
      type = "clock";
      format = "{:%A, %d de %B}";
    };
    weather = {
      type = "weather";
      max_length = 250;
      anchor = true;
    };
    media = {
      type = "media";
      max_length = 400;
      hide_when_no_media = true;
    };
    cat = {
      type = "dotnetrob/cat:cat";
      show_cpu_percent = true;
      interactive = false;
    };
    bongo_cat = {
      type = "noctalia/bongocat:cat";
      audio_spectrum = true;
      tappy_mode = true;
      enable_scroll = false;
    };
    calculator.type = "yuuto/calculator:bar";
    pomodoro_timer.type = "thepunkoff/pomodoro:widget";
    notes.type = "noctalia/notes:notes";
    udiskie_manager.type = "aristides/udiskie:status";
    tray = {
      type = "tray";
      drawer = true;
    };
    network = {
      type = "network";
      show_label = false;
    };
    bluetooth = {
      type = "bluetooth";
      hide_when_adapter_off = true;
    };
    lock_keys = {
      type = "lock_keys";
      hide_when_off = true;
      display = "full";
    };
    volume_input = {
      type = "volume";
      device = "input";
    };
    temperatura = {
      type = "sysmon";
      stat = "cpu_temp";
    };
    umbriel_displays.type = "prponkshe/umbriel-displays:bar";
    umbriel_companion.type = "noctalia/umbriel-companion:bar";
    speedtest_meter.type = "nilsonlinux/speedtest-meter:speedtest-widget";
    phone_connect.type = "icefish/phone-connect:bar";
    printers.type = "andrewdems/printers:printer";
    red_rx = {
      type = "sysmon";
      stat = "net_rx";
    };
    red_tx = {
      type = "sysmon";
      stat = "net_tx";
    };
    privacy = {
      type = "privacy";
      hide_inactive = true;
    };
    screen_recorder.type = "noctalia/screen_recorder:recorder";
  };

  plugin_settings."noctalia/notes" = {
    extension = "md";
    panel_placement = "floating";
    panel_position = "center_right";
    panel_layer = "top";
  };

  dock = {
    enabled = true;
    position = "bottom";
    active_monitor_only = false;
    monitors = lib.optional (equipo ? pantalla) equipo.pantalla.nombre;
    icon_size = 40;
    main_axis_padding = 16;
    cross_axis_padding = 8;
    item_spacing = 6;
    background_opacity = 0.35;
    shadow = true;
    radius = 24;
    radius_top_left = 24;
    radius_top_right = 24;
    radius_bottom_left = 24;
    radius_bottom_right = 24;
    margin_ends = 0;
    margin_edge = 8;
    show_running = true;
    auto_hide = false;
    reserve_space = true;
    active_scale = 1.0;
    inactive_scale = 0.85;
    magnification = true;
    magnification_scale = 1.35;
    active_opacity = 1.0;
    inactive_opacity = 0.85;
    show_instance_count = true;
    show_dots = true;
    launcher_position = "none";
    launcher_icon = "grid-dots";
    pinned = map (entrada: entrada.noctalia) compartido.dock;
  };

  osd = {
    position = "top_center";
    position_vertical = "top_center";
    orientation = "horizontal";
    scale = 1.0;
    background_opacity = 0.30;
    offset_x = 20;
    offset_y = 8;
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

  lockscreen = {
    enabled = true;
    blurred_desktop = true;
    blur_intensity = 0.5;
    tint_intensity = 0.6;
    wallpaper = "${./fondos/oscuro}/capriccio-arco-rovinato-e-una-villa-nello-sfondo.jpg";
    monitors = [];
  };

  calendar = {
    enabled = true;
    refresh_minutes = 15;
  };

  # Solo apuntamos a los fondos existentes. No se modifica ninguna imagen ni
  # el motor de sincronización de fondos.
  wallpaper = {
    enabled = true;
    fill_mode = "crop";
    fill_color = "#111111";
    transition = ["fade" "wipe" "disc" "stripes" "zoom" "honeycomb"];
    transition_duration = 1500;
    edge_smoothness = 0.3;
    transition_on_startup = false;
    directory = "${./fondos}";
    directory_light = "${./fondos/claro}";
    directory_dark = "${./fondos/oscuro}";
    per_monitor_directories = false;
    default.path = "${./fondos/oscuro}/capriccio-arco-rovinato-e-una-villa-nello-sfondo.jpg";
    automation = {
      enabled = false;
      interval_seconds = 1800;
      order = "random";
      recursive = true;
    };
  };

  plugins.enabled = plugins.activos;

  theme = {
    mode = "auto";
    source = "wallpaper";
    builtin = "Noctalia";
    wallpaper_scheme = "m3-tonal-spot";
    templates = {
      enable_builtin_templates = true;
      builtin_ids = ["alacritty" "gtk3" "gtk4" "qt" "umbriel"];
      enable_community_templates = true;
      community_ids = ["blender" "darktable" "discord" "gimp" "heroiclauncher" "inkscape" "libreoffice" "obs" "obsidian" "prismlauncher" "pywalfox" "steam" "vscode"];
    };
  };
}

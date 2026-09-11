{
  config,
  pkgs,
  ...
}:

let
  noctaliaConfig = pkgs.writeText "noctalia-config.toml" ''
    [shell]
    ui_scale = 0.9
    corner_radius_scale = 1.0
    font_family = "sans-serif"
    lang = "es"
    time_format = "{:%H:%M:%S}"
    date_format = "%A, %x"
    offline_mode = false
    telemetry_enabled = false
    setup_wizard_enabled = true
    niri_overview_type_to_launch_enabled = false
    polkit_agent = true
    password_style = "default"
    avatar_path = "${./.face}/avatar.jpg"
    settings_show_advanced = false
    middle_click_opens_widget_settings = true
    show_location = true
    launch_apps_as_systemd_services = false
    screen_time_enabled = false
    app_icon_colorize = false
    app_icon_color = "on_surface"
    clipboard_enabled = true
    clipboard_history_max_entries = 100
    clipboard_confirm_clear_history = true
    clipboard_auto_paste = "auto"
    clipboard_image_action_command = ""
    shared_gl_context = true
    disable_mipmaps = true

    [shell.animation]
    enabled = true
    speed = 1.0

    [shell.shadow]
    direction = "down"
    alpha = 0.55

    [shell.panel]
    transparency_mode = "transparent"
    borders = true
    shadow = false
    launcher_placement = "centered"
    clipboard_placement = "centered"
    control_center_placement = "attached"
    wallpaper_placement = "attached"
    session_placement = "attached"
    floating_offset = 8
    open_near_click_control_center = false
    open_near_click_launcher = false
    launcher_categories = true
    launcher_show_icons = true
    launcher_compact = false
    launcher_session_search = false
    launcher_sort_by_usage = true
    open_near_click_clipboard = false
    open_near_click_wallpaper = false
    open_near_click_session = false

    [shell.screen_corners]
    enabled = false
    size = 32

    [shell.mpris]
    blacklist = []

    [shell.screenshot]
    save_to_file = true
    filename_pattern = "screenshot_%Y%m%d_%H%M%S"
    copy_to_clipboard = true
    freeze_screen = true
    pipe_to_command = false
    pipe_command = ""

    [bar]
    order = ["default"]

    [bar.default]
    position = "top"
    enabled = true
    auto_hide = false
    reserve_space = true
    layer = "top"
    thickness = 34
    background_opacity = 0.5
    border = "outline"
    border_width = 0.0
    shadow = false
    contact_shadow = false
    panel_overlap = 1
    radius = 12
    radius_top_left = 12
    radius_top_right = 12
    radius_bottom_left = 12
    radius_bottom_right = 12
    margin_ends = 180
    margin_edge = 10
    margin_opposite_edge = 0
    padding = 14
    widget_spacing = 12
    scale = 1.0
    font_weight = "regular"
    font_family = ""
    capsule = false
    capsule_fill = "surface_variant"
    capsule_thickness = 0.76
    capsule_radius = 8.0
    capsule_opacity = 1.0
    start = ["launcher", "wallpaper", "workspaces"]
    center = ["clock"]
    end = [
      "media",
      "tray",
      "notifications",
      "clipboard",
      "network",
      "bluetooth",
      "volume",
      "brightness",
      "battery",
      "control-center",
      "session"
    ]

    [dock]
    enabled = true
    position = "bottom"
    active_monitor_only = false
    monitors = []
    icon_size = 40
    main_axis_padding = 16
    cross_axis_padding = 8
    item_spacing = 6
    background_opacity = 0.50
    shadow = false
    radius = 16
    radius_top_left = 16
    radius_top_right = 16
    radius_bottom_left = 16
    radius_bottom_right = 16
    margin_ends = 0
    margin_edge = 8
    show_running = true
    auto_hide = false
    reserve_space = true
    active_scale = 1.0
    inactive_scale = 0.85
    magnification = true
    magnification_scale = 1.35
    active_opacity = 1.0
    inactive_opacity = 0.85
    show_instance_count = true
    show_dots = true
    launcher_position = "none"
    launcher_icon = "grid-dots"
    pinned = [
      "firefox",
      "org.gnome.Nautilus",
      "spotify",
      "steam",
      "net.lutris.Lutris",
      "moe.launcher.an-anime-game-launcher",
      "moe.launcher.the-honkers-railway-launcher",
      "vesktop",
      "affinity.exe",
      "localsend_app",
      "code",
      "com.obsproject.Studio",
      "org.kde.kdenlive",
      "heroic",
      "ONLYOFFICE",
      "birdfont"
    ]

    [osd]
    position = "top_center"
    position_vertical = "top_center"
    orientation = "horizontal"
    scale = 1.0
    background_opacity = 0.50
    offset_x = 20
    offset_y = 8

    [osd.kinds]
    volume = true
    volume_output = true
    volume_input = true
    brightness = true
    wifi = true
    bluetooth = true
    power_profile = true
    caffeine = true
    nightlight = true
    dnd = true
    lock_keys = true
    keyboard_layout = true
    privacy = true

    [lockscreen]
    enabled = false
    fingerprint = false
    allow_empty_password = false
    blurred_desktop = true
    blur_intensity = 0.5
    tint_intensity = 0.6
    wallpaper = "${./fondos/oscuro}/capriccio-arco-rovinato-e-una-villa-nello-sfondo.jpg"
    monitors = []

    [calendar]
    enabled = true
    refresh_minutes = 15

    [calendar.account.mi_google_cal]
    type = "google"
    name = "Google Calendar"

    [wallpaper]
    enabled = true
    fill_mode = "crop"
    fill_color = "#111111"
    transition = ["fade", "wipe", "disc", "stripes", "zoom", "honeycomb"]
    transition_duration = 1500
    edge_smoothness = 0.3
    transition_on_startup = false
    directory = "${./fondos}"
    directory_light = "${./fondos/claro}"
    directory_dark = "${./fondos/oscuro}"
    per_monitor_directories = false

    [wallpaper.default]
    path = "${./fondos/oscuro}/capriccio-arco-rovinato-e-una-villa-nello-sfondo.jpg"

    [wallpaper.automation]
    enabled = false
    interval_seconds = 1800
    order = "random"
    recursive = true

    [theme]
    mode = "auto"
    source = "community"
    builtin = "Noctalia"
    community_palette = "Everforest"

    [theme.templates]
    enable_builtin_templates = true
    builtin_ids = [
      "alacritty",
      "gtk3",
      "gtk4",
      "niri"
    ]
  '';
in {
  programs.noctalia.enable = true;

  environment.etc."noctalia/config.toml".source = noctaliaConfig;

  system.activationScripts.noctaliaConfig.text = ''
    install -d -m 0755 \
      -o ${config.users.users.koru.name} \
      -g ${config.users.users.koru.group} \
      ${config.users.users.koru.home}/.config/noctalia

    install -m 0644 \
      -o ${config.users.users.koru.name} \
      -g ${config.users.users.koru.group} \
      ${noctaliaConfig} \
      ${config.users.users.koru.home}/.config/noctalia/config.toml
  '';
}

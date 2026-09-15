{
  config,
  equipo,
  pkgs,
  ...
}: let
  usuario = config.users.users.${equipo.persona};

  noctaliaConfig = pkgs.writeText "noctalia-config.toml" ''
    [accessibility]
    ui_scale = 0.9
    high_contrast = false

    [shell]
    corner_radius_scale = 1.0
    font_family = "sans-serif"
    lang = "es"
    time_format = "{:%H:%M:%S}"
    date_format = "%A, %x"
    offline_mode = false
    telemetry_enabled = false
    setup_wizard_enabled = false
    polkit_agent = true
    password_style = "default"
    avatar_path = "${./.face/avatar.jpg}"
    settings_show_advanced = false
    show_location = true
    launch_apps_as_systemd_services = false
    screen_time_enabled = true
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

    [shell.greeter_sync]
    auto_sync = true

    [shell.panel]
    transparency_mode = "glass"
    borders = true
    shadow = false
    launcher_placement = "floating"
    clipboard_placement = "floating"
    control_center_placement = "attached"
    wallpaper_placement = "attached"
    session_placement = "attached"
    floating_offset = 8
    open_near_click_control_center = false
    open_near_click_launcher = false
    open_near_click_clipboard = false
    open_near_click_wallpaper = false
    open_near_click_session = false

    [shell.screen_corners]
    enabled = true
    size = 32

    [shell.mpris]
    blacklist = []

    [shell.launcher]
    sort_by_usage = false

    [shell.screenshot]
    save_to_file = true
    directory = "~/Imágenes/Capturas de pantalla"
    filename_pattern = "Captura de pantalla %Y-%m-%d %H-%M-%S"
    copy_to_clipboard = true
    freeze_screen = true
    confirm_region = true
    annotate = true

    [location]
    auto_locate = true

    [control_center]

    [[control_center.shortcuts]]
    type = "wifi"

    [[control_center.shortcuts]]
    type = "bluetooth"

    [[control_center.shortcuts]]
    type = "nightlight"

    [[control_center.shortcuts]]
    type = "notification"

    [[control_center.shortcuts]]
    type = "power_profile"

    [[control_center.shortcuts]]
    type = "dark_mode"

    [bar]
    order = ["default", "derecha"]

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
    radius = 24
    radius_top_left = 24
    radius_top_right = 24
    radius_bottom_left = 24
    radius_bottom_right = 24
    margin_ends = 15
    margin_edge = 10
    margin_opposite_edge = 0
    padding = 14
    widget_spacing = 12
    scale = 0.9
    font_weight = "regular"
    font_family = ""
    capsule = false
    capsule_fill = "surface_variant"
    capsule_thickness = 0.76
    capsule_radius = 8.0
    capsule_opacity = 1.0
    start = ["workspaces", "volume", "cat", "group:inicio"]
    center = ["group:centro"]
    end = [
      "tray",
      "notifications",
      "group:fin",
      "udiskie_manager",
      "network",
      "bluetooth"
    ]

    [[bar.default.capsule_group]]
    id = "inicio"
    members = ["bongo_cat", "media", "audio_visualizer"]

    [[bar.default.capsule_group]]
    id = "centro"
    members = ["fecha", "clock", "weather"]

    [[bar.default.capsule_group]]
    id = "fin"
    members = ["clipboard", "calculator", "pomodoro_timer", "notes"]

    [bar.derecha]
    position = "right"
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
    radius = 24
    radius_top_left = 24
    radius_top_right = 24
    radius_bottom_left = 24
    radius_bottom_right = 24
    margin_ends = 15
    margin_edge = 10
    margin_opposite_edge = 0
    padding = 14
    widget_spacing = 12
    scale = 0.9
    font_weight = "regular"
    font_family = ""
    capsule = false
    capsule_fill = "surface_variant"
    capsule_thickness = 0.76
    capsule_radius = 8.0
    capsule_opacity = 1.0
    start = ["lock_keys", "temperatura", "group:umbriel", "speedtest_meter"]
    center = ["phone_connect", "printers"]
    end = ["wallpaper", "group:red", "group:captura"]

    [[bar.derecha.capsule_group]]
    id = "umbriel"
    members = ["umbriel_displays", "umbriel_companion"]

    [[bar.derecha.capsule_group]]
    id = "red"
    members = ["red_rx", "red_tx"]

    [[bar.derecha.capsule_group]]
    id = "captura"
    members = ["privacy", "screenshot", "screen_recorder"]

    [widget.fecha]
    type = "clock"
    format = "{:%A, %d de %B}"

    [widget.weather]
    type = "weather"
    max_length = 250
    anchor = true

    [widget.media]
    type = "media"
    max_length = 400
    hide_when_no_media = true

    [widget.cat]
    type = "dotnetrob/cat:cat"
    show_cpu_percent = true
    interactive = false

    [widget.bongo_cat]
    type = "noctalia/bongocat:cat"

    [widget.calculator]
    type = "yuuto/calculator:bar"

    [widget.pomodoro_timer]
    type = "thepunkoff/pomodoro:widget"

    [widget.notes]
    type = "noctalia/notes:notes"

    [widget.udiskie_manager]
    type = "aristides/udiskie:status"

    [widget.tray]
    type = "tray"
    drawer = true

    [widget.network]
    type = "network"
    show_label = false

    [widget.bluetooth]
    type = "bluetooth"
    hide_when_adapter_off = true

    [widget.lock_keys]
    type = "lock_keys"
    hide_when_off = true
    display = "full"

    [widget.temperatura]
    type = "sysmon"
    stat = "cpu_temp"

    [widget.umbriel_displays]
    type = "prponkshe/umbriel-displays:bar"

    [widget.umbriel_companion]
    type = "noctalia/umbriel-companion:bar"

    [widget.speedtest_meter]
    type = "nilsonlinux/speedtest-meter:speedtest-widget"

    [widget.phone_connect]
    type = "icefish/phone-connect:bar"

    [widget.printers]
    type = "andrewdems/printers:printer"

    [widget.red_rx]
    type = "sysmon"
    stat = "net_rx"

    [widget.red_tx]
    type = "sysmon"
    stat = "net_tx"

    [widget.privacy]
    type = "privacy"
    hide_inactive = true

    [widget.screen_recorder]
    type = "noctalia/screen_recorder:recorder"

    [plugin_settings."noctalia/notes"]
    extension = "md"
    panel_placement = "floating"
    panel_position = "center_right"
    panel_layer = "top"

    [dock]
    enabled = true
    position = "bottom"
    active_monitor_only = false
    monitors = []
    icon_size = 40
    main_axis_padding = 16
    cross_axis_padding = 8
    item_spacing = 6
    background_opacity = 0.25
    shadow = false
    radius = 24
    radius_top_left = 24
    radius_top_right = 24
    radius_bottom_left = 24
    radius_bottom_right = 24
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
      "anime-game-launcher",
      "honkers-railway-launcher",
      "vesktop",
      "LocalSend",
      "code",
      "com.obsproject.Studio",
      "org.kde.kdenlive",
      "com.heroicgameslauncher.hgl",
      "onlyoffice-desktopeditors",
      "birdfont"
    ]

    [osd]
    position = "top_center"
    position_vertical = "top_center"
    orientation = "horizontal"
    scale = 1.0
    background_opacity = 0.25
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
    enabled = true
    blurred_desktop = true
    blur_intensity = 0.5
    tint_intensity = 0.6
    wallpaper = "${./fondos/oscuro}/capriccio-arco-rovinato-e-una-villa-nello-sfondo.jpg"
    monitors = []

    [calendar]
    enabled = true
    refresh_minutes = 15

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

    [plugins]
    enabled = [
      "noctalia/umbriel-companion",
      "prponkshe/umbriel-displays",
      "noctalia/screen_recorder",
      "noctalia/bongocat",
      "noctalia/timer",
      "noctalia/kaomoji",
      "noctalia/notes",
      "noctalia/world_clock",
      "aristides/udiskie",
      "andrewdems/printers",
      "nilsonlinux/speedtest-meter",
      "thepunkoff/pomodoro",
      "yuuto/calculator",
      "liamwh/emoji-picker",
      "notfinaldev/youtube-search",
      "icefish/phone-connect",
      "dotnetrob/cat"
    ]

    [theme]
    mode = "auto"
    source = "wallpaper"
    builtin = "Noctalia"
    wallpaper_scheme = "m3-tonal-spot"

    [theme.templates]
    enable_builtin_templates = true
    builtin_ids = [
      "alacritty",
      "gtk3",
      "gtk4",
      "qt",
      "umbriel"
    ]
    enable_community_templates = true
    community_ids = [
      "blender",
      "darktable",
      "discord",
      "gimp",
      "heroiclauncher",
      "inkscape",
      "libreoffice",
      "obs",
      "obsidian",
      "prismlauncher",
      "pywalfox",
      "steam",
      "vscode"
    ]
  '';
in {
  programs.noctalia.enable = true;

  environment.etc."noctalia/config.toml".source = noctaliaConfig;

  system.activationScripts.noctaliaConfig.text = ''
    install -d -m 0755 \
      -o ${usuario.name} \
      -g ${usuario.group} \
      ${usuario.home}/.config/noctalia

    # La interfaz de Noctalia guarda sus cambios en settings.toml, que tiene
    # prioridad sobre config.toml. Las plantillas y los ajustes de Notes que ya
    # declaramos no deben quedar duplicados como sobreescrituras de la GUI.
    settings_file=${usuario.home}/.local/state/noctalia/settings.toml
    if [ -f "$settings_file" ]; then
      tmp_file="$settings_file.korunix-tmp"
      skip_templates=false
      skip_notes=false
      while IFS= read -r line || [ -n "$line" ]; do
        case "$line" in
          "[theme.templates]")
            skip_templates=true
            skip_notes=false
            continue
            ;;
          "[plugin_settings.\"noctalia/notes\"]")
            skip_templates=false
            skip_notes=true
            continue
            ;;
          "["*)
            skip_templates=false
            skip_notes=false
            ;;
        esac
        if [ "$skip_templates" = false ] && [ "$skip_notes" = false ]; then
          printf '%s\n' "$line"
        fi
      done < "$settings_file" > "$tmp_file"
      install -m 0644 -o ${usuario.name} -g ${usuario.group} "$tmp_file" "$settings_file"
      rm -f "$tmp_file"
    fi

    install -m 0644 \
      -o ${usuario.name} \
      -g ${usuario.group} \
      ${noctaliaConfig} \
      ${usuario.home}/.config/noctalia/config.toml

    # Notes sigue el directorio XDG de Documentos en vez de asumir ~/Documents.
    # Se genera como una segunda capa TOML para conservar la localización real
    # de cada usuario (Documentos, Documents, Dokumente, etc.).
    documents_dir="$(HOME=${usuario.home} ${pkgs.xdg-user-dirs}/bin/xdg-user-dir DOCUMENTS)"
    if [ -n "$documents_dir" ]; then
      notes_dir="$documents_dir/Notes"
      notes_value="$(${pkgs.python3}/bin/python3 -c 'import json, sys; print(json.dumps(sys.argv[1], ensure_ascii=False))' "$notes_dir")"
      xdg_file=${usuario.home}/.config/noctalia/zz-korunix-xdg.toml
      xdg_tmp="$xdg_file.korunix-tmp"
      {
        printf '%s\n' '[plugin_settings."noctalia/notes"]'
        printf 'notes_dir = %s\n' "$notes_value"
      } > "$xdg_tmp"
      install -m 0644 -o ${usuario.name} -g ${usuario.group} "$xdg_tmp" "$xdg_file"
      rm -f "$xdg_tmp"
    fi
  '';
}

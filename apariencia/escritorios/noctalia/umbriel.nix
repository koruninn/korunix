{
  config,
  equipo,
  lib,
  pkgs,
  ...
}: let
  usuario = config.users.users.${equipo.persona};
  toml = pkgs.formats.toml {};

  umbrielConfig = toml.generate "umbriel-config.toml" ({
    include.files = ["noctalia.toml"];

    general = {
      autostart = ["noctalia"];
      mod_key = "Super";
      xwayland = true;
      show_cheatsheet = false;
      focus_on_activate = false;
    };

    environment = {
      QT_QPA_PLATFORM = "wayland";
      GDK_BACKEND = "wayland";
      QT_QPA_PLATFORMTHEME = "kde";
      QT_QPA_PLATFORMTHEME_QT6 = "kde";
    };

    workspaces = {
      back_and_forth = false;
      empty_above = false;
    };

    appearance = {
      prefer_no_csd = true;
      border_width = 2;
      outer_border_width = 0;
      corner_radius = 20;
      drag_opacity = 0.75;

      blur = {
        enabled = true;
        optimized = true;
        passes = 1;
        radius = 3;
        noise = 0.03;
        brightness = 1.0;
        contrast = 1.0;
        saturation = 1.0;
      };

      shadow.enabled = false;
    };

    input = {
      middle_click_paste = true;
      keyboard = {
        layout = "es";
        variant = "deadtilde";
        options = "";
        numlock_toggle = true;
      };
      touchpad = {
        tap = true;
        natural_scroll = true;
      };
      cursor = {
        theme = "Bibata-Modern-Ice";
        size = 24;
        follows_focus = true;
      };
      focus.follows_mouse = true;
    };

    layout = {
      mode = "scrolling";
      gap = 8;
      width_presets = [0.333 0.5 0.667];
      scrolling = {
        default_width_fraction = 0.5;
        center_underfull_strip = true;
        center_focused = false;
      };
    };

    keybinds = {
      "Mod+Space" = "spawn:noctalia msg panel-toggle launcher";
      "Mod+S" = "spawn:noctalia msg panel-toggle control-center";
      "Mod+Comma" = "spawn:noctalia msg settings-toggle";
      "Mod+L" = "spawn:noctalia msg session lock";
      "Alt+Tab" = "spawn:noctalia msg window-switcher";

      "Mod+T" = "spawn:alacritty";
      "Mod+E" = "spawn:nautilus";
      "Mod+B" = "spawn:zen";

      "Mod+Q" = "window-close";
      "Mod+O" = {
        action = "overview-toggle";
        repeat = false;
      };
      "Mod+Left" = "window-focus-left";
      "Mod+Down" = "window-focus-down";
      "Mod+Up" = "window-focus-up";
      "Mod+Right" = "window-focus-right";
      "Mod+Ctrl+Left" = "column-move-left";
      "Mod+Ctrl+Down" = "window-move-down";
      "Mod+Ctrl+Up" = "window-move-up";
      "Mod+Ctrl+Right" = "column-move-right";
      "Mod+V" = "window-toggle-floating";
      "Mod+Shift+V" = "window-focus-switch-floating";
      "Mod+F" = "window-toggle-maximize";
      "Mod+Shift+F" = "window-toggle-fullscreen";
      "Mod+R" = "window-cycle-width";
      "Mod+Shift+R" = "window-cycle-width-back";

      "Mod+Page_Up" = "workspace-previous";
      "Mod+Page_Down" = "workspace-next";
      "Mod+Ctrl+Page_Up" = "window-move-to-workspace-previous";
      "Mod+Ctrl+Page_Down" = "window-move-to-workspace-next";

      "Mod+1" = "workspace-switch:1";
      "Mod+2" = "workspace-switch:2";
      "Mod+3" = "workspace-switch:3";
      "Mod+4" = "workspace-switch:4";
      "Mod+5" = "workspace-switch:5";
      "Mod+6" = "workspace-switch:6";
      "Mod+7" = "workspace-switch:7";
      "Mod+8" = "workspace-switch:8";
      "Mod+9" = "workspace-switch:9";
      "Mod+Shift+1" = "window-move-to-workspace:1";
      "Mod+Shift+2" = "window-move-to-workspace:2";
      "Mod+Shift+3" = "window-move-to-workspace:3";
      "Mod+Shift+4" = "window-move-to-workspace:4";
      "Mod+Shift+5" = "window-move-to-workspace:5";
      "Mod+Shift+6" = "window-move-to-workspace:6";
      "Mod+Shift+7" = "window-move-to-workspace:7";
      "Mod+Shift+8" = "window-move-to-workspace:8";
      "Mod+Shift+9" = "window-move-to-workspace:9";

      Print = "spawn:noctalia msg screenshot-region";
      "Ctrl+Print" = "spawn:noctalia msg screenshot-fullscreen";

      XF86AudioRaiseVolume = "spawn:noctalia msg volume-up";
      XF86AudioLowerVolume = "spawn:noctalia msg volume-down";
      XF86AudioMute = "spawn:noctalia msg volume-mute";
      XF86AudioMicMute = "spawn:wpctl set-mute @DEFAULT_AUDIO_SOURCE@ toggle";
      XF86AudioPlay = "spawn:noctalia msg media toggle";
      XF86AudioStop = "spawn:noctalia msg media stop";
      XF86AudioPrev = "spawn:noctalia msg media previous";
      XF86AudioNext = "spawn:noctalia msg media next";
      XF86MonBrightnessUp = "spawn:noctalia msg brightness-up";
      XF86MonBrightnessDown = "spawn:noctalia msg brightness-down";
    };

    window_rule = [
      {
        blur = true;
        blur_optimized = true;
        opacity = 0.90;
      }
      {
        match.is_focused = false;
        opacity = 0.85;
      }
      {
        match.app_id = "^dev.noctalia.Noctalia$";
        default_floating = true;
        default_size = [1020 900];
      }
      {
        match.app_id = "^dev.noctalia.UmbrielSharePicker$";
        default_floating = true;
        default_size = [800 600];
      }
      {
        match.title = "^(Picture-in-Picture|Picture in picture)$";
        default_floating = true;
        default_maximize = false;
      }
    ];

    layer_rule = [
      {
        match.namespace = ''^noctalia-(bar-[^"]+|notification|dock|panel|attached-panel|osd|desktop-widget-[^"]*)$'';
        blur = true;
        blur_ignore_alpha = 0.5;
        blur_popups = true;
        blur_optimized = false;
      }
    ];

    animation = {
      enabled = true;
      duration_ms = 250;
      curve = "easeout";
    };
  } // lib.optionalAttrs (equipo ? pantalla) {
    output = {
      "${equipo.pantalla.nombre}" = {
        enabled = true;
        mode = equipo.pantalla.modo;
        scale = equipo.pantalla.escala;
      };
    };
  });
in {
  programs.umbriel.enable = true;

  # xwayland-satellite habilita aplicaciones X11; Bibata queda disponible para
  # el cursor de Umbriel sin imponer el tema al resto de escritorios.
  environment.systemPackages = [
    pkgs.xwayland-satellite
    pkgs.bibata-cursors
  ];

  # Umbriel consulta primero la configuración XDG de la persona. La base sigue
  # siendo declarativa; noctalia.toml queda separado para que Noctalia actualice
  # únicamente los colores sin tocar los atajos ni la gestión de ventanas.
  system.activationScripts.umbrielConfig.text = ''
    config_dir=${usuario.home}/.config/umbriel

    install -d -m 0755 \
      -o ${usuario.name} \
      -g ${usuario.group} \
      "$config_dir"

    install -m 0644 \
      -o ${usuario.name} \
      -g ${usuario.group} \
      ${umbrielConfig} \
      "$config_dir/config.toml"

    if [ ! -e "$config_dir/noctalia.toml" ]; then
      printf '%s\n' '# Noctalia generará aquí la paleta de Umbriel.' > "$config_dir/noctalia.toml"
      chown ${usuario.name}:${usuario.group} "$config_dir/noctalia.toml"
      chmod 0644 "$config_dir/noctalia.toml"
    fi
  '';
}

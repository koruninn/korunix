{
  config,
  equipo,
  inputs,
  lib,
  pkgs,
  ...
}: let
  usuario = config.users.users.koru;
  toml = pkgs.formats.toml {};

  umbrielConfig = toml.generate "umbriel-config.toml" {
    include.files = ["outputs.toml"];
    include.optional.files = ["noctalia.toml"];

    general = {
      mod_key = "Super";
      autostart = ["korunix-noctalia"];
      xwayland = true;
      show_cheatsheet = false;
      focus_on_activate = false;
    };

    workspaces = {
      back_and_forth = false;
      empty_above = false;
    };

    appearance = {
      prefer_no_csd = true;
      border_width = 3;
      outer_border_width = 0;
      corner_radius = 20;
      drag_opacity = 0.75;

      # Ajustar el desenfoque para que se parezca al de Niri.
      blur = {
        enabled = true;
        optimized = false;
        passes = 1;
        radius = 4;
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
      touch = {
        enabled = true;
        map_to_output = "eDP-1";
      };
      touchpad = {
        tap = true;
        natural_scroll = true;
      };
      cursor = {
        theme = "Bibata-Modern-Classic";
        size = 24;
        follows_focus = true;
      };
      focus.follows_mouse = true;
    };

    layout = {
      mode = "scrolling";
      gap = 12;
      extent_presets = [0.333 0.5 0.667];
      scrolling = {
        default_extent_fraction = 0.5;
        center_underfull_strip = false;
        center_focused = "never";
      };
    };

    keybinds = {
      # Noctalia y las aplicaciones principales.
      "Mod+Space" = "spawn:noctalia msg panel-toggle launcher";
      "Mod+S" = "spawn:noctalia msg panel-toggle control-center";
      "Mod+Comma" = "spawn:noctalia msg settings-toggle";
      "Mod+Shift+Escape" = "spawn:noctalia msg session lock";
      "Alt+Tab" = "spawn:noctalia msg window-switcher hold";
      "Mod+T" = "spawn:ptyxis";
      "Ctrl+Alt+T" = "spawn:ptyxis";
      "Mod+Return" = "spawn:ptyxis";
      "Mod+D" = "spawn:noctalia msg panel-toggle launcher";
      "Mod+E" = "spawn:luciole";
      "Mod+B" = "spawn:firefox";

      # Ventanas y vista general.
      "Mod+Q" = "window-close";
      "Mod+O" = {
        action = "overview-toggle";
        repeat = false;
      };
      "Mod+Left" = "window-focus-left";
      "Mod+Down" = "window-focus-down";
      "Mod+Up" = "window-focus-up";
      "Mod+Right" = "window-focus-right";
      "Mod+H" = "window-focus-left";
      "Mod+J" = "window-focus-down";
      "Mod+K" = "window-focus-up";
      "Mod+L" = "window-focus-right";
      "Mod+Ctrl+Left" = "column-move-left";
      "Mod+Ctrl+Down" = "window-move-down";
      "Mod+Ctrl+Up" = "window-move-up";
      "Mod+Ctrl+Right" = "column-move-right";
      "Mod+Ctrl+H" = "column-move-left";
      "Mod+Ctrl+J" = "window-move-down";
      "Mod+Ctrl+K" = "window-move-up";
      "Mod+Ctrl+L" = "column-move-right";
      "Mod+Home" = "column-focus-first";
      "Mod+End" = "column-focus-last";
      "Mod+Ctrl+Home" = "column-move-to-first";
      "Mod+Ctrl+End" = "column-move-to-last";

      # Monitores.
      "Mod+Shift+Left" = "output-focus-left";
      "Mod+Shift+Down" = "output-focus-down";
      "Mod+Shift+Up" = "output-focus-up";
      "Mod+Shift+Right" = "output-focus-right";
      "Mod+Shift+H" = "output-focus-left";
      "Mod+Shift+J" = "output-focus-down";
      "Mod+Shift+K" = "output-focus-up";
      "Mod+Shift+L" = "output-focus-right";
      "Mod+Shift+Ctrl+Left" = "column-move-to-output-left";
      "Mod+Shift+Ctrl+Down" = "window-move-down";
      "Mod+Shift+Ctrl+Up" = "window-move-up";
      "Mod+Shift+Ctrl+Right" = "column-move-to-output-right";
      "Mod+Shift+Ctrl+H" = "column-move-to-output-left";
      "Mod+Shift+Ctrl+J" = "window-move-down";
      "Mod+Shift+Ctrl+K" = "window-move-up";
      "Mod+Shift+Ctrl+L" = "column-move-to-output-right";

      # Estados, tamaños y cómo se ordenan las ventanas.
      "Mod+V" = "window-toggle-floating";
      "Mod+Shift+V" = "window-focus-switch-floating";
      "Mod+F" = "window-toggle-maximize";
      "Mod+Shift+F" = "window-toggle-fullscreen";
      "Mod+M" = "window-toggle-maximize-to-edges";
      "Mod+C" = "column-center";
      "Mod+R" = "window-cycle-primary-extent";
      "Mod+Shift+R" = "window-cycle-primary-extent-back";
      "Mod+Ctrl+Shift+R" = "window-cycle-secondary-extent";
      "Mod+Minus" = "window-modify-width-right:-0.1";
      "Mod+Equal" = "window-modify-width-right:0.1";
      "Mod+Shift+Minus" = "window-modify-height-down:-0.1";
      "Mod+Shift+Equal" = "window-modify-height-down:0.1";
      "Mod+BracketLeft" = "window-consume-or-expel-left";
      "Mod+BracketRight" = "window-consume-or-expel-right";

      # Escritorios: moverse, llevar ventanas y volver al anterior.
      "Mod+Page_Up" = "workspace-previous";
      "Mod+Page_Down" = "workspace-next";
      "Mod+U" = "workspace-next";
      "Mod+I" = "workspace-previous";
      "Mod+Ctrl+Page_Up" = "column-move-to-workspace-previous";
      "Mod+Ctrl+Page_Down" = "column-move-to-workspace-next";
      "Mod+Ctrl+U" = "column-move-to-workspace-next";
      "Mod+Ctrl+I" = "column-move-to-workspace-previous";
      "Mod+Shift+Page_Up" = "workspace-move-up";
      "Mod+Shift+Page_Down" = "workspace-move-down";
      "Mod+Shift+U" = "workspace-move-down";
      "Mod+Shift+I" = "workspace-move-up";
      "Mod+Tab" = "workspace-focus-last";

      # Rueda del ratón.
      "Mod+WheelDown" = "workspace-next";
      "Mod+WheelUp" = "workspace-previous";
      "Mod+Ctrl+WheelDown" = "column-move-to-workspace-next";
      "Mod+Ctrl+WheelUp" = "column-move-to-workspace-previous";
      "Mod+WheelRight" = "window-focus-right";
      "Mod+WheelLeft" = "window-focus-left";
      "Mod+Ctrl+WheelRight" = "column-move-right";
      "Mod+Ctrl+WheelLeft" = "column-move-left";
      "Mod+Shift+WheelDown" = "window-focus-right";
      "Mod+Shift+WheelUp" = "window-focus-left";
      "Mod+Ctrl+Shift+WheelDown" = "column-move-right";
      "Mod+Ctrl+Shift+WheelUp" = "column-move-left";

      # Usar las mismas combinaciones en ambos teclados.
      "Mod+1" = "workspace-switch:1";
      "Mod+2" = "workspace-switch:2";
      "Mod+3" = "workspace-switch:3";
      "Mod+4" = "workspace-switch:4";
      "Mod+5" = "workspace-switch:5";
      "Mod+6" = "workspace-switch:6";
      "Mod+7" = "workspace-switch:7";
      "Mod+8" = "workspace-switch:8";
      "Mod+9" = "workspace-switch:9";
      "Mod+KP_1" = "workspace-switch:1";
      "Mod+KP_2" = "workspace-switch:2";
      "Mod+KP_3" = "workspace-switch:3";
      "Mod+KP_4" = "workspace-switch:4";
      "Mod+KP_5" = "workspace-switch:5";
      "Mod+KP_6" = "workspace-switch:6";
      "Mod+KP_7" = "workspace-switch:7";
      "Mod+KP_8" = "workspace-switch:8";
      "Mod+KP_9" = "workspace-switch:9";
      "Mod+Shift+1" = "window-move-to-workspace:1";
      "Mod+Shift+2" = "window-move-to-workspace:2";
      "Mod+Shift+3" = "window-move-to-workspace:3";
      "Mod+Shift+4" = "window-move-to-workspace:4";
      "Mod+Shift+5" = "window-move-to-workspace:5";
      "Mod+Shift+6" = "window-move-to-workspace:6";
      "Mod+Shift+7" = "window-move-to-workspace:7";
      "Mod+Shift+8" = "window-move-to-workspace:8";
      "Mod+Shift+9" = "window-move-to-workspace:9";

      # Capturas y controles de multimedia.
      Print = "spawn:noctalia msg screenshot-region";
      "Ctrl+Print" = "spawn:noctalia msg screenshot-fullscreen";
      XF86AudioRaiseVolume = { action = "spawn:noctalia msg volume-up"; allow_when_locked = true; };
      XF86AudioLowerVolume = { action = "spawn:noctalia msg volume-down"; allow_when_locked = true; };
      XF86AudioMute = { action = "spawn:noctalia msg volume-mute"; allow_when_locked = true; };
      XF86AudioMicMute = { action = "spawn:wpctl set-mute @DEFAULT_AUDIO_SOURCE@ toggle"; allow_when_locked = true; };
      XF86AudioPlay = { action = "spawn:playerctl play-pause"; allow_when_locked = true; };
      XF86AudioStop = { action = "spawn:noctalia msg media stop"; allow_when_locked = true; };
      XF86AudioPrev = { action = "spawn:playerctl previous"; allow_when_locked = true; };
      XF86AudioNext = { action = "spawn:playerctl next"; allow_when_locked = true; };
      XF86MonBrightnessUp = { action = "spawn:noctalia msg brightness-up"; allow_when_locked = true; };
      XF86MonBrightnessDown = { action = "spawn:noctalia msg brightness-down"; allow_when_locked = true; };

      # Sesión y pantalla.
      "Mod+Shift+E" = "session-quit";
      "Ctrl+Alt+Delete" = "session-quit";
      "Mod+Shift+P" = "dpms-off";
    };

    window_rule = [
      {
        blur = true;
        blur_optimized = false;
        blur_ignore_alpha = 0.0;
      }
      {
        match.is_focused = false;
        opacity = 0.60;
      }
      {
        match.is_focused = true;
        opacity = 0.85;
      }
      {
        match.app_id = "^dev.noctalia.Noctalia$";
        default_floating = true;
        default_floating_size_px = {
          width = 1020;
          height = 900;
        };
      }
      {
        match.app_id = "^dev.noctalia.UmbrielSharePicker$";
        default_floating = true;
        default_floating_size_px = {
          width = 800;
          height = 600;
        };
      }
      {
        match.title = "^(Picture-in-Picture|Picture in picture)$";
        default_floating = true;
        default_maximize = false;
      }
    ];

    layer_rule = [
      {
        match.namespace = ''^noctalia-(bar-[^\"]+|notification|dock|panel|attached-panel|osd|desktop-widget-[^\"]*)$'';
        blur = true;
        # Mantener el desenfoque dentro del panel y los bordes transparentes.
        blur_ignore_alpha = 0.10;
        blur_popups = true;
        blur_optimized = false;
      }
    ];

    animation = {
      enabled = true;
      duration_ms = 250;
      curve = "easeout";

      windows_in = {
        enabled = true;
        duration_ms = 180;
        curve = "easeout";
        style = "popin";
        scale = 0.90;
      };

      windows_out = {
        enabled = true;
        duration_ms = 220;
        curve = "easeout";
        style = "fade";
      };

      windows_move = {
        enabled = true;
        duration_ms = 220;
        curve = "snappy";
      };

      workspaces = {
        enabled = true;
        duration_ms = 220;
        curve = "snappy";
      };

      overview = {
        enabled = true;
        duration_ms = 260;
        curve = "easeout";
        workspace_curve = "spring:1,1000";
      };

      layers = {
        enabled = true;
        duration_ms = 180;
        curve = "easeout";
      };
    };
  };

  umbrielOutputs = toml.generate "umbriel-outputs.toml" (lib.optionalAttrs (equipo ? pantalla) {
    output = {
      "${equipo.pantalla.nombre}" = {
        enabled = true;
        mode = equipo.pantalla.modo;
        scale = equipo.pantalla.escala;
      };
    };
  });
in lib.mkIf (equipo.habilitarUmbriel or false) {
  programs.umbriel = {
    enable = true;
    package = inputs.umbriel.packages.${pkgs.stdenv.hostPlatform.system}.default.overrideAttrs (old: {
      # La pantalla tocada queda activa sin mover el puntero.
      patches = (old.patches or []) ++ [./umbriel-touch-output.patch];
    });
  };

  # Usar el XWayland que trae Umbriel.
  environment.systemPackages = [pkgs.bibata-cursors];

  # Respetar la paleta y los monitores que configura Noctalia.
  system.activationScripts.umbrielConfig.text = ''
    config_dir=${usuario.home}/.config/umbriel

    install -d -m 0755 \
      -o ${usuario.name} \
      -g ${usuario.group} \
      "$config_dir"

    # Guardar la configuración anterior sin tocar outputs.toml.
    if [ -f "$config_dir/config.toml" ] && [ ! -e "$config_dir/config.toml.pre-korunix-umbriel" ]; then
      cp -p "$config_dir/config.toml" "$config_dir/config.toml.pre-korunix-umbriel"
    fi
    install -m 0644 \
      -o ${usuario.name} \
      -g ${usuario.group} \
      ${umbrielConfig} \
      "$config_dir/config.toml"

    if [ ! -e "$config_dir/outputs.toml" ]; then
      install -m 0644 \
        -o ${usuario.name} \
        -g ${usuario.group} \
        ${umbrielOutputs} \
        "$config_dir/outputs.toml"
    fi

    if [ ! -e "$config_dir/noctalia.toml" ]; then
      printf '%s\n' '# Noctalia generará aquí la paleta de Umbriel.' > "$config_dir/noctalia.toml"
      chown ${usuario.name}:${usuario.group} "$config_dir/noctalia.toml"
      chmod 0644 "$config_dir/noctalia.toml"
    fi
  '';
}

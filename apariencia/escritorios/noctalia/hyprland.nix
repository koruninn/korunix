{
  config,
  equipo,
  lib,
  pkgs,
  ...
}: let
  usuario = config.users.users.${equipo.persona};

  hyprlandConfig = pkgs.writeText "korunix-hyprland.lua" ''
    -- Korunix · Hyprland
    -- Noctalia controla la experiencia; Hyprland gestiona las ventanas.

    local mainMod = "SUPER"
    local ipc = "noctalia msg "

    ${lib.optionalString (equipo ? pantalla) ''
      hl.monitor({
        output = "${equipo.pantalla.nombre}",
        mode = "${equipo.pantalla.modo}",
        position = "0x0",
        scale = ${toString equipo.pantalla.escala},
      })
    ''}
    ${lib.optionalString (!(equipo ? pantalla)) ''
      hl.monitor({
        output = "",
        mode = "preferred",
        position = "auto",
        scale = "auto",
      })
    ''}

    -- La sesión queda aislada de Plasma igual que Niri y Umbriel.
    hl.env("XDG_CURRENT_DESKTOP", "Hyprland")
    hl.env("XDG_SESSION_DESKTOP", "Hyprland")
    hl.env("XDG_SESSION_TYPE", "wayland")
    hl.env("QT_QPA_PLATFORM", "wayland")
    hl.env("GDK_BACKEND", "wayland")
    hl.env("QT_QPA_PLATFORMTHEME", "kde")
    hl.env("QT_QPA_PLATFORMTHEME_QT6", "kde")
    hl.env("XCURSOR_THEME", "Bibata-Modern-Ice")
    hl.env("XCURSOR_SIZE", "24")

    hl.config({
      general = {
        gaps_in = 8,
        gaps_out = 8,
        border_size = 2,
        resize_on_border = true,
        allow_tearing = false,
        layout = "scrolling",
        col = {
          active_border = "rgb(9da9a0)",
          inactive_border = "rgb(4f5b58)",
        },
      },

      decoration = {
        rounding = 20,
        rounding_power = 2,
        active_opacity = 0.90,
        inactive_opacity = 0.85,
        shadow = {
          enabled = false,
        },
        blur = {
          enabled = true,
          size = 4,
          passes = 1,
          vibrancy = 0.0,
        },
      },

      animations = {
        enabled = true,
      },

      scrolling = {
        fullscreen_on_one_column = false,
        column_width = 0.5,
        focus_fit_method = 1,
        follow_focus = true,
        explicit_column_widths = "0.333, 0.5, 0.667, 1.0",
        wrap_focus = false,
        wrap_swapcol = false,
        direction = "right",
      },

      input = {
        kb_layout = "es",
        kb_variant = "deadtilde",
        kb_model = "",
        kb_options = "",
        kb_rules = "",
        numlock_by_default = true,
        follow_mouse = 1,
        sensitivity = 0,
        touchpad = {
          natural_scroll = true,
          tap_to_click = true,
        },
      },

      binds = {
        window_direction_monitor_fallback = true,
      },

      misc = {
        force_default_wallpaper = 0,
        disable_hyprland_logo = true,
        middle_click_paste = true,
      },
    })

    -- Mantener los escritorios visibles en Noctalia incluso cuando estén vacíos.
    for i = 1, 9 do
      local rule = {
        workspace = tostring(i),
        persistent = true,
      }
      ${lib.optionalString (equipo ? pantalla) ''rule.monitor = "${equipo.pantalla.nombre}"''}
      hl.workspace_rule(rule)
    end

    -- Noctalia: sin animación duplicada de Hyprland y con blur real detrás de
    -- barras, paneles, dock, notificaciones y OSD.
    hl.layer_rule({
      name = "noctalia",
      match = {
        namespace = "^noctalia-(bar-.+|notification|dock|panel|attached-panel|osd|window-switcher|desktop-widget-.+)$",
      },
      no_anim = true,
      ignore_alpha = 0.10,
      blur = true,
      blur_popups = true,
    })

    -- Picture-in-Picture sigue flotando; Noctalia Settings usa su tamaño natural.
    hl.window_rule({
      name = "picture-in-picture",
      match = { title = "^(Picture-in-Picture|Picture in picture)$" },
      float = true,
    })

    -- Evita que ventanas XWayland vacías se apropien del foco durante arrastres.
    hl.window_rule({
      name = "xwayland-drag-fix",
      match = {
        class = "^$",
        title = "^$",
        xwayland = true,
        float = true,
        fullscreen = false,
        pin = false,
      },
      no_focus = true,
    })

    -- Inicio de sesión.
    hl.on("hyprland.start", function()
      hl.exec_cmd("noctalia")
      hl.exec_cmd("hyprctl setcursor Bibata-Modern-Ice 24")
      hl.exec_cmd([[dconf write /org/gnome/desktop/interface/icon-theme "'Hatter-Slate'"]])
    end)

    -- Noctalia y aplicaciones principales.
    hl.bind(mainMod .. " + Space", hl.dsp.exec_cmd(ipc .. "panel-toggle launcher"))
    hl.bind(mainMod .. " + S", hl.dsp.exec_cmd(ipc .. "panel-toggle control-center"))
    hl.bind(mainMod .. " + comma", hl.dsp.exec_cmd(ipc .. "settings-toggle"))
    hl.bind(mainMod .. " + L", hl.dsp.exec_cmd(ipc .. "session lock"))
    hl.bind("ALT + Tab", hl.dsp.exec_cmd(ipc .. "window-switcher"))
    hl.bind(mainMod .. " + O", hl.dsp.exec_cmd(ipc .. "window-switcher"))
    hl.bind(mainMod .. " + T", hl.dsp.exec_cmd("alacritty"))
    hl.bind(mainMod .. " + E", hl.dsp.exec_cmd("nautilus"))
    hl.bind(mainMod .. " + B", hl.dsp.exec_cmd("zen"))

    -- Foco y movimiento. En horizontal usamos mensajes del layout scrolling para
    -- conservar el comportamiento por columnas de Niri/Umbriel.
    hl.bind(mainMod .. " + left", hl.dsp.focus({ direction = "l" }))
    hl.bind(mainMod .. " + right", hl.dsp.focus({ direction = "r" }))
    hl.bind(mainMod .. " + up", hl.dsp.focus({ direction = "u" }))
    hl.bind(mainMod .. " + down", hl.dsp.focus({ direction = "d" }))
    hl.bind(mainMod .. " + CTRL + left", hl.dsp.layout("swapcol l"))
    hl.bind(mainMod .. " + CTRL + right", hl.dsp.layout("swapcol r"))
    hl.bind(mainMod .. " + CTRL + H", hl.dsp.layout("swapcol l"))
    hl.bind(mainMod .. " + CTRL + L", hl.dsp.layout("swapcol r"))
    hl.bind(mainMod .. " + CTRL + up", hl.dsp.window.move({ direction = "u" }))
    hl.bind(mainMod .. " + CTRL + down", hl.dsp.window.move({ direction = "d" }))
    hl.bind(mainMod .. " + CTRL + K", hl.dsp.window.move({ direction = "u" }))
    hl.bind(mainMod .. " + CTRL + J", hl.dsp.window.move({ direction = "d" }))

    -- Monitores.
    hl.bind(mainMod .. " + SHIFT + left", hl.dsp.focus({ monitor = "l" }))
    hl.bind(mainMod .. " + SHIFT + right", hl.dsp.focus({ monitor = "r" }))
    hl.bind(mainMod .. " + SHIFT + up", hl.dsp.focus({ monitor = "u" }))
    hl.bind(mainMod .. " + SHIFT + down", hl.dsp.focus({ monitor = "d" }))
    hl.bind(mainMod .. " + SHIFT + H", hl.dsp.focus({ monitor = "l" }))
    hl.bind(mainMod .. " + SHIFT + L", hl.dsp.focus({ monitor = "r" }))
    hl.bind(mainMod .. " + SHIFT + K", hl.dsp.focus({ monitor = "u" }))
    hl.bind(mainMod .. " + SHIFT + J", hl.dsp.focus({ monitor = "d" }))
    hl.bind(mainMod .. " + SHIFT + CTRL + left", hl.dsp.window.move({ monitor = "l", follow = true }))
    hl.bind(mainMod .. " + SHIFT + CTRL + right", hl.dsp.window.move({ monitor = "r", follow = true }))
    hl.bind(mainMod .. " + SHIFT + CTRL + up", hl.dsp.window.move({ monitor = "u", follow = true }))
    hl.bind(mainMod .. " + SHIFT + CTRL + down", hl.dsp.window.move({ monitor = "d", follow = true }))

    -- Estados y organización de ventanas.
    hl.bind(mainMod .. " + Q", hl.dsp.window.close())
    hl.bind(mainMod .. " + V", hl.dsp.window.float({ action = "toggle" }))
    hl.bind(mainMod .. " + F", hl.dsp.window.fullscreen({ mode = "maximized", action = "toggle" }))
    hl.bind(mainMod .. " + SHIFT + F", hl.dsp.window.fullscreen({ mode = "fullscreen", action = "toggle" }))
    hl.bind(mainMod .. " + M", hl.dsp.window.fullscreen({ mode = "maximized", action = "toggle" }))
    hl.bind(mainMod .. " + R", hl.dsp.layout("colresize +conf"))
    hl.bind(mainMod .. " + SHIFT + R", hl.dsp.layout("colresize -conf"))
    hl.bind(mainMod .. " + minus", hl.dsp.layout("colresize -0.1"))
    hl.bind(mainMod .. " + equal", hl.dsp.layout("colresize +0.1"))
    hl.bind(mainMod .. " + bracketleft", hl.dsp.layout("consume_or_expel prev"))
    hl.bind(mainMod .. " + bracketright", hl.dsp.layout("consume_or_expel next"))
    hl.bind(mainMod .. " + period", hl.dsp.layout("promote"))
    hl.bind(mainMod .. " + CTRL + F", hl.dsp.layout("colresize 1.0"))
    hl.bind(mainMod .. " + C", hl.dsp.layout("fit active"))
    hl.bind(mainMod .. " + CTRL + C", hl.dsp.layout("fit visible"))
    hl.bind(mainMod .. " + W", hl.dsp.group.toggle())

    -- Escritorios: los persistentes permiten recorrer también los vacíos.
    hl.bind(mainMod .. " + Page_Down", hl.dsp.focus({ workspace = "r+1" }))
    hl.bind(mainMod .. " + Page_Up", hl.dsp.focus({ workspace = "r-1" }))
    hl.bind(mainMod .. " + U", hl.dsp.focus({ workspace = "r+1" }))
    hl.bind(mainMod .. " + I", hl.dsp.focus({ workspace = "r-1" }))
    hl.bind(mainMod .. " + CTRL + Page_Down", hl.dsp.window.move({ workspace = "r+1", follow = true }))
    hl.bind(mainMod .. " + CTRL + Page_Up", hl.dsp.window.move({ workspace = "r-1", follow = true }))
    hl.bind(mainMod .. " + CTRL + U", hl.dsp.window.move({ workspace = "r+1", follow = true }))
    hl.bind(mainMod .. " + CTRL + I", hl.dsp.window.move({ workspace = "r-1", follow = true }))
    hl.bind(mainMod .. " + Tab", hl.dsp.focus({ workspace = "previous" }))

    for i = 1, 9 do
      hl.bind(mainMod .. " + " .. i, hl.dsp.focus({ workspace = i }))
      hl.bind(mainMod .. " + SHIFT + " .. i, hl.dsp.window.move({ workspace = i }))
    end

    -- Rueda del ratón.
    hl.bind(mainMod .. " + mouse_down", hl.dsp.focus({ workspace = "r+1" }))
    hl.bind(mainMod .. " + mouse_up", hl.dsp.focus({ workspace = "r-1" }))
    hl.bind(mainMod .. " + CTRL + mouse_down", hl.dsp.window.move({ workspace = "r+1", follow = true }))
    hl.bind(mainMod .. " + CTRL + mouse_up", hl.dsp.window.move({ workspace = "r-1", follow = true }))
    hl.bind(mainMod .. " + SHIFT + mouse_down", hl.dsp.focus({ direction = "r" }))
    hl.bind(mainMod .. " + SHIFT + mouse_up", hl.dsp.focus({ direction = "l" }))

    -- Arrastrar y redimensionar ventanas flotantes con Super + ratón.
    hl.bind(mainMod .. " + mouse:272", hl.dsp.window.drag(), { mouse = true })
    hl.bind(mainMod .. " + mouse:273", hl.dsp.window.resize(), { mouse = true })

    -- Capturas y multimedia se delegan a Noctalia para mantener la misma UX.
    hl.bind("Print", hl.dsp.exec_cmd(ipc .. "screenshot-region"))
    hl.bind("CTRL + Print", hl.dsp.exec_cmd(ipc .. "screenshot-fullscreen"))
    hl.bind("XF86AudioRaiseVolume", hl.dsp.exec_cmd(ipc .. "volume-up"), { locked = true, repeating = true })
    hl.bind("XF86AudioLowerVolume", hl.dsp.exec_cmd(ipc .. "volume-down"), { locked = true, repeating = true })
    hl.bind("XF86AudioMute", hl.dsp.exec_cmd(ipc .. "volume-mute"), { locked = true })
    hl.bind("XF86AudioMicMute", hl.dsp.exec_cmd("wpctl set-mute @DEFAULT_AUDIO_SOURCE@ toggle"), { locked = true })
    hl.bind("XF86AudioPlay", hl.dsp.exec_cmd(ipc .. "media toggle"), { locked = true })
    hl.bind("XF86AudioStop", hl.dsp.exec_cmd(ipc .. "media stop"), { locked = true })
    hl.bind("XF86AudioPrev", hl.dsp.exec_cmd(ipc .. "media previous"), { locked = true })
    hl.bind("XF86AudioNext", hl.dsp.exec_cmd(ipc .. "media next"), { locked = true })
    hl.bind("XF86MonBrightnessUp", hl.dsp.exec_cmd(ipc .. "brightness-up"), { locked = true, repeating = true })
    hl.bind("XF86MonBrightnessDown", hl.dsp.exec_cmd(ipc .. "brightness-down"), { locked = true, repeating = true })

    -- Sesión y pantalla.
    hl.bind(mainMod .. " + SHIFT + E", hl.dsp.exit())
    hl.bind("CTRL + ALT + Delete", hl.dsp.exit())
    hl.bind(mainMod .. " + SHIFT + P", hl.dsp.dpms({ action = "off" }))

    -- La plantilla oficial de Noctalia sobrescribe únicamente la paleta de
    -- bordes y grupos. Un módulo neutro se instala antes del primer inicio.
    require("noctalia").apply_theme()
  '';

  noctaliaFallback = pkgs.writeText "noctalia.lua" ''
    return {
      colors = {},
      apply_theme = function() end,
    }
  '';
in {
  programs.hyprland = {
    enable = true;
    withUWSM = false;
    xwayland.enable = true;
  };

  environment.systemPackages = [pkgs.bibata-cursors];

  system.activationScripts.hyprlandConfig.text = ''
    config_dir=${usuario.home}/.config/hypr

    install -d -m 0755 \
      -o ${usuario.name} \
      -g ${usuario.group} \
      "$config_dir"

    install -m 0644 \
      -o ${usuario.name} \
      -g ${usuario.group} \
      ${hyprlandConfig} \
      "$config_dir/hyprland.lua"

    if [ ! -e "$config_dir/noctalia.lua" ]; then
      install -m 0644 \
        -o ${usuario.name} \
        -g ${usuario.group} \
        ${noctaliaFallback} \
        "$config_dir/noctalia.lua"
    fi
  '';
}

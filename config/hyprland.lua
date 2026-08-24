-- Configuración de Hyprland administrada por Korunix.
--
-- Hyprland 0.55 y posteriores utilizan Lua como formato principal.
-- Las decisiones específicas del equipo, como layouts de teclado, son
-- sustituidas por Nix al construir para no duplicar la fuente de verdad.
--
-- Noctalia NO se inicia aquí: Korunix ya lo administra mediante systemd.

@KORUNIX_MONITOR_RULE@
-- Cualquier monitor no declarado específicamente utiliza su modo preferido.
hl.monitor({
    output = "",
    mode = "preferred",
    position = "auto",
    scale = "auto",
})

-- Apariencia y comportamiento general.
hl.config({
    general = {
        gaps_in = 6,
        gaps_out = 10,
        border_size = 2,
        resize_on_border = true,
        allow_tearing = false,
        layout = "dwindle",
    },

    decoration = {
        rounding = 20,
        rounding_power = 2,

        active_opacity = 1.0,
        inactive_opacity = 1.0,

        shadow = {
            enabled = true,
            range = 4,
            render_power = 3,
        },

        blur = {
            enabled = true,
            size = 8,
            passes = 2,
            vibrancy = 0.15,
        },
    },

    animations = {
        enabled = true,
    },

    dwindle = {
        preserve_split = true,
    },

    misc = {
        -- Noctalia administra el fondo y la identidad visual de esta sesión.
        disable_hyprland_logo = true,
        force_default_wallpaper = 0,
    },

    input = {
        -- Estos tres valores son sustituidos desde korunix.localization.
        kb_layout = "@KORUNIX_KEYBOARD_LAYOUTS@",
        kb_variant = "@KORUNIX_KEYBOARD_VARIANTS@",
        kb_options = "@KORUNIX_KEYBOARD_OPTIONS@",

        numlock_by_default = true,
        follow_mouse = 1,
        sensitivity = 0,

        touchpad = {
            tap_to_click = true,
            natural_scroll = true,
            disable_while_typing = true,
        },
    },
})

-- Cursor común de Korunix.
hl.env("XCURSOR_THEME", "Bibata-Modern-Classic")
hl.env("XCURSOR_SIZE", "24")
hl.env("HYPRCURSOR_SIZE", "24")

local noctalia = "noctalia msg"

-- ============================================================
-- Atajos semánticos de Korunix
-- ============================================================

-- Aplicaciones principales.
hl.bind("SUPER + T", hl.dsp.exec_cmd("alacritty"))
hl.bind("SUPER + E", hl.dsp.exec_cmd("nautilus"))
hl.bind("SUPER + B", hl.dsp.exec_cmd("firefox"))

-- Noctalia.
hl.bind(
    "SUPER + Space",
    hl.dsp.exec_cmd(noctalia .. " panel-toggle launcher")
)

hl.bind(
    "SUPER + S",
    hl.dsp.exec_cmd(noctalia .. " panel-toggle control-center")
)

hl.bind(
    "SUPER + Comma",
    hl.dsp.exec_cmd(noctalia .. " settings-toggle")
)

hl.bind(
    "SUPER + L",
    hl.dsp.exec_cmd(noctalia .. " session lock")
)

-- Capturas.
--
-- La implementación interna difiere de Niri, pero la intención humana
-- permanece exactamente igual.
hl.bind(
    "Print",
    hl.dsp.exec_cmd(noctalia .. " screenshot-region")
)

hl.bind(
    "CTRL + Print",
    hl.dsp.exec_cmd(noctalia .. " screenshot-fullscreen")
)

hl.bind(
    "SHIFT + Print",
    hl.dsp.exec_cmd(noctalia .. " screenshot-fullscreen all")
)

-- Multimedia y brillo.
hl.bind(
    "XF86AudioRaiseVolume",
    hl.dsp.exec_cmd(noctalia .. " volume-up"),
    { locked = true, repeating = true }
)

hl.bind(
    "XF86AudioLowerVolume",
    hl.dsp.exec_cmd(noctalia .. " volume-down"),
    { locked = true, repeating = true }
)

hl.bind(
    "XF86AudioMute",
    hl.dsp.exec_cmd(noctalia .. " volume-mute"),
    { locked = true }
)

hl.bind(
    "XF86AudioMicMute",
    hl.dsp.exec_cmd(
        "wpctl set-mute @DEFAULT_AUDIO_SOURCE@ toggle"
    ),
    { locked = true }
)

hl.bind(
    "XF86MonBrightnessUp",
    hl.dsp.exec_cmd(noctalia .. " brightness-up"),
    { locked = true, repeating = true }
)

hl.bind(
    "XF86MonBrightnessDown",
    hl.dsp.exec_cmd(noctalia .. " brightness-down"),
    { locked = true, repeating = true }
)

hl.bind(
    "XF86AudioPlay",
    hl.dsp.exec_cmd(noctalia .. " media toggle"),
    { locked = true }
)

hl.bind(
    "XF86AudioPrev",
    hl.dsp.exec_cmd(noctalia .. " media previous"),
    { locked = true }
)

hl.bind(
    "XF86AudioNext",
    hl.dsp.exec_cmd(noctalia .. " media next"),
    { locked = true }
)

-- Ventanas.
hl.bind(
    "SUPER + Q",
    hl.dsp.window.close()
)

hl.bind(
    "SUPER + V",
    hl.dsp.window.float({ action = "toggle" })
)

hl.bind(
    "SUPER + F",
    hl.dsp.window.fullscreen({
        mode = "maximized",
        action = "toggle",
    })
)

hl.bind(
    "SUPER + SHIFT + F",
    hl.dsp.window.fullscreen({
        mode = "fullscreen",
        action = "toggle",
    })
)

-- Foco direccional.
hl.bind(
    "SUPER + Left",
    hl.dsp.focus({ direction = "left" })
)

hl.bind(
    "SUPER + Right",
    hl.dsp.focus({ direction = "right" })
)

hl.bind(
    "SUPER + Up",
    hl.dsp.focus({ direction = "up" })
)

hl.bind(
    "SUPER + Down",
    hl.dsp.focus({ direction = "down" })
)

-- Mover ventanas.
hl.bind(
    "SUPER + CTRL + Left",
    hl.dsp.window.move({ direction = "left" })
)

hl.bind(
    "SUPER + CTRL + Right",
    hl.dsp.window.move({ direction = "right" })
)

hl.bind(
    "SUPER + CTRL + Up",
    hl.dsp.window.move({ direction = "up" })
)

hl.bind(
    "SUPER + CTRL + Down",
    hl.dsp.window.move({ direction = "down" })
)

-- Workspaces 1–10.
-- La fila numérica y el numpad comparten exactamente la misma semántica.
for i = 1, 9 do
    hl.bind(
        "SUPER + " .. i,
        hl.dsp.focus({ workspace = i })
    )

    hl.bind(
        "SUPER + SHIFT + " .. i,
        hl.dsp.window.move({ workspace = i })
    )
end

-- La tecla 0 representa el espacio 10.
hl.bind(
    "SUPER + 0",
    hl.dsp.focus({ workspace = 10 })
)

hl.bind(
    "SUPER + SHIFT + 0",
    hl.dsp.window.move({ workspace = 10 })
)

-- Un mismo botón físico del numpad puede producir dos keysyms diferentes
-- según Bloq Num y Shift. Ambas variantes apuntan al mismo espacio.
local keypad_workspaces = {
    { workspace = 1, number = "KP_1", navigation = "KP_End" },
    { workspace = 2, number = "KP_2", navigation = "KP_Down" },
    { workspace = 3, number = "KP_3", navigation = "KP_Next" },
    { workspace = 4, number = "KP_4", navigation = "KP_Left" },
    { workspace = 5, number = "KP_5", navigation = "KP_Begin" },
    { workspace = 6, number = "KP_6", navigation = "KP_Right" },
    { workspace = 7, number = "KP_7", navigation = "KP_Home" },
    { workspace = 8, number = "KP_8", navigation = "KP_Up" },
    { workspace = 9, number = "KP_9", navigation = "KP_Prior" },
    { workspace = 10, number = "KP_0", navigation = "KP_Insert" },
}

for _, key in ipairs(keypad_workspaces) do
    hl.bind(
        "SUPER + " .. key.number,
        hl.dsp.focus({ workspace = key.workspace })
    )

    hl.bind(
        "SUPER + " .. key.navigation,
        hl.dsp.focus({ workspace = key.workspace })
    )

    hl.bind(
        "SUPER + SHIFT + " .. key.number,
        hl.dsp.window.move({ workspace = key.workspace })
    )

    hl.bind(
        "SUPER + SHIFT + " .. key.navigation,
        hl.dsp.window.move({ workspace = key.workspace })
    )
end

hl.bind(
    "SUPER + Tab",
    hl.dsp.focus({ workspace = "previous_per_monitor" })
)

-- Rueda sobre Super: recorrer workspaces existentes.
hl.bind(
    "SUPER + mouse_down",
    hl.dsp.focus({ workspace = "e+1" })
)

hl.bind(
    "SUPER + mouse_up",
    hl.dsp.focus({ workspace = "e-1" })
)

-- Arrastrar / redimensionar ventanas con el ratón.
hl.bind(
    "SUPER + mouse:272",
    hl.dsp.window.drag(),
    { mouse = true }
)

hl.bind(
    "SUPER + mouse:273",
    hl.dsp.window.resize(),
    { mouse = true }
)

-- ============================================================
-- Integración visual con Noctalia
-- ============================================================

-- La ventana de configuración de Noctalia se comporta como panel flotante.
hl.window_rule({
    name = "noctalia-settings",
    match = {
        class = "dev.noctalia.Noctalia",
    },
    float = true,
    size = { 1080, 920 },
})

-- Hyprland aplica el blur a las superficies layer-shell de Noctalia.
-- Las animaciones del propio compositor se desactivan para estas capas porque
-- Noctalia administra sus propias transiciones.
hl.layer_rule({
    name = "noctalia",
    match = {
        namespace = "^noctalia-(bar-.+|notification|dock|panel|attached-panel|osd|window-switcher)$",
    },
    no_anim = true,
    ignore_alpha = 0.5,
    blur = true,
    blur_popups = true,
})

-- Evita problemas de foco con determinadas ventanas vacías de XWayland.
hl.window_rule({
    name = "xwayland-empty-window",
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

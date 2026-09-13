{pkgs, ...}: {
  # xwayland-satellite 0.8.2 rompe los menús emergentes de Steam bajo Niri.
  # Mantenemos 0.8.1 hasta que la corrección llegue a una versión posterior.
  nixpkgs.overlays = [
    (_final: prev: {
      xwayland-satellite = prev.xwayland-satellite.overrideAttrs (finalAttrs: old: {
        version = "0.8.1";
        src = prev.fetchFromGitHub {
          owner = "Supreeeme";
          repo = "xwayland-satellite";
          rev = "v0.8.1";
          hash = "sha256-BUE41HjLIGPjq3U8VXPjf8asH8GaMI7FYdgrIHKFMXA=";
        };
        cargoHash = "sha256-16L6gsvze+m7XCJlOA1lsPNELE3D364ef2FTdkh0rVY=";
        cargoDeps = prev.rustPlatform.fetchCargoVendor {
          inherit (old) pname;
          inherit (finalAttrs) src version;
          patches = old.cargoDeps.vendorStaging.patches or [];
          hash = finalAttrs.cargoHash;
        };
      });
    })
  ];

  # Activar Niri (Wayland nativo) a nivel de sistema.
  programs.niri.enable = true;

  # Niri debe usar la configuración declarativa de Korunix aunque Noctalia cree
  # una configuración de usuario para sus fragmentos dinámicos.
  environment.variables.NIRI_CONFIG = "/etc/niri/config.kdl";

  environment.etc."niri/config.kdl".text = ''
    // ----------------------------------------
    // SESIÓN AISLADA DE PLASMA
    // ----------------------------------------
    // Estas variables afectan únicamente a los procesos que Niri lanza.
    // GTK no debe detectar Plasma como escritorio activo; Qt/KDE sí usa su
    // integración de plataforma para leer el KColorScheme generado por Noctalia.
    environment {
        XDG_CURRENT_DESKTOP "niri"
        XDG_SESSION_DESKTOP "niri"
        XDG_SESSION_TYPE "wayland"
        QT_QPA_PLATFORM "wayland"
        GDK_BACKEND "wayland"
        QT_QPA_PLATFORMTHEME "kde"
        QT_QPA_PLATFORMTHEME_QT6 "kde"
        KDE_FULL_SESSION null
        KDE_SESSION_VERSION null
        KDE_SESSION_UID null
    }

    // ----------------------------------------
    // ENTRADA (Teclado y Touchpad)
    // ----------------------------------------
    input {
        keyboard {
            xkb {
                layout "es"
                variant "deadtilde"
            }
            numlock
        }
        touchpad {
            tap
            natural-scroll
        }
        mouse {
        }
        warp-mouse-to-focus
        focus-follows-mouse
    }

    include "/etc/niri/monitor.kdl"

    hotkey-overlay {
        skip-at-startup
    }

    prefer-no-csd

    screenshot-path "~/Imágenes/Capturas de pantalla/Captura de pantalla %Y-%m-%d %H-%M-%S.png"

    layer-rule {
        match namespace="^noctalia-backdrop"
        place-within-backdrop true
    }

    animations {
    }

    window-rule {
        geometry-corner-radius 20
        clip-to-geometry true
        draw-border-with-background false
    }

    debug {
        honor-xdg-activation-with-invalid-serial
    }

    window-rule {
        match app-id=r#"zen$"# title="^Picture-in-Picture$"
        open-floating true
    }

    window-rule {
        background-effect {
            blur true
            xray false
        }
    }

    // Todas las ventanas parten de 0.90; las inactivas bajan después a 0.85.
    window-rule {
        opacity 0.90
    }

    window-rule {
        match is-active=false
        opacity 0.85
    }

    layer-rule {
        match namespace=r#"^noctalia-(bar-[^\"]+|notification|dock|panel|attached-panel|osd)$"#
        background-effect {
            xray false
        }
    }

    blur {
        passes 1
        offset 3.0
        noise 0.03
        saturation 1.0
    }

    binds {
        Mod+Space { spawn "noctalia" "msg" "panel-toggle" "launcher"; }
        Mod+S { spawn "noctalia" "msg" "panel-toggle" "control-center"; }
        Mod+Comma { spawn "noctalia" "msg" "settings-toggle"; }
        Mod+L { spawn "noctalia" "msg" "session" "lock"; }
        XF86AudioRaiseVolume { spawn "noctalia" "msg" "volume-up"; }
        XF86AudioLowerVolume { spawn "noctalia" "msg" "volume-down"; }
        XF86AudioMute { spawn "noctalia" "msg" "volume-mute"; }
        XF86MonBrightnessUp { spawn "noctalia" "msg" "brightness-up"; }
        XF86MonBrightnessDown { spawn "noctalia" "msg" "brightness-down"; }
        Mod+T hotkey-overlay-title="Abrir terminal: Alacritty" { spawn "alacritty"; }
        XF86AudioMicMute allow-when-locked=true { spawn-sh "wpctl set-mute @DEFAULT_AUDIO_SOURCE@ toggle"; }
        XF86AudioPlay allow-when-locked=true { spawn "noctalia" "msg" "media" "toggle"; }
        XF86AudioStop allow-when-locked=true { spawn "noctalia" "msg" "media" "stop"; }
        XF86AudioPrev allow-when-locked=true { spawn "noctalia" "msg" "media" "previous"; }
        XF86AudioNext allow-when-locked=true { spawn "noctalia" "msg" "media" "next"; }
        Mod+O repeat=false { toggle-overview; }
        Mod+Q repeat=false { close-window; }
        Mod+Left { focus-column-left; }
        Mod+Down { focus-window-down; }
        Mod+Up { focus-window-up; }
        Mod+Right { focus-column-right; }
        Mod+Ctrl+Left { move-column-left; }
        Mod+Ctrl+Down { move-window-down; }
        Mod+Ctrl+Up { move-window-up; }
        Mod+Ctrl+Right { move-column-right; }
        Mod+Ctrl+H { move-column-left; }
        Mod+Ctrl+J { move-window-down; }
        Mod+Ctrl+K { move-window-up; }
        Mod+Ctrl+L { move-column-right; }
        Mod+Home { focus-column-first; }
        Mod+End { focus-column-last; }
        Mod+Ctrl+Home { move-column-to-first; }
        Mod+Ctrl+End { move-column-to-last; }
        Mod+Shift+Left { focus-monitor-left; }
        Mod+Shift+Down { focus-monitor-down; }
        Mod+Shift+Up { focus-monitor-up; }
        Mod+Shift+Right { focus-monitor-right; }
        Mod+Shift+H { focus-monitor-left; }
        Mod+Shift+J { focus-monitor-down; }
        Mod+Shift+K { focus-monitor-up; }
        Mod+Shift+L { focus-monitor-right; }
        Mod+Shift+Ctrl+Left { move-column-to-monitor-left; }
        Mod+Shift+Ctrl+Down { move-window-down; }
        Mod+Shift+Ctrl+Up { move-window-up; }
        Mod+Shift+Ctrl+Right { move-column-to-monitor-right; }
        Mod+Shift+Ctrl+H { move-column-to-monitor-left; }
        Mod+Shift+Ctrl+J { move-window-down; }
        Mod+Shift+Ctrl+K { move-window-up; }
        Mod+Shift+Ctrl+L { move-column-to-monitor-right; }
        Mod+E { spawn "nautilus"; }
        Mod+B { spawn "zen"; }
        Mod+Page_Down { focus-workspace-down; }
        Mod+Page_Up { focus-workspace-up; }
        Mod+U { focus-workspace-down; }
        Mod+I { focus-workspace-up; }
        Mod+Ctrl+Page_Down { move-column-to-workspace-down; }
        Mod+Ctrl+Page_Up { move-column-to-workspace-up; }
        Mod+Ctrl+U { move-column-to-workspace-down; }
        Mod+Ctrl+I { move-column-to-workspace-up; }
        Mod+Shift+Page_Down { move-workspace-down; }
        Mod+Shift+Page_Up { move-workspace-up; }
        Mod+Shift+U { move-workspace-down; }
        Mod+Shift+I { move-workspace-up; }
        Mod+WheelScrollDown cooldown-ms=150 { focus-workspace-down; }
        Mod+WheelScrollUp cooldown-ms=150 { focus-workspace-up; }
        Mod+Ctrl+WheelScrollDown cooldown-ms=150 { move-column-to-workspace-down; }
        Mod+Ctrl+WheelScrollUp cooldown-ms=150 { move-column-to-workspace-up; }
        Mod+WheelScrollRight { focus-column-right; }
        Mod+WheelScrollLeft { focus-column-left; }
        Mod+Ctrl+WheelScrollRight { move-column-right; }
        Mod+Ctrl+WheelScrollLeft { move-column-left; }
        Mod+Shift+WheelScrollDown { focus-column-right; }
        Mod+Shift+WheelScrollUp { focus-column-left; }
        Mod+Ctrl+Shift+WheelScrollDown { move-column-right; }
        Mod+Ctrl+Shift+WheelScrollUp { move-column-left; }
        Mod+1 { focus-workspace 1; }
        Mod+2 { focus-workspace 2; }
        Mod+3 { focus-workspace 3; }
        Mod+4 { focus-workspace 4; }
        Mod+5 { focus-workspace 5; }
        Mod+6 { focus-workspace 6; }
        Mod+7 { focus-workspace 7; }
        Mod+8 { focus-workspace 8; }
        Mod+9 { focus-workspace 9; }
        Mod+Shift+1 { move-window-to-workspace 1; }
        Mod+Shift+2 { move-window-to-workspace 2; }
        Mod+Shift+3 { move-window-to-workspace 3; }
        Mod+Shift+4 { move-window-to-workspace 4; }
        Mod+Shift+5 { move-window-to-workspace 5; }
        Mod+Shift+6 { move-window-to-workspace 6; }
        Mod+Shift+7 { move-window-to-workspace 7; }
        Mod+Shift+8 { move-window-to-workspace 8; }
        Mod+Shift+9 { move-window-to-workspace 9; }
        Mod+Tab { focus-workspace-previous; }
        Mod+BracketLeft { consume-or-expel-window-left; }
        Mod+BracketRight { consume-or-expel-window-right; }
        Mod+Period { expel-window-from-column; }
        Mod+R { switch-preset-column-width; }
        Mod+Shift+R { switch-preset-column-width-back; }
        Mod+Ctrl+Shift+R { switch-preset-window-height; }
        Mod+Ctrl+R { reset-window-height; }
        Mod+F { maximize-column; }
        Mod+Shift+F { fullscreen-window; }
        Mod+M { maximize-window-to-edges; }
        Mod+Ctrl+F { expand-column-to-available-width; }
        Mod+C { center-column; }
        Mod+Ctrl+C { center-visible-columns; }
        Mod+Minus { set-column-width "-10%"; }
        Mod+Equal { set-column-width "+10%"; }
        Mod+Shift+Minus { set-window-height "-10%"; }
        Mod+Shift+Equal { set-window-height "+10%"; }
        Mod+V { toggle-window-floating; }
        Mod+Shift+V { switch-focus-between-floating-and-tiling; }
        Mod+W { toggle-column-tabbed-display; }
        Print { spawn "noctalia" "msg" "screenshot-region"; }
        Ctrl+Print { spawn "noctalia" "msg" "screenshot-fullscreen"; }
        Alt+Print { screenshot-window; }
        Mod+Escape allow-inhibiting=false { toggle-keyboard-shortcuts-inhibit; }
        Mod+Shift+E { quit; }
        Ctrl+Alt+Delete { quit; }
        Mod+Shift+P { power-off-monitors; }
    }

    // Configuración de Noctalia
    spawn-at-startup "dconf" "write" "/org/gnome/desktop/interface/icon-theme" "'Hatter-Slate'"
    spawn-at-startup "noctalia"
    spawn-at-startup "xwayland-satellite"
    include optional=true "~/.config/niri/noctalia.kdl"
  '';
}

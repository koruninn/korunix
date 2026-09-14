{
  config,
  pkgs,
  ...
}: let
  materialYou = pkgs.python3Packages."kde-material-you-colors";
  noctaliaPackage = config.programs.noctalia.package;

  plasmaThemeBridge = pkgs.writeShellApplication {
    name = "korunix-plasma-theme";
    runtimeInputs = with pkgs; [
      bash
      coreutils
      dbus
      gawk
      glib
      jq
    ];
    text = ''
      # Este puente solo puede ser dueño de los colores mientras Plasma está
      # realmente activo. Si se invoca desde Niri/Umbriel, no toca nada.
      case "''${XDG_CURRENT_DESKTOP:-}" in
        *KDE*) ;;
        *) exit 0 ;;
      esac

      config_home="''${XDG_CONFIG_HOME:-$HOME/.config}"
      state_home="''${XDG_STATE_HOME:-$HOME/.local/state}"
      material_json="''${TMPDIR:-/tmp}/kde-material-you-colors-$USER.json"
      noctalia_config="$config_home/noctalia/config.toml"
      community_root="$state_home/noctalia/community-templates"

      if [ ! -f "$noctalia_config" ] && [ -f /etc/noctalia/config.toml ]; then
        noctalia_config=/etc/noctalia/config.toml
      fi

      # KDE Material You Colors escribe este JSON antes de ejecutar el hook.
      # Si todavía no existe, no reutilizamos jamás los últimos colores de
      # Noctalia: esperamos al siguiente evento del propio backend de Plasma.
      if [ ! -s "$material_json" ]; then
        exit 0
      fi

      # Plasma vuelve a apropiarse de GTK después de haber aplicado su nueva
      # paleta. Así Breeze se sincroniza con el color actual y no con el que
      # hubiera quedado al entrar en la sesión.
      korunix-gtk-session plasma

      dbus-send \
        --session \
        --type=method_call \
        --print-reply \
        --dest=org.kde.kded6 \
        /kded \
        org.kde.kded6.loadModule \
        string:gtkconfig \
        >/dev/null 2>&1 || true

      dbus-send \
        --session \
        --type=method_call \
        --print-reply \
        --dest=org.kde.GtkConfig \
        /GtkConfig \
        org.kde.GtkConfig.setGtkTheme \
        string:Breeze \
        >/dev/null 2>&1 || true

      theme_json="$(mktemp)"
      trap 'rm -f "$theme_json"' EXIT

      # KDE Material You Colors usa nombres camelCase y Noctalia snake_case.
      # Conservamos la paleta completa de Plasma; no reconstruimos otra desde
      # un único AccentColor.
      if ! jq '
        def snake:
          gsub("(?<c>[A-Z])"; "_\(.c)") | ascii_downcase;
        {
          dark: (.schemes.dark // {} | with_entries(.key |= snake)),
          light: (.schemes.light // {} | with_entries(.key |= snake))
        }
      ' "$material_json" > "$theme_json"; then
        exit 0
      fi

      if ! jq -e '
        .dark.primary and
        .dark.surface and
        .dark.on_surface and
        .light.primary and
        .light.surface and
        .light.on_surface
      ' "$theme_json" >/dev/null; then
        exit 0
      fi

      mode="$(jq -r 'if .light == true then "light" else "dark" end' "$material_json")"
      wallpaper="$(jq -r '.wallpaper.data // empty' "$material_json")"

      template_ids() {
        [ -f "$noctalia_config" ] || return 0
        awk '
          /^[[:space:]]*community_ids[[:space:]]*=[[:space:]]*\[/ {
            inside = 1
            next
          }
          inside && /^[[:space:]]*\]/ {
            exit
          }
          inside {
            line = $0
            while (match(line, /"[^"]+"/)) {
              print substr(line, RSTART + 1, RLENGTH - 2)
              line = substr(line, RSTART + RLENGTH)
            }
          }
        ' "$noctalia_config"
      }

      # Las plantillas son compartidas, pero durante Plasma se vuelven a
      # renderizar con la paleta completa de Plasma. Al volver a Niri/Umbriel,
      # Noctalia las vuelve a generar con su propia paleta al arrancar.
      while IFS= read -r id; do
        [ -n "$id" ] || continue
        template_config="$community_root/$id/template.toml"
        [ -f "$template_config" ] || continue

        args=(
          theme
          --theme-json "$theme_json"
          --scheme m3-tonal-spot
          --default-mode "$mode"
          --config "$template_config"
        )

        # Mantener la imagen permite que sigan funcionando plantillas que usan
        # {{ image }} además de los tokens Material.
        if [ -n "$wallpaper" ] && [ -f "$wallpaper" ]; then
          args=(theme "$wallpaper" "''${args[@]:1}")
        fi

        if ! ${noctaliaPackage}/bin/noctalia "''${args[@]}" >/dev/null 2>&1; then
          echo "korunix-plasma-theme: no se pudo aplicar la plantilla $id" >&2
        fi
      done < <(template_ids)
    '';
  };
in {
  services.desktopManager.plasma6.enable = true;

  environment.plasma6.excludePackages = [
    pkgs.kdePackages.kwin-x11
  ];

  # Plasma tiene un único dueño de color: KDE Material You Colors. No dejamos
  # en paralelo el puente AccentColor -> Matugen que podía conservar o inventar
  # una paleta distinta de la que Plasma estaba mostrando.
  environment.systemPackages = [
    materialYou
    plasmaThemeBridge
  ];

  # El backend ejecuta el puente después de cada cambio de fondo, modo o ajuste.
  # El mismo backend queda expuesto directamente al plasmoid, sin un watcher
  # adicional de Korunix entre ambos.
  environment.etc."xdg/autostart/kde-material-you-colors.desktop".text = ''
[Desktop Entry]
Type=Application
Name=KDE Material You Colors
Comment=Genera los colores de Plasma a partir del fondo de pantalla
Exec=${materialYou}/bin/kde-material-you-colors --on-change-hook ${plasmaThemeBridge}/bin/korunix-plasma-theme
Icon=color-management
OnlyShowIn=KDE;
NoDisplay=true
X-KDE-AutostartScript=true
  '';

  # kde-gtk-config permanece activo en Plasma; el hook anterior lo vuelve a
  # sincronizar una vez que la nueva paleta Material ya fue aplicada.
  environment.etc."xdg/kded6rc".text = ''
[Module-gtkconfig]
autoload=true
  '';

  environment.etc."xdg/kcminputrc".text = ''
[Keyboard]
NumLock=0
  '';

  environment.etc."xdg/kde-mimeapps.list".text = ''
[Default Applications]
inode/directory=org.kde.dolphin.desktop;
  '';

  environment.etc."xdg/plasma-org.kde.plasma.desktop-appletsrc".text = ''
[Containments][2][Applets][5][Configuration][General]
launchers=applications:systemsettings.desktop,applications:org.kde.dolphin.desktop,applications:zen.desktop
  '';

  environment.variables = {
    XKB_DEFAULT_LAYOUT = "es";
    XKB_DEFAULT_VARIANT = "deadtilde";
  };
}

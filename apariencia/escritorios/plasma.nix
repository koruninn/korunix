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
      gawk
      glib
      jq
    ];
    text = ''
      config_home="''${XDG_CONFIG_HOME:-$HOME/.config}"
      state_home="''${XDG_STATE_HOME:-$HOME/.local/state}"
      noctalia_templates="${noctaliaPackage}/share/noctalia/assets/templates"
      material_json="''${TMPDIR:-/tmp}/kde-material-you-colors-$USER.json"
      noctalia_config=/etc/noctalia/config.toml

      run_undo() {
        local script="$1"
        if [ -f "$script" ]; then
          bash "$script" >/dev/null 2>&1 || true
        fi
      }

      # Plasma no debe heredar la capa global de Noctalia. Las plantillas de
      # aplicaciones se tratan aparte, usando la paleta actual de Material You.
      run_undo "$noctalia_templates/alacritty/undo.sh"
      run_undo "$noctalia_templates/gtk/undo-gtk3.sh"
      run_undo "$noctalia_templates/gtk/undo-gtk4.sh"
      run_undo "$noctalia_templates/qt/undo.sh"

      if command -v gsettings >/dev/null 2>&1; then
        gsettings set org.gnome.desktop.interface gtk-theme 'Breeze' >/dev/null 2>&1 || true
      fi

      # KDE Material You Colors exporta su paleta completa después de cada
      # cambio de fondo o de modo. No borramos las plantillas comunitarias:
      # las volvemos a renderizar con esa paleta, por lo que las aplicaciones
      # pueden conservar exactamente el mismo tema seleccionado en ambos
      # escritorios sin compartir los colores de Noctalia con Plasma.
      if [ ! -s "$material_json" ]; then
        exit 0
      fi

      theme_json=$(mktemp)
      trap 'rm -f "$theme_json"' EXIT

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

      mode=$(jq -r 'if .light == true then "light" else "dark" end' "$material_json")
      wallpaper=$(jq -r '.wallpaper.data // empty' "$material_json")
      community_root="$state_home/noctalia/community-templates"

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

        # El tema procede del JSON de Material You, pero pasar también la imagen
        # conserva {{ image }} para cualquier plantilla que la necesite.
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
  # Plasma 6 únicamente como sesión Wayland.
  services.desktopManager.plasma6.enable = true;

  environment.plasma6.excludePackages = [
    pkgs.kdePackages.kwin-x11
  ];

  # Material You gobierna Plasma. El puente reutiliza sus colores para las
  # mismas plantillas de aplicaciones que Noctalia mantiene en Niri/Umbriel.
  environment.systemPackages = [
    materialYou
    plasmaThemeBridge
  ];

  # KDE Material You Colors escribe su JSON de colores antes de ejecutar el
  # hook. Así cada cambio de fondo o de modo vuelve a renderizar las plantillas
  # comunitarias con la paleta de Plasma, sin cambiar qué tema tiene escogido
  # cada aplicación.
  environment.etc."xdg/autostart/kde-material-you-colors.desktop".text = ''
[Desktop Entry]
Type=Application
Name=KDE Material You Colors
Comment=Colores dinámicos de Plasma a partir del fondo
Exec=${materialYou}/bin/kde-material-you-colors --on-change-hook ${plasmaThemeBridge}/bin/korunix-plasma-theme
Icon=color-management
OnlyShowIn=KDE;
X-KDE-AutostartScript=true
  '';

  # kde-gtk-config mantiene Breeze GTK sincronizado con el esquema de colores
  # activo de Plasma, incluido el que genera KDE Material You Colors.
  environment.etc."xdg/kded5rc".text = ''
[Module-gtkconfig]
autoload=true
  '';

  # Num Lock encendido al iniciar Plasma.
  environment.etc."xdg/kcminputrc".text = ''
[Keyboard]
NumLock=0
  '';

  # Plasma usa Dolphin como gestor de archivos sin cambiar Umbriel.
  environment.etc."xdg/kde-mimeapps.list".text = ''
[Default Applications]
inode/directory=org.kde.dolphin.desktop;
  '';

  # Orden declarativo de los lanzadores fijados del panel de Plasma.
  environment.etc."xdg/plasma-org.kde.plasma.desktop-appletsrc".text = ''
[Containments][2][Applets][5][Configuration][General]
launchers=applications:systemsettings.desktop,applications:org.kde.dolphin.desktop,applications:zen.desktop
  '';

  environment.variables = {
    XKB_DEFAULT_LAYOUT = "es";
    XKB_DEFAULT_VARIANT = "deadtilde";
  };
}

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
      case "''${XDG_CURRENT_DESKTOP:-}" in
        *KDE*) ;;
        *) exit 0 ;;
      esac

      config_home="''${XDG_CONFIG_HOME:-$HOME/.config}"
      state_home="''${XDG_STATE_HOME:-$HOME/.local/state}"
      material_json="''${1:-}"
      noctalia_config="$config_home/noctalia/config.toml"
      community_root="$state_home/noctalia/community-templates"
      log_dir="$state_home/korunix"
      log_file="$log_dir/plasma-theme.log"

      mkdir -p "$log_dir"

      log() {
        printf '%s %s\n' "$(date -Is)" "$*" >> "$log_file"
      }

      if [ -z "$material_json" ]; then
        material_json="''${TMPDIR:-/tmp}/kde-material-you-colors-$USER.json"
      fi

      if [ ! -f "$noctalia_config" ] && [ -f /etc/noctalia/config.toml ]; then
        noctalia_config=/etc/noctalia/config.toml
      fi

      if [ ! -s "$material_json" ]; then
        log "JSON ausente: $material_json"
        exit 0
      fi

      if [ ! -d "$community_root" ]; then
        log "Caché de plantillas ausente: $community_root"
        exit 0
      fi

      theme_json="$(mktemp)"
      trap 'rm -f "$theme_json"' EXIT

      if ! jq '
        def snake:
          gsub("(?<c>[A-Z])"; "_\(.c)") | ascii_downcase;
        {
          dark: (.schemes.dark // {} | with_entries(.key |= snake)),
          light: (.schemes.light // {} | with_entries(.key |= snake))
        }
      ' "$material_json" > "$theme_json"; then
        log "No se pudo convertir el JSON de KDE Material You Colors"
        exit 1
      fi

      if ! jq -e '
        .dark.primary and
        .dark.surface and
        .dark.on_surface and
        .light.primary and
        .light.surface and
        .light.on_surface
      ' "$theme_json" >/dev/null; then
        log "El JSON no contiene una paleta Material completa"
        exit 1
      fi

      mode="$(jq -r 'if .light == true then "light" else "dark" end' "$material_json")"
      wallpaper="$(jq -r '.wallpaper.data // empty' "$material_json")"
      seed="$(jq -r '.seed.color // .schemes.dark.primary // "desconocido"' "$material_json")"

      # Plasma recupera GTK solo después de que KDE Material You Colors haya
      # escrito la nueva paleta. Así Breeze no conserva el acento de la sesión
      # anterior de Noctalia.
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

      applied=0
      failed=0
      missing=0

      while IFS= read -r id; do
        [ -n "$id" ] || continue
        template_config="$community_root/$id/template.toml"

        if [ ! -f "$template_config" ]; then
          missing=$((missing + 1))
          log "Plantilla no encontrada: $id ($template_config)"
          continue
        fi

        args=(
          theme
          --theme-json "$theme_json"
          --scheme m3-tonal-spot
          --default-mode "$mode"
          --config "$template_config"
        )

        if [ -n "$wallpaper" ] && [ -f "$wallpaper" ]; then
          args=(theme "$wallpaper" "''${args[@]:1}")
        fi

        output=""
        if output="$(${noctaliaPackage}/bin/noctalia "''${args[@]}" 2>&1)"; then
          applied=$((applied + 1))
        else
          failed=$((failed + 1))
          log "ERROR plantilla $id: $output"
        fi
      done < <(template_ids)

      log "Paleta Plasma aplicada: seed=$seed modo=$mode aplicadas=$applied fallidas=$failed ausentes=$missing"

      if [ "$failed" -gt 0 ]; then
        exit 1
      fi
    '';
  };

  plasmaMaterialYouSession = pkgs.writeShellApplication {
    name = "korunix-plasma-material-you";
    runtimeInputs = with pkgs; [
      bash
      coreutils
      gawk
      inotify-tools
      materialYou
      plasmaThemeBridge
    ];
    text = ''
      case "''${XDG_CURRENT_DESKTOP:-}" in
        *KDE*) ;;
        *) exit 0 ;;
      esac

      # Forzamos el mismo directorio temporal para el backend y el supervisor.
      export TMPDIR=/tmp
      material_json="$TMPDIR/kde-material-you-colors-$USER.json"
      material_dir="$(dirname "$material_json")"
      state_home="''${XDG_STATE_HOME:-$HOME/.local/state}"
      log_dir="$state_home/korunix"
      log_file="$log_dir/plasma-theme.log"
      backend_pid=""
      last_hash=""

      mkdir -p "$log_dir"
      rm -f "$material_json"

      log() {
        printf '%s %s\n' "$(date -Is)" "$*" >> "$log_file"
      }

      cleanup() {
        if [ -n "$backend_pid" ] && kill -0 "$backend_pid" 2>/dev/null; then
          kill "$backend_pid" 2>/dev/null || true
          wait "$backend_pid" 2>/dev/null || true
        fi
      }
      trap cleanup EXIT INT TERM HUP

      # El backend queda como proceso real de la sesión Plasma para que también
      # pueda ser detectado por el plasmoid oficial.
      ${materialYou}/bin/kde-material-you-colors &
      backend_pid=$!
      log "Backend KDE Material You Colors iniciado: pid=$backend_pid"

      while kill -0 "$backend_pid" 2>/dev/null; do
        if [ -s "$material_json" ]; then
          current_hash="$(sha256sum "$material_json" 2>/dev/null | awk '{print $1}' || true)"
          if [ -n "$current_hash" ] && [ "$current_hash" != "$last_hash" ]; then
            if ${plasmaThemeBridge}/bin/korunix-plasma-theme "$material_json"; then
              last_hash="$current_hash"
            else
              log "El puente devolvió error para hash=$current_hash"
            fi
          fi
        fi

        # KDE Material You Colors reescribe el JSON en cada cambio de fondo,
        # modo o configuración. Vigilamos el directorio para no depender de que
        # el archivo exista cuando empieza la sesión.
        inotifywait \
          -q \
          -t 30 \
          -e close_write,create,moved_to \
          "$material_dir" \
          >/dev/null 2>&1 || true
      done

      wait "$backend_pid"
    '';
  };
in {
  services.desktopManager.plasma6.enable = true;

  environment.plasma6.excludePackages = [
    pkgs.kdePackages.kwin-x11
  ];

  environment.systemPackages = [
    materialYou
    plasmaThemeBridge
    plasmaMaterialYouSession
  ];

  # Korunix supervisa directamente el JSON que genera el backend. Ya no
  # dependemos de un hook silencioso para aplicar las plantillas de Plasma.
  environment.etc."xdg/autostart/kde-material-you-colors.desktop".text = ''
[Desktop Entry]
Type=Application
Name=KDE Material You Colors
Comment=Genera los colores de Plasma a partir del fondo de pantalla
Exec=${plasmaMaterialYouSession}/bin/korunix-plasma-material-you
Icon=color-management
OnlyShowIn=KDE;
NoDisplay=true
X-KDE-AutostartScript=true
  '';

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

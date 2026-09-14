{
  config,
  equipo,
  pkgs,
  ...
}: let
  usuario = config.users.users.${equipo.persona};
  noctaliaPackage = config.programs.noctalia.package;

  gtkSession = pkgs.writeShellApplication {
    name = "korunix-gtk-session";
    runtimeInputs = with pkgs; [
      bash
      coreutils
      glib
      gnugrep
      gnused
    ];
    text = ''
      config_home="''${XDG_CONFIG_HOME:-$HOME/.config}"
      noctalia_apply="${noctaliaPackage}/share/noctalia/assets/templates/gtk/apply.sh"

      strip_noctalia() {
        local version file
        for version in 3 4; do
          file="$config_home/gtk-$version.0/gtk.css"
          if [ -f "$file" ]; then
            sed -i '/^[[:space:]]*@import.*noctalia\.css.*$/d' "$file"
          fi
        done
      }

      strip_kde() {
        local version file
        for version in 3 4; do
          file="$config_home/gtk-$version.0/gtk.css"
          if [ -f "$file" ]; then
            sed -i '/^[[:space:]]*@import.*colors\.css.*$/d' "$file"
          fi
        done
      }

      set_ini_value() {
        local file="$1"
        local key="$2"
        local value="$3"

        mkdir -p "$(dirname "$file")"
        if [ ! -f "$file" ]; then
          printf '%s\n' '[Settings]' > "$file"
        fi

        if grep -q "^$key=" "$file"; then
          sed -i "s|^$key=.*|$key=$value|" "$file"
        elif grep -q '^\[Settings\]$' "$file"; then
          sed -i "/^\[Settings\]$/a $key=$value" "$file"
        else
          printf '\n[Settings]\n%s=%s\n' "$key" "$value" >> "$file"
        fi
      }

      current_mode() {
        local mode scheme

        mode="$(${noctaliaPackage}/bin/noctalia msg theme-mode-get 2>/dev/null | tr -d '[:space:]' || true)"
        case "$mode" in
          light|dark)
            printf '%s\n' "$mode"
            return 0
            ;;
        esac

        scheme=$(gsettings get org.gnome.desktop.interface color-scheme 2>/dev/null || true)
        case "$scheme" in
          *light*) printf '%s\n' light ;;
          *) printf '%s\n' dark ;;
        esac
      }

      set_theme() {
        local theme="$1"
        local mode="$2"
        local dark=0
        local version

        if [ "$mode" = dark ]; then
          dark=1
        fi

        for version in 3 4; do
          set_ini_value "$config_home/gtk-$version.0/settings.ini" gtk-theme-name "$theme"
          set_ini_value "$config_home/gtk-$version.0/settings.ini" gtk-application-prefer-dark-theme "$dark"
        done

        if gsettings list-schemas 2>/dev/null | grep -qx org.gnome.desktop.interface; then
          gsettings set org.gnome.desktop.interface gtk-theme "$theme" >/dev/null 2>&1 || true
          gsettings set org.gnome.desktop.interface color-scheme "prefer-$mode" >/dev/null 2>&1 || true
        fi
      }

      case "''${1:-}" in
        plasma)
          strip_noctalia
          set_theme Breeze dark
          ;;

        noctalia)
          case "''${XDG_CURRENT_DESKTOP:-}" in
            *KDE*) exit 0 ;;
          esac

          strip_kde

          tries=0
          while [ "$tries" -lt 50 ]; do
            if [ -f "$config_home/gtk-3.0/noctalia.css" ] && [ -f "$config_home/gtk-4.0/noctalia.css" ]; then
              break
            fi
            tries=$((tries + 1))
            sleep 0.1
          done

          if [ -f "$config_home/gtk-3.0/noctalia.css" ] && [ -f "$config_home/gtk-4.0/noctalia.css" ]; then
            mode=$(current_mode)
            if [ "$mode" = light ]; then
              set_theme adw-gtk3 light
            else
              set_theme adw-gtk3-dark dark
            fi
            bash "$noctalia_apply" "$mode" >/dev/null 2>&1 || true
          fi
          ;;

        *)
          echo "Uso: korunix-gtk-session {plasma|noctalia}" >&2
          exit 2
          ;;
      esac
    '';
  };

  noctaliaSession = pkgs.writeShellApplication {
    name = "korunix-noctalia-session";
    runtimeInputs = [
      pkgs.coreutils
      gtkSession
    ];
    text = ''
      case "''${XDG_CURRENT_DESKTOP:-}" in
        *KDE*) exit 0 ;;
      esac

      state_home="''${XDG_STATE_HOME:-$HOME/.local/state}"
      log_dir="$state_home/korunix"
      log_file="$log_dir/noctalia-session.log"
      mkdir -p "$log_dir"

      log() {
        printf '%s %s\n' "$(date -Is)" "$*" >> "$log_file"
      }

      # Primero retiramos cualquier resto GTK de Plasma y recuperamos la capa
      # de Noctalia que ya exista, para no mostrar Breeze durante el arranque.
      korunix-gtk-session noctalia

      # Al cambiar desde Plasma no basta con volver a enlazar noctalia.css:
      # hay que regenerar las plantillas con la paleta que Noctalia acaba de
      # resolver para esta sesión. El IPC oficial fuerza esa reaplicación.
      tries=0
      while [ "$tries" -lt 50 ]; do
        if output="$(${noctaliaPackage}/bin/noctalia msg templates-apply 2>&1)"; then
          log "Plantillas de Noctalia reaplicadas: escritorio=''${XDG_CURRENT_DESKTOP:-desconocido}"
          korunix-gtk-session noctalia
          exit 0
        fi
        tries=$((tries + 1))
        sleep 0.1
      done

      log "No se pudieron reaplicar las plantillas de Noctalia: ''${output:-sin respuesta}"
      exit 1
    '';
  };

  noctaliaGtkHook = pkgs.writeText "noctalia-gtk-session.toml" ''
    [hooks]
    started = "korunix-noctalia-session"
    colors_changed = "korunix-gtk-session noctalia"
  '';
in {
  environment.systemPackages = [
    pkgs.adw-gtk3
    gtkSession
    noctaliaSession
  ];

  system.activationScripts.noctaliaGtkSession.text = ''
    install -d -m 0755 \
      -o ${usuario.name} \
      -g ${usuario.group} \
      ${usuario.home}/.config/noctalia

    install -m 0644 \
      -o ${usuario.name} \
      -g ${usuario.group} \
      ${noctaliaGtkHook} \
      ${usuario.home}/.config/noctalia/gtk-session.toml
  '';

  environment.etc = {
    "gtk-2.0/gtkrc".text = ''
      gtk-icon-theme-name="Hatter-Slate"
      gtk-cursor-theme-name="Bibata-Modern-Classic"
      gtk-cursor-theme-size=24
    '';

    "gtk-3.0/settings.ini".text = ''
      [Settings]
      gtk-icon-theme-name=Hatter-Slate
      gtk-cursor-theme-name=Bibata-Modern-Classic
      gtk-cursor-theme-size=24
    '';

    "gtk-4.0/settings.ini".text = ''
      [Settings]
      gtk-icon-theme-name=Hatter-Slate
      gtk-cursor-theme-name=Bibata-Modern-Classic
      gtk-cursor-theme-size=24
    '';
  };
}

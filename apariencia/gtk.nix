{
  config,
  pkgs,
  ...
}: let
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
        local scheme
        scheme=$(gsettings get org.gnome.desktop.interface color-scheme 2>/dev/null || true)
        case "$scheme" in
          *dark*) printf '%s\n' dark ;;
          *) printf '%s\n' light ;;
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

      restore_noctalia_identity() {
        local version

        for version in 3 4; do
          set_ini_value "$config_home/gtk-$version.0/settings.ini" gtk-icon-theme-name Hatter-Slate
          set_ini_value "$config_home/gtk-$version.0/settings.ini" gtk-cursor-theme-name Bibata-Modern-Classic
          set_ini_value "$config_home/gtk-$version.0/settings.ini" gtk-cursor-theme-size 24
        done

        if gsettings list-schemas 2>/dev/null | grep -qx org.gnome.desktop.interface; then
          gsettings set org.gnome.desktop.interface icon-theme Hatter-Slate >/dev/null 2>&1 || true
          gsettings set org.gnome.desktop.interface cursor-theme Bibata-Modern-Classic >/dev/null 2>&1 || true
          gsettings set org.gnome.desktop.interface cursor-size 24 >/dev/null 2>&1 || true
        fi
      }

      restore_noctalia() {
        strip_kde
        restore_noctalia_identity

        tries=0
        while [ "$tries" -lt 50 ]; do
          if [ -f "$config_home/gtk-3.0/noctalia.css" ] && [ -f "$config_home/gtk-4.0/noctalia.css" ]; then
            break
          fi
          tries=$((tries + 1))
          sleep 0.1
        done

        mode=$(current_mode)
        if [ "$mode" = light ]; then
          set_theme adw-gtk3 light
        else
          set_theme adw-gtk3-dark dark
        fi

        if [ -f "$config_home/gtk-3.0/noctalia.css" ] && [ -f "$config_home/gtk-4.0/noctalia.css" ]; then
          bash "$noctalia_apply" "$mode" >/dev/null 2>&1 || true
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
          restore_noctalia
          ;;

        noctalia-session)
          restore_noctalia
          ;;

        *)
          echo "Uso: korunix-gtk-session {plasma|noctalia|noctalia-session}" >&2
          exit 2
          ;;
      esac
    '';
  };

  noctaliaSession = pkgs.writeShellApplication {
    name = "korunix-noctalia-session";
    runtimeInputs = [gtkSession];
    text = ''
      korunix-gtk-session noctalia-session
      exec ${noctaliaPackage}/bin/noctalia "$@"
    '';
  };
in {
  environment.systemPackages = [
    pkgs.adw-gtk3
    gtkSession
    noctaliaSession
  ];

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

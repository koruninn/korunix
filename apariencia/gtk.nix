{
  config,
  equipo,
  pkgs,
  ...
}: let
  usuario = config.users.users.${equipo.persona};
  noctaliaPackage = config.programs.noctalia.package;

  noctaliaGtkHook = pkgs.writeText "noctalia-gtk-session.toml" ''
    [hooks]
    started = "korunix-gtk-session noctalia"
    colors_changed = "korunix-gtk-session noctalia"
  '';

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

      case "''${1:-}" in
        plasma)
          # Plasma conserva los archivos de Noctalia, pero desconecta su CSS y
          # deja que kde-gtk-config aplique Breeze para esta sesión.
          strip_noctalia
          set_theme Breeze dark
          ;;

        noctalia)
          # Abrir Noctalia manualmente dentro de Plasma no debe cambiar GTK.
          case "''${XDG_CURRENT_DESKTOP:-}" in
            *KDE*) exit 0 ;;
          esac

          # Al entrar a Niri/Umbriel restauramos primero la identidad GTK de la
          # sesión. Esto no toca Qt ni las plantillas de aplicaciones.
          strip_kde
          restore_noctalia_identity

          # La paleta GTK de Noctalia se conserva entre sesiones. Si ya existe,
          # la reconectamos de inmediato; cuando Noctalia arranque o cambie de
          # colores, sus plantillas GTK volverán a escribirla normalmente.
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
    runtimeInputs = [gtkSession];
    text = ''
      # La sesión Noctalia recupera GTK antes de iniciar el shell. Así una sesión
      # Plasma previa no puede dejar Breeze/Hatter desincronizados durante el
      # arranque de Niri o Umbriel.
      korunix-gtk-session noctalia
      exec ${noctaliaPackage}/bin/noctalia "$@"
    '';
  };
in {
  environment.systemPackages = [
    pkgs.adw-gtk3
    gtkSession
    noctaliaSession
  ];

  # Noctalia reafirma la propiedad GTK al arrancar y cuando cambia su paleta.
  # El arranque inicial también queda cubierto por korunix-noctalia-session.
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

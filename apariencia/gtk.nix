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

      strip_chromaleon() {
        local version dir file
        for version in 3 4; do
          dir="$config_home/gtk-$version.0"
          file="$dir/gtk.css"
          if [ -f "$file" ]; then
            sed -i '/\/\* CustomAccentExtension Start \*\//,/\/\* CustomAccentExtension End \*\//d' "$file"
          fi
          rm -f "$dir/custom-accent.css"
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

      case "''${1:-}" in
        plasma)
          # Plasma conserva los archivos generados por Noctalia para que volver
          # a Niri/Umbriel no requiera reconstruir el estado. Solo desconecta
          # sus imports y cualquier CSS que haya dejado ChromaLeon.
          strip_noctalia
          strip_chromaleon
          set_theme Breeze dark
          ;;

        gnome)
          # GNOME/ChromaLeon no debe heredar ni Noctalia ni la paleta CSS que
          # kde-gtk-config genera para Plasma.
          strip_noctalia
          strip_kde
          mode=$(current_mode)
          if [ "$mode" = light ]; then
            set_theme adw-gtk3 light
          else
            set_theme adw-gtk3-dark dark
          fi
          ;;

        noctalia)
          # El hook vive en la configuración de Noctalia, pero si alguien abre
          # el shell manualmente dentro de Plasma o GNOME no debe cambiar GTK.
          case "''${XDG_CURRENT_DESKTOP:-}" in
            *KDE*|*GNOME*) exit 0 ;;
          esac

          # Fuera de Plasma retiramos su colors.css y la capa de ChromaLeon.
          # Noctalia conserva noctalia.css entre sesiones; esperamos únicamente
          # el primer render de una instalación nueva antes de reactivar imports.
          strip_kde
          strip_chromaleon

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
          echo "Uso: korunix-gtk-session {plasma|gnome|noctalia}" >&2
          exit 2
          ;;
      esac
    '';
  };
in {
  # adw-gtk3 es la base que Noctalia usa para GTK 3. El conmutador conserva
  # una sola configuración de usuario y cambia únicamente la capa visual GTK
  # cuando empieza cada sesión.
  environment.systemPackages = [
    pkgs.adw-gtk3
    gtkSession
  ];

  # Noctalia ejecuta el cambio una vez que termina su arranque y sus plantillas
  # ya están disponibles. Así Niri y Umbriel comparten la misma ruta sin añadir
  # lógica específica a cada compositor.
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

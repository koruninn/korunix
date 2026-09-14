{
  config,
  equipo,
  pkgs,
  ...
}: let
  usuario = config.users.users.${equipo.persona};

  plasmaDynamicScheme = pkgs.runCommand "korunix-plasma-dynamic-colors" {} ''
    mkdir -p "$out/share/color-schemes"

    awk '
      /^\[General\]$/ {
        print
        print "TintFactor=0.15"
        next
      }
      /^ColorScheme=BreezeDark$/ {
        print "ColorScheme=KorunixDynamic"
        next
      }
      /^Name=Breeze Dark$/ {
        print "Name=Korunix Dynamic"
        next
      }
      /^Name\[/ {
        next
      }
      { print }
    ' ${pkgs.kdePackages.breeze}/share/color-schemes/BreezeDark.colors \
      > "$out/share/color-schemes/KorunixDynamic.colors"
  '';

  plasmaAppColors = pkgs.writeShellApplication {
    name = "korunix-plasma-app-colors";
    runtimeInputs = [
      pkgs.bash
      pkgs.coreutils
      pkgs.dasel
      pkgs.inotify-tools
      pkgs.jq
      pkgs.kdePackages.kconfig
      pkgs.matugen
    ];
    text = ''
      config_home="''${XDG_CONFIG_HOME:-$HOME/.config}"
      state_home="''${XDG_STATE_HOME:-$HOME/.local/state}"
      kdeglobals="$config_home/kdeglobals"
      noctalia_config="$config_home/noctalia/config.toml"
      community_root="$state_home/noctalia/community-templates"

      if [ ! -f "$noctalia_config" ] && [ -f /etc/noctalia/config.toml ]; then
        noctalia_config=/etc/noctalia/config.toml
      fi

      accent_hex() {
        local value r g b
        value="$(kreadconfig6 \
          --file "$kdeglobals" \
          --group General \
          --key AccentColor \
          2>/dev/null || true)"
        value="$(printf '%s' "$value" | tr -d '[:space:]')"

        if [[ "$value" =~ ^#[0-9A-Fa-f]{6}$ ]]; then
          printf '%s\n' "$value"
          return 0
        fi

        IFS=',' read -r r g b _ <<< "$value"
        if [[ "$r" =~ ^[0-9]+$ ]] \
          && [[ "$g" =~ ^[0-9]+$ ]] \
          && [[ "$b" =~ ^[0-9]+$ ]] \
          && ((r >= 0 && r <= 255)) \
          && ((g >= 0 && g <= 255)) \
          && ((b >= 0 && b <= 255)); then
          printf '#%02x%02x%02x\n' "$r" "$g" "$b"
          return 0
        fi

        return 1
      }

      render_template() {
        local id="$1"
        local accent="$2"
        local dir="$community_root/$id"
        local source="$dir/template.toml"
        local json rewritten config_file tmp name command output

        [ -f "$source" ] || return 0

        json="$(mktemp)"
        rewritten="$(mktemp)"

        if ! dasel -i toml -o json < "$source" > "$json" 2>/dev/null; then
          rm -f "$json" "$rewritten"
          return 0
        fi

        jq --arg config_dir "$dir" '
          def replace_config_dir:
            if type == "string" then
              gsub("\\{\\{ config_dir \\}\\}"; $config_dir)
            elif type == "array" then
              map(replace_config_dir)
            elif type == "object" then
              with_entries(.value |= replace_config_dir)
            else
              .
            end;

          replace_config_dir
          | del(.catalog)
          | .config = {
              version_check: false,
              caching: false
            }
        ' "$json" > "$rewritten"
        mv "$rewritten" "$json"

        # Noctalia admite rutas de salida calculadas por la propia plantilla.
        # Matugen no tiene ese campo, así que resolvemos el comando de manera
        # genérica y convertimos el resultado en output_path antes de renderizar.
        while IFS=$'\t' read -r name command; do
          [ -n "$name" ] || continue

          output="$(bash -c "$command" 2>/dev/null | tail -n 1)" || output=""
          tmp="$(mktemp)"

          if [ -n "$output" ]; then
            jq \
              --arg name "$name" \
              --arg output "$output" \
              '.templates[$name].output_path = $output
               | del(.templates[$name].output_path_dynamic)' \
              "$json" > "$tmp"
          else
            jq \
              --arg name "$name" \
              '.templates[$name].enabled = false
               | del(.templates[$name].output_path_dynamic)' \
              "$json" > "$tmp"
          fi

          mv "$tmp" "$json"
        done < <(
          jq -r '
            .templates // {}
            | to_entries[]
            | select(.value.output_path_dynamic? != null)
            | [.key, .value.output_path_dynamic]
            | @tsv
          ' "$json"
        )

        tmp="$(mktemp)"
        jq '
          .templates |= with_entries(
            .value |= del(.output_path_dynamic, .post_action, .hook_async)
          )
        ' "$json" > "$tmp"
        mv "$tmp" "$json"

        config_file="$(mktemp "$dir/.korunix-plasma.XXXXXX")"
        if dasel -i json -o toml < "$json" > "$config_file" 2>/dev/null; then
          matugen color hex "$accent" \
            --config "$config_file" \
            --mode dark \
            --type scheme-tonal-spot \
            --quiet \
            --continue-on-error \
            >/dev/null 2>&1 || true
        fi

        rm -f "$json" "$config_file"
      }

      apply_templates() {
        local accent="$1"
        local id

        [ -f "$noctalia_config" ] || return 0
        [ -d "$community_root" ] || return 0

        while IFS= read -r id; do
          [ -n "$id" ] || continue
          render_template "$id" "$accent"
        done < <(
          dasel -i toml -o json < "$noctalia_config" 2>/dev/null \
            | jq -r '.theme.templates.community_ids[]?' 2>/dev/null
        )
      }

      apply_current() {
        local accent
        accent="$(accent_hex)" || return 0
        apply_templates "$accent"
      }

      case "''${1:-apply}" in
        apply)
          apply_current
          ;;

        watch|--watch)
          last=""
          while :; do
            current="$(accent_hex 2>/dev/null || true)"
            if [ -n "$current" ] && [ "$current" != "$last" ]; then
              apply_templates "$current"
              last="$current"
            fi

            # Plasma actualiza AccentColor en kdeglobals cuando cambia el fondo.
            # Vigilamos el directorio porque KConfig puede reemplazar el archivo.
            inotifywait \
              -q \
              -e close_write,moved_to,create \
              "$config_home" \
              >/dev/null 2>&1 || sleep 1
          done
          ;;

        *)
          echo "Uso: korunix-plasma-app-colors {apply|--watch}" >&2
          exit 2
          ;;
      esac
    '';
  };

  plasmaNativeColors = pkgs.writeShellApplication {
    name = "korunix-plasma-colors";
    runtimeInputs = [
      pkgs.dbus
      pkgs.kdePackages.plasma-workspace
      plasmaAppColors
    ];
    text = ''
      # La frontera GTK se conmuta por sesión: Plasma desconecta la capa GTK de
      # Noctalia, pero deja sus archivos disponibles para restaurarlos al volver.
      korunix-gtk-session plasma

      # Plasma mantiene su motor nativo de acento derivado del fondo.
      plasma-apply-colorscheme KorunixDynamic >/dev/null 2>&1 || true

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

      # Las plantillas de aplicaciones que Noctalia dejó en el HOME se vuelven
      # a renderizar con una paleta M3 basada en el AccentColor de Plasma. El
      # mismo proceso queda atento a cambios del fondo durante toda la sesión.
      exec korunix-plasma-app-colors --watch
    '';
  };
in {
  services.desktopManager.plasma6.enable = true;

  environment.plasma6.excludePackages = [
    pkgs.kdePackages.kwin-x11
  ];

  # Plasma usa únicamente su motor nativo para el escritorio. Matugen se limita
  # a traducir ese mismo acento a la paleta M3 que necesitan las plantillas de
  # aplicaciones, sin sustituir el sistema de colores de Plasma.
  environment.systemPackages = [
    plasmaDynamicScheme
    plasmaAppColors
    plasmaNativeColors
  ];

  # Dejamos preparada la selección antes de que arranque Plasma para que su
  # servicio nativo de acento pueda reaccionar al fondo desde el inicio.
  system.activationScripts.plasmaNativeColors.text = ''
    install -d -m 0755 \
      -o ${usuario.name} \
      -g ${usuario.group} \
      ${usuario.home}/.config

    HOME=${usuario.home} \
    XDG_CONFIG_HOME=${usuario.home}/.config \
      ${pkgs.kdePackages.kconfig}/bin/kwriteconfig6 \
        --file kdeglobals \
        --group General \
        --key ColorScheme \
        KorunixDynamic

    HOME=${usuario.home} \
    XDG_CONFIG_HOME=${usuario.home}/.config \
      ${pkgs.kdePackages.kconfig}/bin/kwriteconfig6 \
        --file kdeglobals \
        --group General \
        --key accentColorFromWallpaper \
        true

    chown ${usuario.name}:${usuario.group} ${usuario.home}/.config/kdeglobals
  '';

  environment.etc."xdg/autostart/korunix-plasma-colors.desktop".text = ''
[Desktop Entry]
Type=Application
Name=Korunix Plasma Colors
Comment=Aplica los colores dinámicos del fondo a Plasma y sus aplicaciones
Exec=${plasmaNativeColors}/bin/korunix-plasma-colors
OnlyShowIn=KDE;
NoDisplay=true
X-KDE-AutostartScript=true
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

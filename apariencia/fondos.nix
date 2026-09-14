{
  config,
  pkgs,
  ...
}: let
  noctaliaPackage = config.programs.noctalia.package;
  fondosClaros = ./escritorios/noctalia/fondos/claro;
  fondosOscuros = ./escritorios/noctalia/fondos/oscuro;

  wallpaperSync = pkgs.writeShellApplication {
    name = "korunix-wallpaper-sync";
    runtimeInputs = with pkgs; [
      bash
      coreutils
      findutils
      glib
      jq
      python3
      util-linux
    ];
    text = ''
      config_home="''${XDG_CONFIG_HOME:-$HOME/.config}"
      state_home="''${XDG_STATE_HOME:-$HOME/.local/state}"
      state_dir="$state_home/korunix"
      current_file="$state_dir/wallpaper-current"
      palette_file="$state_dir/palette.json"
      rotation_file="$state_dir/wallpaper-last-rotation"
      log_file="$state_dir/wallpaper.log"
      noctalia_config="$config_home/noctalia/config.toml"
      noctalia_settings="$state_home/noctalia/settings.toml"
      community_root="$state_home/noctalia/community-templates"
      template_root="${noctaliaPackage}/share/noctalia/assets/templates"

      mkdir -p "$state_dir"

      log() {
        printf '%s %s\n' "$(date -Is)" "$*" >> "$log_file"
      }

      noctalia_running() {
        ${noctaliaPackage}/bin/noctalia msg theme-mode-get >/dev/null 2>&1
      }

      current_mode() {
        local mode scheme

        if mode="$(${noctaliaPackage}/bin/noctalia msg theme-mode-get 2>/dev/null)"; then
          case "$mode" in
            dark|light)
              printf '%s\n' "$mode"
              return 0
              ;;
          esac
        fi

        scheme="$(gsettings get org.gnome.desktop.interface color-scheme 2>/dev/null || true)"
        case "$scheme" in
          *dark*) printf '%s\n' dark ;;
          *) printf '%s\n' light ;;
        esac
      }

      set_gnome_mode() {
        local mode="$1"
        if [ "$mode" = dark ]; then
          gsettings set org.gnome.desktop.interface color-scheme prefer-dark >/dev/null 2>&1 || true
        else
          gsettings set org.gnome.desktop.interface color-scheme prefer-light >/dev/null 2>&1 || \
            gsettings set org.gnome.desktop.interface color-scheme default >/dev/null 2>&1 || true
        fi
      }

      path_to_uri() {
        python3 - "$1" <<'PY'
from pathlib import Path
import sys
print(Path(sys.argv[1]).expanduser().resolve().as_uri())
PY
      }

      uri_to_path() {
        python3 - "$1" <<'PY'
import sys
from urllib.parse import unquote, urlparse
value = sys.argv[1].strip().strip("'").strip('"')
if value.startswith("file://"):
    print(unquote(urlparse(value).path))
else:
    print(value)
PY
      }

      set_gnome_wallpaper() {
        local path="$1" uri
        uri="$(path_to_uri "$path")"
        gsettings set org.gnome.desktop.background picture-uri "$uri" >/dev/null 2>&1 || true
        gsettings set org.gnome.desktop.background picture-uri-dark "$uri" >/dev/null 2>&1 || true
      }

      set_identity() {
        gsettings set org.gnome.desktop.interface icon-theme Hatter-Slate >/dev/null 2>&1 || true
        gsettings set org.gnome.desktop.interface cursor-theme Bibata-Modern-Classic >/dev/null 2>&1 || true
        gsettings set org.gnome.desktop.interface cursor-size 24 >/dev/null 2>&1 || true
      }

      read_runtime_config() {
        python3 - \
          "$noctalia_config" \
          "$noctalia_settings" \
          "${fondosClaros}" \
          "${fondosOscuros}" <<'PY'
import pathlib
import sys
import tomllib

config_path, settings_path, default_light, default_dark = sys.argv[1:]
wallpaper = {
    "directory_light": default_light,
    "directory_dark": default_dark,
}
automation = {
    "enabled": False,
    "interval_seconds": 1800,
    "order": "random",
    "recursive": True,
}

for raw_path in (config_path, settings_path):
    path = pathlib.Path(raw_path)
    if not path.is_file():
        continue
    try:
        data = tomllib.loads(path.read_text())
    except Exception:
        continue
    section = data.get("wallpaper", {})
    for key in ("directory_light", "directory_dark"):
        value = section.get(key)
        if isinstance(value, str) and value:
            wallpaper[key] = value
    auto = section.get("automation", {})
    if isinstance(auto, dict):
        automation.update({k: v for k, v in auto.items() if k in automation})

print(
    "\t".join(
        [
            "true" if bool(automation["enabled"]) else "false",
            str(int(automation["interval_seconds"])),
            str(automation["order"]),
            "true" if bool(automation["recursive"]) else "false",
            wallpaper["directory_light"],
            wallpaper["directory_dark"],
        ]
    )
)
PY
      }

      load_runtime_config() {
        IFS=$'\t' read -r \
          automation_enabled \
          automation_interval \
          automation_order \
          automation_recursive \
          light_dir \
          dark_dir \
          < <(read_runtime_config)

        if ! [[ "$automation_interval" =~ ^[0-9]+$ ]]; then
          automation_interval=1800
        fi
        [ "$automation_interval" -ge 60 ] || automation_interval=60

        case "$automation_order" in
          alphabetical|random) ;;
          *) automation_order=random ;;
        esac
      }

      list_wallpapers() {
        local directory="$1" recursive="$2"
        [ -d "$directory" ] || return 0

        if [ "$recursive" = true ]; then
          find "$directory" -type f \
            \( -iname '*.jpg' -o -iname '*.jpeg' -o -iname '*.png' -o -iname '*.webp' -o -iname '*.avif' \) \
            -print
        else
          find "$directory" -maxdepth 1 -type f \
            \( -iname '*.jpg' -o -iname '*.jpeg' -o -iname '*.png' -o -iname '*.webp' -o -iname '*.avif' \) \
            -print
        fi
      }

      pick_wallpaper() {
        local mode="$1" order="''${2:-random}" directory current next_index i
        local -a files

        load_runtime_config
        if [ "$mode" = dark ]; then
          directory="$dark_dir"
        else
          directory="$light_dir"
        fi

        mapfile -t files < <(list_wallpapers "$directory" "$automation_recursive" | sort -f)
        [ "''${#files[@]}" -gt 0 ] || return 1

        if [ "$order" = random ]; then
          printf '%s\n' "''${files[@]}" | shuf -n 1
          return 0
        fi

        current="$(cat "$current_file" 2>/dev/null || true)"
        next_index=0
        for i in "''${!files[@]}"; do
          if [ "''${files[$i]}" = "$current" ]; then
            next_index=$(( (i + 1) % ''${#files[@]} ))
            break
          fi
        done
        printf '%s\n' "''${files[$next_index]}"
      }

      template_ids() {
        python3 - "$noctalia_config" <<'PY'
import pathlib
import sys
import tomllib

path = pathlib.Path(sys.argv[1])
if not path.is_file():
    raise SystemExit(0)
try:
    data = tomllib.loads(path.read_text())
except Exception:
    raise SystemExit(0)
for item in data.get("theme", {}).get("templates", {}).get("community_ids", []):
    if isinstance(item, str) and item:
        print(item)
PY
      }

      render_gnome_palette() {
        local wallpaper="$1" mode tmp_palette output id template_config
        local applied=0 failed=0 missing=0

        mode="$(current_mode)"
        mkdir -p \
          "$config_home/gtk-3.0" \
          "$config_home/gtk-4.0" \
          "$config_home/qt5ct/colors" \
          "$config_home/qt6ct/colors" \
          "$config_home/alacritty/themes"

        tmp_palette="$(mktemp)"
        if ! ${noctaliaPackage}/bin/noctalia theme "$wallpaper" \
          --scheme m3-tonal-spot \
          --both \
          -o "$tmp_palette" >/dev/null 2>&1; then
          rm -f "$tmp_palette"
          log "ERROR generando paleta M3 para $wallpaper"
          return 1
        fi

        install -m 0644 "$tmp_palette" "$palette_file"
        rm -f "$tmp_palette"

        if ! ${noctaliaPackage}/bin/noctalia theme \
          --theme-json "$palette_file" \
          --default-mode "$mode" \
          -r "$template_root/gtk/gtk3.css:$config_home/gtk-3.0/noctalia.css" \
          -r "$template_root/gtk/gtk4.css:$config_home/gtk-4.0/noctalia.css" \
          -r "$template_root/qt/qtct.conf:$config_home/qt5ct/colors/noctalia.conf" \
          -r "$template_root/qt/qtct.conf:$config_home/qt6ct/colors/noctalia.conf" \
          -r "$template_root/alacritty/alacritty.toml:$config_home/alacritty/themes/noctalia.toml" \
          >/dev/null 2>&1; then
          log "ERROR renderizando plantillas base para GNOME"
          return 1
        fi

        if command -v korunix-gtk-session >/dev/null 2>&1; then
          korunix-gtk-session noctalia-session >/dev/null 2>&1 || true
        else
          bash "$template_root/gtk/apply.sh" "$mode" >/dev/null 2>&1 || true
        fi
        bash "$template_root/alacritty/apply.sh" >/dev/null 2>&1 || true
        set_identity

        if [ -d "$community_root" ]; then
          while IFS= read -r id; do
            [ -n "$id" ] || continue
            template_config="$community_root/$id/template.toml"
            if [ ! -f "$template_config" ]; then
              missing=$((missing + 1))
              log "Plantilla ausente en GNOME: $id"
              continue
            fi

            output=""
            if output="$(${noctaliaPackage}/bin/noctalia theme "$wallpaper" \
              --theme-json "$palette_file" \
              --scheme m3-tonal-spot \
              --default-mode "$mode" \
              --config "$template_config" 2>&1)"; then
              applied=$((applied + 1))
            else
              failed=$((failed + 1))
              log "ERROR plantilla GNOME $id: $output"
            fi
          done < <(template_ids)
        fi

        if command -v korunix-gnome-shell-palette >/dev/null 2>&1; then
          korunix-gnome-shell-palette "$palette_file" "$mode" >/dev/null 2>&1 || true
        fi

        log "Paleta GNOME aplicada: modo=$mode aplicadas=$applied fallidas=$failed ausentes=$missing fondo=$wallpaper"
        [ "$failed" -eq 0 ]
      }

      apply_path() {
        local path="$1" origin="''${2:-korunix}" mode
        [ -f "$path" ] || {
          log "Fondo inválido: $path"
          return 1
        }

        printf '%s\n' "$path" > "$current_file"
        set_gnome_wallpaper "$path"

        if noctalia_running; then
          mode="$(current_mode)"
          set_gnome_mode "$mode"
          if [ "$origin" != noctalia ]; then
            ${noctaliaPackage}/bin/noctalia msg wallpaper-set "$path" >/dev/null 2>&1 || true
          fi
          log "Fondo sincronizado con Noctalia: modo=$mode fondo=$path"
        else
          render_gnome_palette "$path"
        fi
      }

      from_noctalia() {
        local path="$1" mode
        [ -f "$path" ] || return 0
        mode="$(current_mode)"
        printf '%s\n' "$path" > "$current_file"
        set_gnome_mode "$mode"
        set_gnome_wallpaper "$path"
        set_identity
        log "Noctalia → estado compartido: modo=$mode fondo=$path"
      }

      from_gnome() {
        local path="$1" current
        [ -f "$path" ] || return 0
        current="$(cat "$current_file" 2>/dev/null || true)"
        [ "$path" != "$current" ] || return 0
        apply_path "$path" gnome
        log "GNOME → estado compartido: fondo=$path"
      }

      apply_current() {
        local path mode
        path="$(cat "$current_file" 2>/dev/null || true)"
        if [ -n "$path" ] && [ -f "$path" ]; then
          apply_path "$path" session
          return
        fi

        mode="$(current_mode)"
        path="$(pick_wallpaper "$mode" random)" || return 1
        apply_path "$path" session
      }

      rotate() {
        local mode path
        load_runtime_config
        mode="$(current_mode)"
        path="$(pick_wallpaper "$mode" "$automation_order")" || return 1
        apply_path "$path" rotation
        date +%s > "$rotation_file"
      }

      mode_change() {
        local mode path
        mode="$(current_mode)"
        set_gnome_mode "$mode"
        path="$(pick_wallpaper "$mode" random)" || return 1
        apply_path "$path" mode-change
      }

      effective_gnome_wallpaper() {
        local mode uri
        mode="$(current_mode)"
        if [ "$mode" = dark ]; then
          uri="$(gsettings get org.gnome.desktop.background picture-uri-dark 2>/dev/null || true)"
        else
          uri="$(gsettings get org.gnome.desktop.background picture-uri 2>/dev/null || true)"
        fi
        uri_to_path "$uri"
      }

      monitor_gnome_background() {
        gsettings monitor org.gnome.desktop.background 2>/dev/null |
          while IFS= read -r _line; do
            path="$(effective_gnome_wallpaper)"
            [ -n "$path" ] || continue
            from_gnome "$path" || true
          done
      }

      monitor_gnome_mode() {
        gsettings monitor org.gnome.desktop.interface color-scheme 2>/dev/null |
          while IFS= read -r _line; do
            if noctalia_running; then
              continue
            fi
            mode_change || true
          done
      }

      daemon() {
        local bg_pid mode_pid now last

        exec 9>"$state_dir/wallpaper-daemon.lock"
        flock -n 9 || exit 0

        apply_current || true

        monitor_gnome_background &
        bg_pid=$!
        monitor_gnome_mode &
        mode_pid=$!

        cleanup() {
          kill "$bg_pid" "$mode_pid" >/dev/null 2>&1 || true
        }
        trap cleanup EXIT INT TERM HUP

        while sleep 30; do
          load_runtime_config
          [ "$automation_enabled" = true ] || continue

          now="$(date +%s)"
          last="$(cat "$rotation_file" 2>/dev/null || printf '0')"
          if ! [[ "$last" =~ ^[0-9]+$ ]]; then
            last=0
          fi

          if [ $((now - last)) -ge "$automation_interval" ]; then
            rotate || true
          fi
        done
      }

      case "''${1:-}" in
        daemon)
          daemon
          ;;
        apply|session-start)
          apply_current
          ;;
        random)
          mode="$(current_mode)"
          path="$(pick_wallpaper "$mode" random)"
          apply_path "$path" random
          ;;
        rotate)
          rotate
          ;;
        mode-change)
          mode_change
          ;;
        from-noctalia)
          from_noctalia "''${2:-}"
          ;;
        from-gnome)
          from_gnome "''${2:-}"
          ;;
        *)
          echo "Uso: korunix-wallpaper-sync {daemon|apply|random|rotate|mode-change|from-noctalia <ruta>|from-gnome <ruta>}" >&2
          exit 2
          ;;
      esac
    '';
  };
in {
  environment.systemPackages = [wallpaperSync];

  environment.etc."xdg/autostart/korunix-wallpaper-sync.desktop".text = ''
[Desktop Entry]
Type=Application
Name=Korunix · Fondos sincronizados
Comment=Mantiene fondo, modo y paleta sincronizados entre GNOME, Niri y Umbriel
Exec=${wallpaperSync}/bin/korunix-wallpaper-sync daemon
NoDisplay=true
X-GNOME-Autostart-enabled=true
  '';
}

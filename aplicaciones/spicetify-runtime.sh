set -euo pipefail

mode=${1:-}
if [[ $mode != --prepare && $mode != --launch ]]; then
  echo "Uso: korunix-spotify-runtime --prepare | --launch [argumentos]" >&2
  exit 2
fi
shift

config_dir=${XDG_CONFIG_HOME:-$HOME/.config}/spicetify
data_dir=${XDG_DATA_HOME:-$HOME/.local/share}/korunix/spotify
state_dir=${XDG_STATE_HOME:-$HOME/.local/state}/korunix
prefs_file=${XDG_CONFIG_HOME:-$HOME/.config}/spotify/prefs
marker=$state_dir/spotify-source
export SPICETIFY_CONFIG=$config_dir

mkdir -p "$state_dir" "$config_dir/Themes/Comfy" "$config_dir/Extensions" "$(dirname "$prefs_file")" "$(dirname "$data_dir")"
exec 9>"$state_dir/spotify.lock"
flock 9

# No borrar los recursos de una versión anterior mientras Spotify los use.
for old in "$(dirname "$data_dir")"/.spotify-previous-*; do
  if [[ -d $old ]] && ! fuser -s "$old/.spotify-wrapped" 2>/dev/null; then
    rm -rf -- "$old"
  fi
done

version="$KORUNIX_SPOTIFY_SOURCE
$KORUNIX_COMFY_SOURCE
$KORUNIX_ADBLOCK_SOURCE
$KORUNIX_LYRICS_SOURCE
$KORUNIX_ONEKO_SOURCE"

if [[ ! -f $marker || $(cat "$marker") != "$version" || ! -x $data_dir/spotify ]]; then
  stage=$(mktemp -d "$(dirname "$data_dir")/.spotify-stage.XXXXXXXX")
  previous="$(dirname "$data_dir")/.spotify-previous-$$"
  trap 'rm -rf -- "$stage"' EXIT

  # Mantener el entorno del wrapper de Nix, pero abrir los recursos del usuario.
  cp -a --reflink=auto "$KORUNIX_SPOTIFY_SOURCE/." "$stage/"
  chmod -R u+w "$stage"
  original="$KORUNIX_SPOTIFY_SOURCE/.spotify-wrapped"
  if ! grep -Fq -- "$original" "$stage/spotify"; then
    echo "El wrapper de Spotify cambió; no se reemplazará la instalación anterior." >&2
    exit 1
  fi
  replacement=${data_dir//\\/\\\\}
  replacement=${replacement//&/\\&}
  replacement=${replacement//|/\\|}
  sed "s|$original|$replacement/.spotify-wrapped|g" "$stage/spotify" > "$stage/spotify.korunix"
  chmod --reference="$stage/spotify" "$stage/spotify.korunix"
  mv "$stage/spotify.korunix" "$stage/spotify"

  # Conservar los colores de Noctalia al actualizar la estructura de Comfy.
  rsync -rL --exclude=/color.ini "$KORUNIX_COMFY_SOURCE/" "$config_dir/Themes/Comfy/"
  chmod -R u+w "$config_dir/Themes/Comfy"
  if [[ ! -f $config_dir/Themes/Comfy/color.ini ]]; then
    cp "$KORUNIX_COMFY_SOURCE/color.ini" "$config_dir/Themes/Comfy/color.ini"
    chmod u+w "$config_dir/Themes/Comfy/color.ini"
  fi
  cp -L "$KORUNIX_ADBLOCK_SOURCE" "$config_dir/Extensions/adblock.js"
  cp -L "$KORUNIX_LYRICS_SOURCE" "$config_dir/Extensions/spicy-lyrics.mjs"
  cp -L "$KORUNIX_ONEKO_SOURCE" "$config_dir/Extensions/oneko.js"
  cat "$KORUNIX_COMFY_SOURCE/theme.js" "$KORUNIX_COMFY_SOURCE/theme.script.js" > "$config_dir/Extensions/theme.js"
  chmod u+w "$config_dir/Extensions/"{adblock.js,spicy-lyrics.mjs,oneko.js,theme.js}

  touch "$prefs_file"
  if [[ -f $config_dir/config-xpui.ini && ! -e $config_dir/config-xpui.ini.korunix-backup ]]; then
    cp "$config_dir/config-xpui.ini" "$config_dir/config-xpui.ini.korunix-backup"
  fi
  cat > "$config_dir/config-xpui.ini" <<EOF
[Setting]
spotify_path = $data_dir
prefs_path = $prefs_file
current_theme = Comfy
color_scheme = Comfy
inject_css = 1
replace_colors = 1
overwrite_assets = 1
inject_theme_js = 1
check_spicetify_update = 0

[AdditionalOptions]
extensions = adblock.js|spicy-lyrics.mjs|oneko.js|theme.js
home_config = 1
sidebar_config = 0

[Preprocesses]
disable_ui_logging = 1
remove_rtl_rule = 1
expose_apis = 1
disable_sentry = 1
EOF

  if [[ -e $data_dir ]]; then
    mv "$data_dir" "$previous"
  fi
  mv "$stage" "$data_dir"
  trap - EXIT
  if ! "$KORUNIX_SPICETIFY_CLI" --no-restart backup apply; then
    rm -rf -- "$data_dir"
    if [[ -e $previous ]]; then mv "$previous" "$data_dir"; fi
    echo "No se pudo aplicar Spicetify; se conservó el Spotify anterior." >&2
    exit 1
  fi
  printf '%s\n' "$version" > "$marker"
fi

flock -u 9
if [[ $mode == --launch ]]; then
  exec "$data_dir/spotify" "$@"
fi

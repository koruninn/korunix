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
theme_dir=$config_dir/Themes/$KORUNIX_THEME_NAME
palette_file=$config_dir/Themes/Comfy/color.ini
export SPICETIFY_CONFIG=$config_dir

mkdir -p "$state_dir" "$config_dir/Themes/Comfy" "$theme_dir" "$config_dir/Extensions" "$(dirname "$prefs_file")" "$(dirname "$data_dir")"
exec 9>"$state_dir/spotify.lock"
flock 9

# No borrar los recursos de una versión anterior mientras Spotify los use.
for old in "$(dirname "$data_dir")"/.spotify-previous-*; do
  if [[ -d $old ]] && ! fuser -s "$old/.spotify-wrapped" 2>/dev/null; then
    rm -rf -- "$old"
  fi
done

version="$KORUNIX_SPOTIFY_SOURCE
$KORUNIX_THEME_NAME
$KORUNIX_THEME_SOURCE
$KORUNIX_THEME_INJECT_CSS
$KORUNIX_THEME_INJECT_JS
$KORUNIX_THEME_REPLACE_COLORS
$KORUNIX_THEME_OVERWRITE_ASSETS
$KORUNIX_THEME_HOME_CONFIG
$KORUNIX_THEME_EXPERIMENTAL_FEATURES
$KORUNIX_THEME_EXTENSIONS
$KORUNIX_THEME_PATCHES
$KORUNIX_THEME_SETUP
$KORUNIX_THEME_ADDITIONAL_CSS
$KORUNIX_PALETTE_FALLBACK"

needs_apply=0
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

  # Instalar la estructura y los complementos declarados por el tema elegido.
  rsync -rL --exclude=/color.ini "$KORUNIX_THEME_SOURCE/" "$theme_dir/"
  if [[ -s $KORUNIX_THEME_ADDITIONAL_CSS ]]; then
    cat "$KORUNIX_THEME_ADDITIONAL_CSS" >> "$theme_dir/user.css"
  fi

  extension_names=
  while IFS=$'\t' read -r source name; do
    [[ -n $source && -n $name ]] || continue
    rm -rf -- "$config_dir/Extensions/$name"
    cp -rL -- "$source" "$config_dir/Extensions/$name"
    if [[ -z $extension_names ]]; then
      extension_names=$name
    else
      extension_names="$extension_names|$name"
    fi
  done < "$KORUNIX_THEME_EXTENSIONS"
  chmod -R u+w "$config_dir/Extensions"
  (cd "$config_dir" && "$KORUNIX_THEME_SETUP")

  touch "$prefs_file"
  if [[ -f $config_dir/config-xpui.ini && ! -e $config_dir/config-xpui.ini.korunix-backup ]]; then
    cp "$config_dir/config-xpui.ini" "$config_dir/config-xpui.ini.korunix-backup"
  fi
  cat > "$config_dir/config-xpui.ini" <<EOF
[Setting]
spotify_path = $data_dir
prefs_path = $prefs_file
current_theme = $KORUNIX_THEME_NAME
color_scheme = Comfy
inject_css = $KORUNIX_THEME_INJECT_CSS
replace_colors = $KORUNIX_THEME_REPLACE_COLORS
overwrite_assets = $KORUNIX_THEME_OVERWRITE_ASSETS
inject_theme_js = $KORUNIX_THEME_INJECT_JS
check_spicetify_update = 0

[AdditionalOptions]
extensions = $extension_names
home_config = $KORUNIX_THEME_HOME_CONFIG
sidebar_config = 0
experimental_features = $KORUNIX_THEME_EXPERIMENTAL_FEATURES

[Preprocesses]
disable_ui_logging = 1
remove_rtl_rule = 1
expose_apis = 1
disable_sentry = 1
EOF
  cat "$KORUNIX_THEME_PATCHES" >> "$config_dir/config-xpui.ini"

  if [[ -e $data_dir ]]; then
    mv "$data_dir" "$previous"
  fi
  mv "$stage" "$data_dir"
  trap - EXIT
  needs_apply=1
fi

# Combinar el esquema original del tema con la paleta M3 que genera Noctalia.
# Se mantiene el nombre Comfy porque la plantilla escribe esa sección.
if [[ ! -f $palette_file ]]; then
  sed 's/^\[Ocean\]$/[Comfy]/' "$KORUNIX_PALETTE_FALLBACK/color.ini" > "$palette_file"
fi
theme_colors=$(mktemp "$state_dir/theme-colors.XXXXXXXX")
if [[ -f $KORUNIX_THEME_SOURCE/color.ini ]]; then
  awk '
    /^\[Comfy\]$/ { skipping = 1; next }
    skipping && /^\[[^]]+\]$/ { skipping = 0 }
    !skipping { print }
  ' "$KORUNIX_THEME_SOURCE/color.ini" > "$theme_colors"
fi
printf '\n' >> "$theme_colors"
cat "$palette_file" >> "$theme_colors"
chmod u+w "$theme_colors"
mv -f -- "$theme_colors" "$theme_dir/color.ini"

if (( needs_apply )); then
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

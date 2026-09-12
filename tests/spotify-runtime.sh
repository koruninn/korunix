#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
tmp=$(mktemp -d)
trap 'rm -rf -- "$tmp"' EXIT
export HOME=$tmp/home
export XDG_CONFIG_HOME=$HOME/.config
export XDG_DATA_HOME=$HOME/.local/share
export XDG_STATE_HOME=$HOME/.local/state
mkdir -p "$HOME" "$tmp/source/share/spotify/Apps" "$tmp/comfy" "$tmp/ext"

source_dir=$tmp/source/share/spotify
cat > "$source_dir/spotify" <<EOF
#!/usr/bin/env bash
exec "$source_dir/.spotify-wrapped" "\$@"
EOF
cat > "$source_dir/.spotify-wrapped" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "$*" > "$HOME/launched"
EOF
chmod +x "$source_dir/spotify" "$source_dir/.spotify-wrapped"
printf 'theme\n' > "$tmp/comfy/color.ini"
printf 'css\n' > "$tmp/comfy/user.css"
printf 'js\n' > "$tmp/comfy/theme.js"
printf 'script\n' > "$tmp/comfy/theme.script.js"
for extension in adblock.js spicy-lyrics.mjs oneko.js; do
  printf 'extension\n' > "$tmp/ext/$extension"
done
cat > "$tmp/cli" <<'EOF'
#!/usr/bin/env bash
[[ $* == '--no-restart backup apply' ]]
[[ -f $SPICETIFY_CONFIG/Themes/Comfy/color.ini ]]
spotify_path=$(sed -n 's/^spotify_path = //p' "$SPICETIFY_CONFIG/config-xpui.ini")
[[ -x $spotify_path/spotify ]]
[[ -d $spotify_path/Apps ]]
[[ -f $SPICETIFY_CONFIG/Extensions/theme.js ]]
if [[ -e $HOME/fail-apply ]]; then exit 1; fi
printf 'apply\n' >> "$HOME/applies"
EOF
chmod +x "$tmp/cli"

export KORUNIX_SPOTIFY_SOURCE=$source_dir
export KORUNIX_COMFY_SOURCE=$tmp/comfy
export KORUNIX_ADBLOCK_SOURCE=$tmp/ext/adblock.js
export KORUNIX_LYRICS_SOURCE=$tmp/ext/spicy-lyrics.mjs
export KORUNIX_ONEKO_SOURCE=$tmp/ext/oneko.js
export KORUNIX_SPICETIFY_CLI=$tmp/cli

bash "$root/aplicaciones/spicetify-runtime.sh" --prepare
[[ $(wc -l < "$HOME/applies") == 1 ]]
bash "$root/aplicaciones/spicetify-runtime.sh" --launch spotify:album:abc
[[ $(cat "$HOME/launched") == 'spotify:album:abc' ]]

printf 'noctalia-m3-tonal-spot\n' > "$HOME/.config/spicetify/Themes/Comfy/color.ini"
bash "$root/aplicaciones/spicetify-runtime.sh" --prepare
[[ $(wc -l < "$HOME/applies") == 1 ]]

# Una actualización renueva la estructura de Comfy sin pisar la paleta.
mkdir -p "$tmp/comfy2"
cp -a "$tmp/comfy/." "$tmp/comfy2/"
printf 'new-css\n' > "$tmp/comfy2/user.css"
export KORUNIX_COMFY_SOURCE=$tmp/comfy2
bash "$root/aplicaciones/spicetify-runtime.sh" --prepare
[[ $(cat "$HOME/.config/spicetify/Themes/Comfy/color.ini") == 'noctalia-m3-tonal-spot' ]]
[[ $(cat "$HOME/.config/spicetify/Themes/Comfy/user.css") == 'new-css' ]]
[[ $(wc -l < "$HOME/applies") == 2 ]]

# Si falla el parcheado de una actualización, el Spotify anterior sigue ahí.
mkdir -p "$tmp/comfy3"
cp -a "$tmp/comfy2/." "$tmp/comfy3/"
export KORUNIX_COMFY_SOURCE=$tmp/comfy3
touch "$HOME/fail-apply"
if bash "$root/aplicaciones/spicetify-runtime.sh" --prepare; then
  echo 'Se esperaba un fallo al aplicar Spicetify.' >&2
  exit 1
fi
[[ -x $HOME/.local/share/korunix/spotify/spotify ]]
[[ $(wc -l < "$HOME/applies") == 2 ]]
echo 'Integración de Spotify: OK'

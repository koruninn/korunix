#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
tmp=$(mktemp -d)
trap 'rm -rf -- "$tmp"' EXIT
export HOME=$tmp/home
export XDG_CONFIG_HOME=$HOME/.config
export XDG_DATA_HOME=$HOME/.local/share
export XDG_STATE_HOME=$HOME/.local/state
mkdir -p "$HOME" "$tmp/source/share/spotify/Apps" "$tmp/default" "$tmp/ext"

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
printf '[Base]\n\n[Ocean]\nmain = 0F111A\n' > "$tmp/default/color.ini"
for extension in adblock.js spicy-lyrics.mjs oneko.js; do
  printf 'extension\n' > "$tmp/ext/$extension"
done
cat > "$tmp/cli" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
[[ $* == '--no-restart backup apply' ]]
[[ -L $SPICETIFY_CONFIG/Themes/Default/color.ini ]]
[[ $(readlink "$SPICETIFY_CONFIG/Themes/Default/color.ini") == ../Comfy/color.ini ]]
[[ -f $SPICETIFY_CONFIG/Themes/Default/color.ini ]]
grep -q '^current_theme = Default$' "$SPICETIFY_CONFIG/config-xpui.ini"
grep -q '^color_scheme = Comfy$' "$SPICETIFY_CONFIG/config-xpui.ini"
grep -q '^inject_css = 0$' "$SPICETIFY_CONFIG/config-xpui.ini"
grep -q '^inject_theme_js = 0$' "$SPICETIFY_CONFIG/config-xpui.ini"
spotify_path=$(sed -n 's/^spotify_path = //p' "$SPICETIFY_CONFIG/config-xpui.ini")
[[ -x $spotify_path/spotify ]]
[[ -d $spotify_path/Apps ]]
! grep -q '^extensions = .*theme.js' "$SPICETIFY_CONFIG/config-xpui.ini"
if [[ -e $HOME/fail-apply ]]; then exit 1; fi
printf 'apply\n' >> "$HOME/applies"
EOF
chmod +x "$tmp/cli"

export KORUNIX_SPOTIFY_SOURCE=$source_dir
export KORUNIX_DEFAULT_SOURCE=$tmp/default
export KORUNIX_ADBLOCK_SOURCE=$tmp/ext/adblock.js
export KORUNIX_LYRICS_SOURCE=$tmp/ext/spicy-lyrics.mjs
export KORUNIX_ONEKO_SOURCE=$tmp/ext/oneko.js
export KORUNIX_SPICETIFY_CLI=$tmp/cli

bash "$root/aplicaciones/spicetify-runtime.sh" --prepare
[[ $(wc -l < "$HOME/applies") == 1 ]]
grep -q '^\[Comfy\]$' "$HOME/.config/spicetify/Themes/Default/color.ini"
bash "$root/aplicaciones/spicetify-runtime.sh" --launch spotify:album:abc
[[ $(cat "$HOME/launched") == 'spotify:album:abc' ]]

printf '[Comfy]\nmain = ABCDEF\n' > "$HOME/.config/spicetify/Themes/Comfy/color.ini"
printf 'personal\n' > "$HOME/.config/spicetify/Themes/Comfy/personal.css"
bash "$root/aplicaciones/spicetify-runtime.sh" --prepare
[[ $(wc -l < "$HOME/applies") == 1 ]]

# Una actualización de Default conserva los colores de Noctalia.
mkdir -p "$tmp/default2"
cp -a "$tmp/default/." "$tmp/default2/"
printf '[Base]\n\n[Ocean]\nmain = FFFFFF\n' > "$tmp/default2/color.ini"
export KORUNIX_DEFAULT_SOURCE=$tmp/default2
bash "$root/aplicaciones/spicetify-runtime.sh" --prepare
grep -q '^main = ABCDEF$' "$HOME/.config/spicetify/Themes/Default/color.ini"
[[ $(cat "$HOME/.config/spicetify/Themes/Comfy/personal.css") == 'personal' ]]
[[ $(readlink "$HOME/.config/spicetify/Themes/Default/color.ini") == ../Comfy/color.ini ]]
[[ $(wc -l < "$HOME/applies") == 2 ]]

# Si falla el parcheado de una actualización, el Spotify anterior sigue ahí.
mkdir -p "$tmp/default3"
cp -a "$tmp/default2/." "$tmp/default3/"
export KORUNIX_DEFAULT_SOURCE=$tmp/default3
touch "$HOME/fail-apply"
if bash "$root/aplicaciones/spicetify-runtime.sh" --prepare; then
  echo 'Se esperaba un fallo al aplicar Spicetify.' >&2
  exit 1
fi
[[ -x $HOME/.local/share/korunix/spotify/spotify ]]
[[ $(wc -l < "$HOME/applies") == 2 ]]
echo 'Integración de Spotify: OK'

#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
tmp=$(mktemp -d)
trap 'rm -rf -- "$tmp"' EXIT
export HOME=$tmp/home
export XDG_CONFIG_HOME=$HOME/.config
export XDG_DATA_HOME=$HOME/.local/share
export XDG_STATE_HOME=$HOME/.local/state
mkdir -p "$HOME" "$tmp/source/share/spotify/Apps" "$tmp/theme" "$tmp/fallback" "$tmp/ext"

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
printf '[Text]\naccent = 123456\n' > "$tmp/theme/color.ini"
printf 'text-css\n' > "$tmp/theme/user.css"
printf '[Base]\n\n[Ocean]\nmain = 0F111A\n' > "$tmp/fallback/color.ini"
for extension in adblock.js spicy-lyrics.mjs oneko.js text-extra.js; do
  printf 'extension\n' > "$tmp/ext/$extension"
done
cat > "$tmp/extensions" <<EOF
$tmp/ext/adblock.js	adblock.js
$tmp/ext/spicy-lyrics.mjs	spicy-lyrics.mjs
$tmp/ext/oneko.js	oneko.js
$tmp/ext/text-extra.js	text-extra.js
EOF
cat > "$tmp/patches" <<'EOF'

[Patch]
xpui.js_find_8008 = ,(\w+=)56
xpui.js_repl_8008 = ,${1}32
EOF
cat > "$tmp/setup" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
touch Extensions/theme-setup-ran
EOF
chmod +x "$tmp/setup"
: > "$tmp/additional.css"

cat > "$tmp/cli" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
[[ $* == '--no-restart backup apply' ]]
grep -q '^current_theme = text$' "$SPICETIFY_CONFIG/config-xpui.ini"
grep -q '^color_scheme = Comfy$' "$SPICETIFY_CONFIG/config-xpui.ini"
grep -q '^inject_css = 1$' "$SPICETIFY_CONFIG/config-xpui.ini"
grep -q '^inject_theme_js = 1$' "$SPICETIFY_CONFIG/config-xpui.ini"
grep -q '^overwrite_assets = 0$' "$SPICETIFY_CONFIG/config-xpui.ini"
grep -q '^extensions = adblock.js|spicy-lyrics.mjs|oneko.js|text-extra.js$' "$SPICETIFY_CONFIG/config-xpui.ini"
grep -q '^\[Patch\]$' "$SPICETIFY_CONFIG/config-xpui.ini"
grep -q '^\[Text\]$' "$SPICETIFY_CONFIG/Themes/text/color.ini"
grep -q '^\[Comfy\]$' "$SPICETIFY_CONFIG/Themes/text/color.ini"
[[ -f $SPICETIFY_CONFIG/Themes/text/user.css ]]
[[ -f $SPICETIFY_CONFIG/Extensions/theme-setup-ran ]]
spotify_path=$(sed -n 's/^spotify_path = //p' "$SPICETIFY_CONFIG/config-xpui.ini")
[[ -x $spotify_path/spotify ]]
[[ -d $spotify_path/Apps ]]
if [[ -e $HOME/fail-apply ]]; then exit 1; fi
printf 'apply\n' >> "$HOME/applies"
EOF
chmod +x "$tmp/cli"

export KORUNIX_SPOTIFY_SOURCE=$source_dir
export KORUNIX_THEME_NAME=text
export KORUNIX_THEME_SOURCE=$tmp/theme
export KORUNIX_THEME_INJECT_CSS=1
export KORUNIX_THEME_INJECT_JS=1
export KORUNIX_THEME_REPLACE_COLORS=1
export KORUNIX_THEME_OVERWRITE_ASSETS=0
export KORUNIX_THEME_HOME_CONFIG=1
export KORUNIX_THEME_EXPERIMENTAL_FEATURES=0
export KORUNIX_THEME_EXTENSIONS=$tmp/extensions
export KORUNIX_THEME_PATCHES=$tmp/patches
export KORUNIX_THEME_SETUP=$tmp/setup
export KORUNIX_THEME_ADDITIONAL_CSS=$tmp/additional.css
export KORUNIX_PALETTE_FALLBACK=$tmp/fallback
export KORUNIX_SPICETIFY_CLI=$tmp/cli

bash "$root/aplicaciones/spicetify-runtime.sh" --prepare
[[ $(wc -l < "$HOME/applies") == 1 ]]
grep -q '^main = 0F111A$' "$HOME/.config/spicetify/Themes/text/color.ini"
bash "$root/aplicaciones/spicetify-runtime.sh" --launch spotify:album:abc
[[ $(cat "$HOME/launched") == 'spotify:album:abc' ]]

# Noctalia actualiza su archivo; el wrapper lo combina antes de ejecutar apply.
printf '[Comfy]\nmain = ABCDEF\n' > "$HOME/.config/spicetify/Themes/Comfy/color.ini"
bash "$root/aplicaciones/spicetify-runtime.sh" --prepare
[[ $(wc -l < "$HOME/applies") == 1 ]]
grep -q '^accent = 123456$' "$HOME/.config/spicetify/Themes/text/color.ini"
grep -q '^main = ABCDEF$' "$HOME/.config/spicetify/Themes/text/color.ini"

# Una actualización del tema renueva su estructura sin pisar la paleta.
mkdir -p "$tmp/theme2"
cp -a "$tmp/theme/." "$tmp/theme2/"
printf 'new-css\n' > "$tmp/theme2/user.css"
export KORUNIX_THEME_SOURCE=$tmp/theme2
bash "$root/aplicaciones/spicetify-runtime.sh" --prepare
[[ $(cat "$HOME/.config/spicetify/Themes/text/user.css") == 'new-css' ]]
grep -q '^main = ABCDEF$' "$HOME/.config/spicetify/Themes/text/color.ini"
[[ $(wc -l < "$HOME/applies") == 2 ]]

# Si falla el parcheado de una actualización, el Spotify anterior sigue ahí.
mkdir -p "$tmp/theme3"
cp -a "$tmp/theme2/." "$tmp/theme3/"
export KORUNIX_THEME_SOURCE=$tmp/theme3
touch "$HOME/fail-apply"
if bash "$root/aplicaciones/spicetify-runtime.sh" --prepare; then
  echo 'Se esperaba un fallo al aplicar Spicetify.' >&2
  exit 1
fi
[[ -x $HOME/.local/share/korunix/spotify/spotify ]]
[[ $(wc -l < "$HOME/applies") == 2 ]]
echo 'Integración de Spotify: OK'

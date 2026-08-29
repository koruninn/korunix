#!/usr/bin/env bash
set -euo pipefail

config_home="${XDG_CONFIG_HOME:-$HOME/.config}"
gtk_dir="$config_home/gtk-4.0"
gtk_css="$gtk_dir/gtk.css"
import="@import url("korunix-noctalia-live.css");"

mkdir -p "$gtk_dir"

tmp="$gtk_css.korunix.$$"

if [ -f "$gtk_css" ]; then
  grep -Fvx "$import" "$gtk_css" > "$tmp" || true
else
  : > "$tmp"
fi

printf "
%s
" "$import" >> "$tmp"
mv "$tmp" "$gtk_css"

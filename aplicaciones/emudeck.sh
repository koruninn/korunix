set -eo pipefail

if [ "$(id -u)" -eq 0 ]; then
  echo "Abre EmuDeck con tu usuario, sin sudo." >&2
  exit 1
fi

tarjeta="/run/media/$(id -un)/EmuDeck"
imagen="$HOME/Applications/EmuDeck.AppImage"
backend="$HOME/.config/EmuDeck/backend"
accion="${1:-abrir}"
case "$accion" in
  abrir)
    if [ ! -f "$imagen" ]; then
      mkdir -p "$(dirname "$imagen")"
      metadata="$(curl -fsSL --retry 3 https://api.github.com/repos/EmuDeck/emudeck-electron/releases/tags/v2.7.1)"
      asset="$(jq -cer '[.assets[] | select(.name | test("(x86_64|amd64).*\\.AppImage$"; "i"))] | if length == 1 then .[0] else error("No se encontró un único AppImage de Linux x86_64") end' <<< "$metadata")"
      url="$(jq -er '.browser_download_url' <<< "$asset")"
      digest="$(jq -r '.digest // empty' <<< "$asset")"
      temporal="$(mktemp "$(dirname "$imagen")/.EmuDeck.XXXXXXXX")"
      trap 'rm -f -- "$temporal"' EXIT
      curl -fL --retry 3 "$url" -o "$temporal"
      if [[ "$digest" == sha256:* ]]; then
        printf '%s  %s\n' "${digest#sha256:}" "$temporal" | sha256sum -c -
      fi
      chmod +x "$temporal"
      mv -T -- "$temporal" "$imagen"
      trap - EXIT
    fi
    exec appimage-run "$imagen"
    ;;
  srm|reparar-srm)
    if ! mountpoint -q "$tarjeta"; then
      echo "Conecta la tarjeta EmuDeck antes de continuar." >&2
      exit 1
    fi
    if pgrep -u "$(id -u)" -x steam >/dev/null; then
      echo "Cierra Steam por completo antes de abrir Steam ROM Manager." >&2
      exit 1
    fi
    if [ "$accion" = reparar-srm ]; then
      if [ ! -f "$backend/functions/all.sh" ]; then
        echo "Abre EmuDeck y completa su instalación primero." >&2
        exit 1
      fi
      cd "$backend"
      # El backend oficial no está diseñado para ejecutarse con errexit.
      set +e
      source functions/all.sh || exit 1
      set -e
      if [ "${romsPath%/}" != "$tarjeta/Emulation/roms" ]; then
        echo "EmuDeck utiliza otra carpeta de juegos: $romsPath" >&2
        exit 1
      fi
      set +e
      SRM_resetConfig || exit 1
      set -e
    fi
    for srm in "$tarjeta/Emulation/tools/Steam-ROM-Manager.AppImage" "$tarjeta/Emulation/tools/srm/Steam-ROM-Manager.AppImage"; do
      if [ -f "$srm" ]; then
        exec appimage-run "$srm"
      fi
    done
    echo "Steam ROM Manager no está instalado. Abre EmuDeck y selecciónalo." >&2
    exit 1
    ;;
  *)
    echo "Uso: emudeck [srm|reparar-srm]" >&2
    exit 1
    ;;
esac

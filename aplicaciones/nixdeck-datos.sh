# Organizar los datos sin sobrescribir ni mezclar partidas.
set -eu
raiz="$1"
modo="$2"

avisar() { printf 'NixDeck: %s\n' "$*" >&2; }

vincular() {
  local origen="$raiz/$1" destino="$raiz/$2"
  if [ -L "$origen" ]; then
    if [ "$(realpath -m -- "$origen")" != "$(realpath -m -- "$destino")" ]; then
      avisar "Enlace existente conservado: $origen"
    fi
    return 0
  fi
  if [ -L "$destino" ] || { [ -e "$destino" ] && [ ! -d "$destino" ]; } ||
     { [ -e "$origen" ] && [ ! -d "$origen" ]; }; then
    avisar "Ruta no compatible; se conserva: $origen / $destino"
    return 0
  fi

  mkdir -p -- "$(dirname -- "$origen")" "$(dirname -- "$destino")"
  if [ -d "$origen" ]; then
    if [ -d "$destino" ]; then
      if rmdir -- "$origen" 2>/dev/null; then
        : # origen vacío; conservar el destino.
      elif rmdir -- "$destino" 2>/dev/null; then
        mv -T --no-clobber -- "$origen" "$destino"
        if [ -e "$origen" ]; then
          avisar "No se pudo trasladar sin sobrescribir: $origen"
          return 0
        fi
      else
        avisar "Ambas carpetas tienen datos; no se fusionan: $origen / $destino"
        return 0
      fi
    else
      mv -T --no-clobber -- "$origen" "$destino"
      if [ -e "$origen" ]; then
        avisar "No se pudo trasladar sin sobrescribir: $origen"
        return 0
      fi
    fi
  fi

  mkdir -p -- "$destino"
  ln -sT -- "$(realpath --relative-to="$(dirname -- "$origen")" -- "$destino")" "$origen"
}

# Conservar los archivos de BIOS de ambas ubicaciones.
vincular_archivo() {
  local origen="$raiz/$1" destino="$raiz/$2"
  if [ -L "$origen" ]; then return 0; fi
  if [ -L "$destino" ]; then avisar "Enlace existente conservado: $destino"; return 0; fi
  if [ -f "$origen" ] && [ -f "$destino" ]; then
    if ! cmp -s -- "$origen" "$destino"; then
      avisar "Archivos distintos conservados: $origen / $destino"
      return 0
    fi
    # Guardar el original sin reemplazar copias anteriores.
    if [ -e "$origen.nixdeck-respaldo" ]; then return 0; fi
    mv -T --no-clobber -- "$origen" "$origen.nixdeck-respaldo"
  elif [ -f "$origen" ] && [ ! -e "$destino" ]; then
    mkdir -p -- "$(dirname -- "$destino")"
    mv -T --no-clobber -- "$origen" "$destino"
  fi
  if [ ! -e "$origen" ] && [ -f "$destino" ]; then
    mkdir -p -- "$(dirname -- "$origen")"
    ln -sT -- "$(realpath --relative-to="$(dirname -- "$origen")" -- "$destino")" "$origen"
  fi
}

# Evitar dos migraciones al mismo tiempo, no impedir que se juegue.
exec 9>"$raiz/emulators/.nixdeck-organizar.lock"
flock -x 9

organizar() {
  local base
  case "$1" in
    duckstation)
      base=emulators/duckstation/config/duckstation
      vincular "$base/bios" bios/psx
      vincular "$base/memcards" saves/psx
      vincular "$base/savestates" states/psx
      vincular "$base/screenshots" screenshots/psx
      vincular "$base/textures" texture_packs/psx
      vincular "$base/shaders" shaders/psx
      ;;
    pcsx2)
      base=emulators/pcsx2/config/PCSX2
      vincular "$base/bios" bios/ps2
      vincular "$base/memcards" saves/ps2
      vincular "$base/sstates" states/ps2
      vincular "$base/snaps" screenshots/ps2
      vincular "$base/textures" texture_packs/ps2
      ;;
    dolphin)
      base=emulators/dolphin/data/dolphin-emu
      vincular "$base/GC" saves/gc
      # Conservar todo el almacenamiento de Wii, no solo sus partidas.
      vincular "$base/Wii" storage/wii
      vincular saves/wii storage/wii/title
      vincular "$base/StateSaves" states/dolphin
      vincular "$base/ScreenShots" screenshots/dolphin
      vincular "$base/Load/Textures" texture_packs/dolphin
      ;;
    ppsspp)
      base=emulators/ppsspp/config/ppsspp/PSP
      vincular "$base/SAVEDATA" saves/psp
      vincular "$base/PPSSPP_STATE" states/psp
      vincular "$base/TEXTURES" texture_packs/psp
      ;;
    cemu)
      base=emulators/cemu/data/Cemu
      vincular "$base/mlc01" storage/wiiu/mlc01
      vincular saves/wiiu storage/wiiu/mlc01/usr/save
      vincular "$base/shaderCache" shaders/wiiu
      vincular "$base/graphicPacks" mods/wiiu
      vincular_archivo emulators/cemu/config/Cemu/keys.txt bios/wiiu/keys.txt
      ;;
    eden)
      base=emulators/eden/data/eden
      vincular "$base/keys" bios/switch/keys
      for clave in prod.keys title.keys; do
        vincular_archivo "bios/switch/$clave" "bios/switch/keys/$clave"
      done
      vincular "$base/nand" storage/switch/nand
      vincular "$base/sdmc" storage/switch/sdmc
      vincular "$base/load" mods/switch
      vincular "$base/shader" shaders/switch
      vincular "$base/screenshots" screenshots/switch
      vincular saves/switch storage/switch/nand/user/save
      vincular bios/switch/installed storage/switch/nand/system/Contents/registered
      ;;
    azahar)
      base=emulators/azahar/data/azahar-emu
      vincular "$base/sdmc" storage/n3ds/sdmc
      vincular "$base/nand" storage/n3ds/nand
      vincular "$base/sysdata" bios/n3ds/sysdata
      vincular "$base/states" states/n3ds
      vincular "$base/cheats" mods/n3ds/cheats
      vincular "$base/load/textures" texture_packs/n3ds
      vincular saves/n3ds storage/n3ds/sdmc
      ;;
    rpcs3)
      base=emulators/rpcs3/config/rpcs3
      vincular "$base/dev_hdd0" storage/ps3/dev_hdd0
      vincular "$base/dev_flash" storage/ps3/dev_flash
      vincular saves/ps3 storage/ps3/dev_hdd0/home
      vincular bios/ps3/installed storage/ps3/dev_flash
      ;;
    melonds|mgba)
      configurar "$raiz" "$1"
      ;;
    xenia)
      mkdir -p "$raiz/storage/xbox360" "$raiz/saves/xbox360"
      vincular storage/xbox360/content saves/xbox360
      ;;
  esac
}

case "$modo" in
  organizar)
    for emu in duckstation pcsx2 dolphin ppsspp cemu eden azahar rpcs3 melonds mgba xenia; do
      organizar "$emu"
    done
    ;;
  *) organizar "$modo" ;;
esac

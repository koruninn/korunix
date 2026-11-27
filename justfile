# Aplicar los cambios.
switch:
	sudo nixos-rebuild switch --flake ".#veskalia"

# Revisar sin aplicar cambios.
revisar:
	nix flake check --no-build
	nix eval --raw ".#nixosConfigurations.veskalia.config.system.build.toplevel.drvPath"
	@echo

# Construir sin activar el sistema.
probar:
	nix build ".#nixosConfigurations.veskalia.config.system.build.toplevel" --no-link

# Dar formato a los archivos Nix.
formatear:
	nix fmt .

actualizar:
	nix flake update

volver:
	sudo nixos-rebuild switch --rollback

# Limpiar generaciones de más de 30 días.
limpiar:
	sudo nix-collect-garbage --delete-older-than 30d
	nix store optimise

# Registrar huellas.
huellas:
    #!/usr/bin/env bash
    set -euo pipefail

    dedos=(
      left-thumb
      left-index-finger
      left-middle-finger
      left-ring-finger
      left-little-finger
      right-thumb
      right-index-finger
      right-middle-finger
      right-ring-finger
      right-little-finger
    )

    while true; do
      echo
      echo "Huellas registradas para koru:"
      fprintd-list koru || true

      echo
      PS3="Selecciona un dedo (o Terminar): "
      select dedo in "${dedos[@]}" "Terminar"; do
        [[ -n "${dedo:-}" ]] || {
          echo "Selecciona una opción válida."
          continue
        }
        [[ "$dedo" == "Terminar" ]] && exit 0
        break
      done

      fprintd-enroll -f "$dedo" koru

      echo
      read -r -p "¿Registrar otro dedo? [s/N] " continuar || break
      [[ "$continuar" =~ ^[sS]$ ]] || break
    done

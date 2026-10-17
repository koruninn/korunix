# Lo normal: aplicar un equipo por nombre.
switch equipo:
	sudo nixos-rebuild switch --flake ".#{{equipo}}"

# Atajos fáciles para los dos equipos actuales.
veskalia:
	just switch veskalia

optiplex:
	just switch optiplex

# Revisa que la configuración tenga sentido sin modificar el sistema.
revisar:
	nix flake check --no-build
	@set -eu; for equipo in $(nix eval --raw .#nixosConfigurations --apply 'x: builtins.concatStringsSep " " (builtins.attrNames x)'); do \
		echo "Evaluando $equipo"; \
		nix eval --raw ".#nixosConfigurations.$equipo.config.system.build.toplevel.drvPath"; \
		echo; \
	done

# Da formato al código Nix cuando tú quieras. No se ejecuta automáticamente.
formatear:
	nix fmt .

# Construye un equipo sin instalarlo. Sirve para probar cambios con seguridad.
probar equipo:
	nix build ".#nixosConfigurations.{{equipo}}.config.system.build.toplevel" --no-link

# Construye todos los equipos sin instalarlos.
probar-todo:
	@set -eu; for equipo in $(nix eval --raw .#nixosConfigurations --apply 'x: builtins.concatStringsSep " " (builtins.attrNames x)'); do \
		echo "Construyendo $equipo"; \
		nix build ".#nixosConfigurations.$equipo.config.system.build.toplevel" --no-link; \
	done

actualizar:
	nix flake update

volver:
	sudo nixos-rebuild switch --rollback

# La limpieza semanal de 30 días ya es automática; esto fuerza una limpieza.
limpiar:
	sudo nix-collect-garbage --delete-older-than 30d
	nix store optimise

# Registro interactivo de huellas para cualquier usuario local.
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

    actual="$(id -un)"
    read -r -p "Usuario de destino [$actual]: " destino
    destino="${destino:-$actual}"

    if ! getent passwd "$destino" >/dev/null 2>&1; then
      echo "No existe la cuenta local '$destino'." >&2
      exit 1
    fi

    if [[ "$destino" == root ]]; then
      echo "No se permite registrar huellas para root." >&2
      exit 1
    fi

    if [[ "$destino" != "$actual" ]]; then
      echo "Se requieren permisos de administrador para '$destino'."
      sudo -v
    fi

    while true; do
      echo
      echo "Huellas registradas para $destino:"
      if [[ "$destino" == "$actual" ]]; then
        fprintd-list "$destino" || true
      else
        sudo fprintd-list "$destino" || true
      fi

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

      if [[ "$destino" == "$actual" ]]; then
        fprintd-enroll -f "$dedo"
      else
        sudo fprintd-enroll -f "$dedo" "$destino"
      fi

      echo
      read -r -p "¿Registrar otro dedo? [s/N] " continuar || break
      [[ "$continuar" =~ ^[sS]$ ]] || break
    done
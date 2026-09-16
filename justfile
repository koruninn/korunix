# Muestra las acciones disponibles cuando se escribe solamente `just`.
default:
	@just --list

# Lo normal: aplicar un equipo por nombre.
switch equipo:
	sudo nixos-rebuild switch --flake ".#{{equipo}}"

# Aplica el equipo actual. Si se indica un nombre, aplica ese equipo.
aplicar equipo='':
	@equipo="{{equipo}}"; if test -z "$equipo"; then equipo="$(hostname)"; fi; sudo nixos-rebuild switch --flake ".#$equipo"

# Atajos fáciles para los dos equipos actuales.
korunix:
	just switch korunix

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

# Busca código Nix innecesario o sospechoso sin modificar archivos.
analizar:
	nix run nixpkgs#deadnix -- --fail .
	nix run nixpkgs#statix -- check .

# Construye el equipo actual sin instalarlo. También acepta otro nombre.
probar equipo='':
	@equipo="{{equipo}}"; if test -z "$equipo"; then equipo="$(hostname)"; fi; nix build ".#nixosConfigurations.$equipo.config.system.build.toplevel" --no-link

# Construye todos los equipos sin instalarlos.
probar-todo:
	@set -eu; for equipo in $(nix eval --raw .#nixosConfigurations --apply 'x: builtins.concatStringsSep " " (builtins.attrNames x)'); do \
		echo "Construyendo $equipo"; \
		nix build ".#nixosConfigurations.$equipo.config.system.build.toplevel" --no-link; \
	done

# Actualiza todo. Si las nuevas versiones no construyen, restaura el lock anterior.
actualizar:
	@set -eu; copia="$(mktemp)"; cp flake.lock "$copia"; \
	if nix flake update && just probar-todo; then \
		rm -f "$copia"; \
		echo "Actualización comprobada."; \
	else \
		cp "$copia" flake.lock; rm -f "$copia"; \
		echo "La actualización no pasó las pruebas. Se restauró flake.lock." >&2; \
		exit 1; \
	fi

# Actualiza solo una entrada del flake y la prueba antes de conservarla.
actualizar-uno entrada:
	@set -eu; copia="$(mktemp)"; cp flake.lock "$copia"; \
	if nix flake update "{{entrada}}" && just probar-todo; then \
		rm -f "$copia"; \
		echo "{{entrada}} actualizado y comprobado."; \
	else \
		cp "$copia" flake.lock; rm -f "$copia"; \
		echo "{{entrada}} no pasó las pruebas. Se restauró flake.lock." >&2; \
		exit 1; \
	fi

volver:
	sudo nixos-rebuild switch --rollback

# La limpieza semanal de 30 días ya es automática; esto fuerza una limpieza.
limpiar:
	sudo nix-collect-garbage --delete-older-than 30d
	nix store optimise

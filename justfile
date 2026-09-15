# Lo normal: aplicar un equipo por nombre.
switch equipo:
	sudo nixos-rebuild switch --flake ".#{{equipo}}"

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

# Aplica la configuración de un equipo.
aplicar equipo:
	sudo nixos-rebuild switch --flake ".#{{equipo}}"

# Revisa formato y configuraciones sin modificar el sistema.
revisar:
	nix fmt -- --check .
	nix flake check --no-build
	@set -eu; for equipo in $(nix eval --raw .#nixosConfigurations --apply 'x: builtins.concatStringsSep " " (builtins.attrNames x)'); do \
		echo "Evaluando $equipo"; \
		nix eval --raw ".#nixosConfigurations.$equipo.config.system.build.toplevel.drvPath"; \
		echo; \
	done

# Actualiza las versiones fijadas en flake.lock.
actualizar:
	nix flake update

# Vuelve a la generación anterior del equipo actual.
volver:
	sudo nixos-rebuild switch --rollback

# Elimina generaciones antiguas y optimiza el almacén de Nix.
limpiar:
	sudo nix-collect-garbage --delete-older-than 30d
	nix store optimise

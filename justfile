# Justfile

# Reconstruye la configuración NixOS de un equipo.
korunix:
	sudo nixos-rebuild switch --flake .#korunix

optiplex:
	sudo nixos-rebuild switch --flake .#optiplex

# Evalúa todas las configuraciones que el flake descubra en equipos/.
check:
	nix flake check --no-build
	@set -eu; for equipo in $$(nix eval --raw .#nixosConfigurations --apply 'x: builtins.concatStringsSep " " (builtins.attrNames x)'); do \
		echo "Evaluando $$equipo"; \
		nix eval --raw ".#nixosConfigurations.$$equipo.config.system.build.toplevel.drvPath"; \
		echo; \
	done

# Actualiza la versión de los paquetes (flake.lock)
update:
	nix flake update

# Limpia la basura de Nix y optimiza el almacenamiento
clean:
	sudo nix-collect-garbage --delete-older-than 30d
	nix store optimise

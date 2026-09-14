# Justfile

# Reconstruye cualquier equipo descubierto por el flake y lo deja como generación activa.
rebuild equipo:
	sudo nixos-rebuild switch --flake ".#{{equipo}}"

# Construye una generación completa sin activarla.
build equipo:
	nixos-rebuild build --flake ".#{{equipo}}"

# Prueba una generación sin convertirla en el arranque predeterminado.
test equipo:
	sudo nixos-rebuild test --flake ".#{{equipo}}"

# Vuelve a la generación anterior del equipo actual.
rollback:
	sudo nixos-rebuild switch --rollback

# Atajos para los equipos actuales.
korunix:
	sudo nixos-rebuild switch --flake .#korunix

optiplex:
	sudo nixos-rebuild switch --flake .#optiplex

# Evalúa todas las configuraciones que el flake descubra en equipos/.
check:
	nix flake check --no-build
	@set -eu; for equipo in $(nix eval --raw .#nixosConfigurations --apply 'x: builtins.concatStringsSep " " (builtins.attrNames x)'); do \
		echo "Evaluando $equipo"; \
		nix eval --raw ".#nixosConfigurations.$equipo.config.system.build.toplevel.drvPath"; \
		echo; \
	done

# Formatea todos los archivos Nix con el formatter declarado por el flake.
format:
	nix fmt -- .

# Comprueba el formato sin modificar archivos.
format-check:
	nix fmt -- --check .

# Actualiza la versión de los paquetes (flake.lock)
update:
	nix flake update

# Limpia la basura de Nix y optimiza el almacenamiento
clean:
	sudo nix-collect-garbage --delete-older-than 30d
	nix store optimise

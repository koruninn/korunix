# Justfile

# Reconstruye la configuración NixOS de un equipo.
korunix:
	sudo nixos-rebuild switch --flake .#korunix

optiplex:
	sudo nixos-rebuild switch --flake .#optiplex

# Evalúa las dos configuraciones antes de reconstruir un equipo.
check:
	nix flake check --no-build
	nix eval --raw .#nixosConfigurations.korunix.config.system.build.toplevel.drvPath
	nix eval --raw .#nixosConfigurations.optiplex.config.system.build.toplevel.drvPath
	bash tests/spotify-runtime.sh

# Actualiza la versión de los paquetes (flake.lock)
update:
	nix flake update

# Limpia la basura de Nix y optimiza el almacenamiento
clean:
	sudo nix-collect-garbage --delete-older-than 30d
	nix store optimise

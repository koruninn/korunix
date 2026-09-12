# Justfile

# Reconstruye la configuración NixOS de un equipo.
korunix:
	sudo nixos-rebuild switch --flake .#korunix

optiplex:
	sudo nixos-rebuild switch --flake .#optiplex

# Actualiza la versión de los paquetes (flake.lock)
update:
	nix flake update

# Limpia la basura de Nix y optimiza el almacenamiento
clean:
	sudo nix-collect-garbage -d
	nix store optimise

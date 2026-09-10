# Justfile

# Reconstruye todo el sistema NixOS
os:
	sudo nixos-rebuild switch --flake .#korunix

# Actualiza la versión de los paquetes (flake.lock)
update:
	nix flake update

# Limpia la basura de Nix y optimiza el almacenamiento
clean:
	sudo nix-collect-garbage -d
	nix store optimise

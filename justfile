# Justfile

# Reconstruye todo el sistema NixOS
os:
	sudo nixos-rebuild switch --flake .#korunix

# Reconstruye solo el entorno de tu usuario (Home Manager)
home:
	home-manager switch --flake .#koru

# Formatea todo el código Nix del repositorio
fmt:
	nix fmt

# Actualiza la versión de los paquetes (flake.lock)
update:
	nix flake update

# Limpia la basura de Nix y optimiza el almacenamiento
clean:
	sudo nix-collect-garbage -d
	nix store optimise

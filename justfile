# Aplicar los cambios.
switch:
	sudo nixos-rebuild switch --flake ".#veskalia"

# Revisar sin aplicar cambios.
revisar:
	nix flake check --no-build
	nix eval --raw ".#nixosConfigurations.veskalia.config.system.build.toplevel.drvPath"
	@echo

# Construir sin activar el sistema.
probar:
	nix build ".#nixosConfigurations.veskalia.config.system.build.toplevel" --no-link

# Dar formato a los archivos Nix.
formatear:
	nix fmt .

actualizar:
	nix flake update

volver:
	sudo nixos-rebuild switch --rollback

# Limpiar generaciones de más de 30 días.
limpiar:
	sudo nix-collect-garbage --delete-older-than 30d
	nix store optimise


{config, equipo, lib, pkgs, ...}: let
  catalogo = import ./catalogo-plantillas.nix;
  ids = ["pywalfox" "blender" "darktable" "discord" "gimp" "heroiclauncher"
         "inkscape" "libreoffice" "obs" "obsidian" "prismlauncher" "steam" "vscode"];
  preparar = pkgs.writeShellApplication {
    name = "korunix-noctalia-templates";
    runtimeInputs = [pkgs.coreutils];
    text = ''
      state_home="''${XDG_STATE_HOME:-$HOME/.local/state}"
      destino="$state_home/noctalia/community-templates"
      mkdir -p "$destino"
      for id in ${lib.escapeShellArgs ids}; do
        mkdir -p "$destino/$id"
        for origen in "${catalogo}/$id/"*; do
          [ -f "$origen" ] || continue
          nombre="$(basename "$origen")"
          case "$nombre" in
            *.png|*.jpg|*.jpeg|*.webp|README.md) continue ;;
          esac
          # Conservar las plantillas y los cambios que ya existen.
          if [ ! -e "$destino/$id/$nombre" ]; then
            install -m 0644 "$origen" "$destino/$id/$nombre"
          fi
        done
      done
      printf '%s\n' "Korunix: plantillas comunitarias disponibles en $destino"
    '';
  };
  iniciar = pkgs.writeShellApplication {
    name = "korunix-noctalia";
    runtimeInputs = [preparar pkgs.python3 pkgs.zip pkgs.procps pkgs.bash pkgs.coreutils];
    text = ''
      korunix-noctalia-templates || echo "Korunix: no se pudo preparar la caché de plantillas" >&2
      exec ${lib.getExe config.programs.noctalia.package} "$@"
    '';
  };
in lib.mkIf (equipo.habilitarUmbriel or false) {
  environment.systemPackages = [preparar iniciar];
}

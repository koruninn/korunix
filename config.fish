# Desactivar el saludo inicial por defecto de Fish
set -g fish_greeting ""

# Ejecutar fetch solo si la sesión es interactiva (la terminal está abierta).
# Forzamos el mismo logo pequeño de NixOS que usaba Fastfetch.
if status is-interactive
    fetch -l nixos_small
end

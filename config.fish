# Desactivar el saludo inicial por defecto de Fish
set -g fish_greeting ""

# Ejecutar fastfetch solo si la sesión es interactiva (la terminal está abierta)
if status is-interactive
    fastfetch
end

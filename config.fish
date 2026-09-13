# Desactivar el saludo inicial por defecto de Fish
set -g fish_greeting ""

# Ejecutar fetch solo si la sesión es interactiva (la terminal está abierta)
if status is-interactive
    fetch
end

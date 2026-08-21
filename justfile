# just es un atajo para las tareas frecuentes del repositorio. No es un requisito
# para instalar o usar Korunix; simplemente evita memorizar comandos largos durante
# desarrollo y mantenimiento.

# Muestra las acciones disponibles cuando se ejecuta `just` sin argumentos.
default:
    ./scripts/korunix

# Aplica la configuración completa de sistema y usuarios como una sola generación.
os:
    ./scripts/korunix apply

# Abre el punto de entrada actual de Korunix. Cuando exista la GUI, este mismo atajo
# podrá iniciar la aplicación sin cambiar la memoria muscular del repositorio.
korunix:
    ./scripts/korunix status

# Comprueba estructura, sintaxis y evaluación de todos los hosts.
check:
    ./scripts/korunix validate

# Formatea todos los archivos Nix mediante Alejandra.
fmt:
    ./scripts/korunix format

# Prepara una reconstrucción sin aplicarla al equipo.
preview:
    ./scripts/korunix preview

# Construye la generación completa sin activarla.
build:
    ./scripts/korunix build

# Actualiza flake.lock de forma transaccional: si la validación falla, restaura el
# lock anterior y deja el sistema sin aplicar cambios.
update:
    ./scripts/korunix update

# Conserva las tres generaciones de sistema más recientes y recoge lo inaccesible.
clean:
    ./scripts/korunix clean

# Elimina todas las generaciones antiguas del sistema, nunca la generación activa.
clean-all:
    ./scripts/korunix clean-all

# Enseña la rama, los cambios y el modelo Korunix del host actual.
status:
    ./scripts/korunix status

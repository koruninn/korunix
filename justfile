# just ofrece nombres cortos para las operaciones frecuentes de Korunix.
# No contiene lógica propia: scripts/korunix sigue siendo la fuente real del
# comportamiento para que terminal e interfaz gráfica compartan el mismo modelo.

# Muestra las acciones disponibles.
default:
    ./scripts/korunix

# Muestra el estado local sin consultar Internet.
status:
    ./scripts/korunix status

# Enseña todas las generaciones recuperables.
generations:
    ./scripts/korunix generations

# Comprueba estructura, sintaxis y evaluación de todos los hosts.
check:
    ./scripts/korunix validate

# Previsualiza paquetes y servicios sin activar una generación.
preview:
    ./scripts/korunix preview

# Construye la generación completa sin activarla.
build:
    ./scripts/korunix build

# Valida, previsualiza, confirma, aplica y verifica el sistema.
os:
    ./scripts/korunix apply

# Actualiza todas las entradas de flake.lock sin aplicar el sistema.
update:
    ./scripts/korunix update

# Actualiza una entrada concreta, por ejemplo Noctalia.
update-one input:
    ./scripts/korunix update {{input}}

# Vuelve de forma controlada a una generación concreta.
rollback generation:
    ./scripts/korunix rollback {{generation}}

# Enseña qué retiraría la limpieza normal.
clean-preview:
    ./scripts/korunix clean-preview

# Conserva las tres recientes y protege las generaciones necesarias.
clean:
    ./scripts/korunix clean

# Enseña qué retiraría la limpieza agresiva.
clean-all-preview:
    ./scripts/korunix clean-all-preview

# Ejecuta la limpieza agresiva con confirmación fuerte.
clean-all:
    ./scripts/korunix clean-all

# Muestra el árbol útil del repositorio.
structure:
    ./scripts/korunix structure

# Formatea archivos Nix cuando una persona decida hacerlo explícitamente.
fmt:
    ./scripts/korunix format

# Abre el punto de entrada humano actual de Korunix.
korunix:
    ./scripts/korunix status

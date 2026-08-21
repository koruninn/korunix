# just ofrece nombres cortos para las operaciones frecuentes de Korunix.
# No contiene lógica propia: scripts/korunix sigue siendo la fuente real del
# comportamiento para que terminal e interfaz gráfica compartan el mismo modelo.

# Muestra las acciones disponibles.
default:
    ./scripts/korunix

# Muestra las personas, perfiles y estado de adopción del equipo.
users:
    ./scripts/korunix users

# Exporta una identidad portable sin credenciales ni estado de hardware.
profile-export id:
    ./scripts/korunix users export {{id}}

# Inspecciona un bundle portable sin modificar el equipo.
profile-inspect file:
    ./scripts/korunix users inspect {{file}}

# Calcula cómo se integraría un bundle en este host.
profile-plan file:
    ./scripts/korunix users plan {{file}}

# Prepara una importación después de mostrar el plan.
profile-import file:
    ./scripts/korunix users import {{file}}

# Detecta el hardware local y comprueba los hechos adoptados por Korunix.
hardware:
    ./scripts/korunix hardware

# Muestra el estado local sin consultar Internet.
status:
    ./scripts/korunix status

# Muestra los puntos de recuperación disponibles.
recovery:
    ./scripts/korunix recovery

# Alias técnico conservado para compatibilidad.
generations:
    ./scripts/korunix generations

# Comprueba estructura, sintaxis y evaluación de todos los hosts.
check:
    ./scripts/korunix validate

# Previsualiza paquetes y servicios sin activar un sistema.
preview:
    ./scripts/korunix preview

# Construye el sistema completo sin activarlo.
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

# Usa un punto de recuperación una vez en el próximo arranque.
rollback id:
    ./scripts/korunix rollback {{id}}

# Enseña qué retiraría la limpieza normal.
clean-preview:
    ./scripts/korunix clean-preview

# Conserva los tres puntos recientes y protege los necesarios.
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

# Formatea archivos Nix únicamente cuando una persona lo decide explícitamente.
fmt:
    ./scripts/korunix format

# Abre el punto de entrada humano actual de Korunix.
korunix:
    ./scripts/korunix status

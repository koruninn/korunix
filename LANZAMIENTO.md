# Criterios de lanzamiento de Korunix

Korunix solo se considera publicable cuando la puerta automática y la matriz real de aceptación están completas. La ausencia de una prueba no se interpreta como éxito.

## Puerta automática

`scripts/validar-lanzamiento.sh` comprueba en una sola ejecución el árbol Git, formato, pruebas Rust, motor y GUI, evaluación Nix, paquetes para las arquitecturas soportadas, metadatos de escritorio, privacidad del source y contratos automatizables de robustez.

## Matriz real

Antes de etiquetar una versión deben quedar registradas juntas estas comprobaciones:

- instalación gráfica limpia de NixOS → bootstrap remoto → adopción → `nixos-rebuild test` → reinicio;
- segunda ejecución del bootstrap sin cambios, que debe ser idempotente;
- instalación o actualización desde una copia local sin Internet;
- x86_64 y AArch64, con evaluación completa y arranque real donde exista hardware;
- UEFI y BIOS conforme a la política declarada;
- Niri, Hyprland, Plasma y Cinnamon sin contaminación entre sesiones;
- pérdida de red durante una operación remota sin bloquear funciones locales;
- desconexión, dispositivo ocupado y pérdida de señal en audio y cámara;
- escritura y expulsión de almacenamiento extraíble confirmando persistencia antes de declarar éxito;
- recuperación tras interrupción de actualización, restauración y mutaciones declarativas;
- navegación por teclado, escalado, 1366×768, ventana estrecha, diacríticos y textos largos.

## Idiomas

El español es la fuente canónica. La lista de idiomas de Noctalia se verifica contra la revisión fijada en `flake.lock`; Korunix no declara una localización completa si una explicación visible cae silenciosamente a otro idioma. La expansión lingüística es una puerta de lanzamiento, no una cifra histórica codificada a mano.

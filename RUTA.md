# Korunix — hoja de ruta

Este archivo existe para poder retomar el proyecto aunque un chat se agote.

No reemplaza `spec.md`:

- `spec.md` dice **cómo debe comportarse Korunix**;
- `RUTA.md` dice **dónde estamos, qué ya cerró y qué sigue**.

Si existe contexto reciente del proyecto, se revisa primero. Después se cruza con el `spec.md` más reciente de `pruebas`. Esta hoja sirve para no perder el estado operativo entre conversaciones.

## Cómo retomar

Al abrir un chat nuevo:

1. revisar las decisiones, quejas y cierres recientes disponibles;
2. leer el `spec.md` actual de `pruebas`;
3. leer este `RUTA.md`;
4. comprobar el `HEAD` real de `desde-cero`;
5. si el trabajo afecta NixOS, comprobar activa, persistente y preview;
6. no reabrir un bloque cerrado salvo que aparezca una regresión nueva.

## Estado actual

Último cierre funcional de `desde-cero`:

```text
a980aaa4704eace16294d569af00e16bed881b23
añade transferencias seguras y expulsión
```

Spec asociado en `pruebas`:

```text
7346cc2e0c68d26200c82e2ee6338a2378d86aab
registra transferencias y expulsión seguras
```

`main` sigue intacta en:

```text
cc2ff5ba028e49258be0b6dfb6eb4ba4ad8dfb6d
```

Generación activa y persistente:

```text
/nix/store/2f04ngymw4l9i4zdn7lzjrbvd7qgvf8f-nixos-system-korunix-26.11.20260831.34ab990
```

Última puerta funcional:

```text
100 tests de Rust
GUI real: transferencia de 256 MiB verificada
CLI real: transferencia de 64 MiB verificada
expulsión segura real: probada
activa = persistente = preview
```

## Cerrado

Estos frentes no se reabren sin una regresión nueva:

- base Rust + Nix + TOML + GTK4/libadwaita;
- migración de las decisiones actuales del equipo;
- Niri, Hyprland, Plasma y Cinnamon;
- Noctalia aislado a Niri/Hyprland;
- apariencia Predeterminado/Dinámico/Everforest + Claro/Oscuro/Automático;
- aplicaciones actuales e integraciones especiales;
- Steam y Sunshine granulares;
- preview aplicable;
- Apply exacto sin rebuild;
- rollback exacto;
- GUI compartiendo el mismo Rust que la CLI;
- sesión, personas actuales, idioma, teclados y monitor del equipo;
- identidad humana de almacenamiento;
- adopción segura de unidades;
- lectura local de discos sin `nix eval`;
- transferencia segura con progreso real;
- rechazo de sobrescritura silenciosa;
- `fsync` individual sin `sync` global;
- expulsión segura de USB mediante UDisks2.

## Avance comprobado sin cierre: Copias → Historial → Restauración

El motor y la ruta gráfica principal ya están muy avanzados, pero este bloque **no está cerrado**. Quedan pendientes obligatorios de experiencia visual que deben resolverse antes del cierre integral.

Ya se comprobó:

```text
Copia portable de Korunix
→ configuracion.toml + flake.lock + avatares usados
→ sin hardware, contraseñas, claves privadas ni historial interno
→ integridad SHA-256
→ sin sobrescritura silenciosa

Plan de restauración
→ no modifica archivos

Restaurar
→ protección automática de lo actual
→ restaura configuracion.toml + flake.lock + avatares
→ verifica el resultado
→ recupera el estado anterior si falla a mitad
→ no toca hardware.nix
→ no toca NixOS
→ registra protección + restauración en Historial
```

Validación alcanzada antes de cambiar de frente:

```text
108 tests de Rust en paralelo
nix build del motor correcto
nix build de la interfaz correcto
prueba CLI real sobre raíz temporal correcta
prueba GUI real:
  Crear → Elegir → Revisar Plan → Restaurar
  Restaurar bloqueado antes del Plan
  Restaurar habilitado solo después del Plan
  restauración verificada
  NixOS intacto
```

Correcciones que no deben perderse:

- los tests de Historial no comparten `XDG_STATE_HOME`;
- los tests de Preview protegen comportamiento, no frases exactas;
- los ejecutables falsos de los tests de Preview se escriben primero con nombre temporal y luego se publican con `rename`, evitando `Text file busy` dentro de Nix;
- una copia no puede usar un avatar para reemplazar archivos centrales ni atravesar enlaces simbólicos para escribir fuera de Korunix.

### Pendientes obligatorios del cierre integral

No son mejoras opcionales:

- `Ver historial` debe mostrar el contenido claramente dentro de «Copias e historial»;
- `Transferir un archivo` debe separar claramente **Archivo → Elegir archivo** de **Copiar a → unidad de destino**. El selector de unidad no limita el origen del archivo;
- una unidad reconectada debe poder actualizarse sin cerrar y abrir Korunix;
- una transferencia o expulsión larga no debe deshabilitar controles ajenos sin necesidad.

Al volver a este bloque no se repite el motor ya probado salvo que el cambio pueda afectarlo. Se corrige la UX, se hace una prueba gráfica real y recién entonces se mueve el bloque a «Cerrado».

## Ahora

### Gestor humano de actualizaciones

Objetivo:

```text
ver estado actual
→ buscar actualizaciones cuando haya conexión
→ explicar qué cambiaría
→ indicar reinicio o nueva sesión cuando corresponda
→ construir un preview completo
→ aplicar exactamente ese preview
→ verificar
```

Primera versión:

- buscar actualizaciones no modifica NixOS;
- estable/inestable sigue siendo una sola decisión humana;
- no inventar porcentajes mientras Nix resuelve o construye;
- no congelar GTK;
- reutilizar Preview y Apply existentes;
- Apply sigue activando exactamente la generación revisada, sin reconstruir;
- offline muestra el último estado local conocido; Internet solo hace falta para buscar información nueva;
- antes de implementar se revisa el comportamiento útil del corte `d0b40b682fcc6e70f9181a5b2f4b93175cbbe609` y se rescata solo lo que siga teniendo sentido.

## Después

Orden previsto después del gestor humano de actualizaciones:

1. interfaz completa de Aplicaciones con AppStream y caché local;
2. detección y adaptación a otros equipos;
3. ampliar idiomas, teclados, métodos de entrada y Personas;
4. acceso remoto completo con Sunshine/Moonlight + Tailscale;
5. accesibilidad y modo compacto;
6. volver a los pendientes obligatorios de UX acumulados;
7. puerta final integral: ningún pendiente marcado puede quedar abierto.

El orden puede cambiar si aparece una dependencia real o una decisión humana nueva. Si cambia, se actualiza aquí y en `spec.md` cuando el cambio también afecte al comportamiento esperado.

## Regla de trazabilidad

Todo push a `desde-cero` debe revisar esta hoja.

Cuando el push cierra o cambia un bloque funcional, la misma publicación debe dejar aquí:

- qué bloque quedó cerrado;
- el commit funcional que lo cerró;
- la generación aplicada, si hubo Apply;
- la validación real que pasó;
- qué quedó pendiente;
- cuál es el siguiente bloque;
- cualquier regresión o corrección relevante que no deba olvidarse.

Flujo recomendado para un cierre:

```text
código validado
→ commit funcional
→ actualizar RUTA.md con ese commit y el nuevo estado
→ commit de continuidad
→ actualizar spec.md en pruebas si cambió comportamiento
→ push atómico de desde-cero + pruebas
```

El commit de `RUTA.md` no sustituye la validación del código. Sirve para que Git conserve una fotografía legible del estado del proyecto en cada publicación.

No se borra un problema histórico importante para que la hoja “se vea limpia”. Si dejó una lección que evita repetir el error, se conserva de forma breve.

## Regla final

Si un chat nuevo solo pudiera leer dos archivos del repositorio para orientarse:

```text
pruebas/spec.md
desde-cero/RUTA.md
```

deberían bastar para entender qué debe hacer Korunix, qué ya funciona y por dónde continuar.

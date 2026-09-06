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

## Ahora

### Copias → Historial → Restauración

Diseñar las tres piezas juntas para no crear un formato de copia que después estorbe a Historial o Restauración.

Objetivo humano:

```text
elegir qué guardar
→ elegir dónde
→ Korunix crea una copia reconocible y verificable
→ Historial muestra qué existe y si está íntegro
→ Restaurar explica qué recuperará
→ solo entonces restaura
```

La primera versión debe seguir estas reglas:

- no pedir UUID, `/dev/...` ni rutas internas cuando Korunix pueda derivarlas;
- no presentar una copia incompleta como terminada;
- no sobrescribir silenciosamente;
- verificar lo guardado;
- conservar suficiente información para restaurar después;
- mostrar progreso real cuando sea medible;
- no congelar GTK;
- funcionar offline;
- no inventar otra base de datos si archivos simples bastan;
- no confundir este historial de copias con el rollback de generaciones NixOS.

## Después

Orden previsto después de cerrar Copias/Historial/Restauración:

1. gestor humano de actualizaciones;
2. interfaz completa de Aplicaciones con AppStream y caché local;
3. detección y adaptación a otros equipos;
4. ampliar idiomas, teclados, métodos de entrada y Personas;
5. acceso remoto completo con Sunshine/Moonlight + Tailscale;
6. accesibilidad, modo compacto y cierre integral.

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

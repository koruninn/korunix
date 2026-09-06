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
d010427200e0b7dd35747eccda706cead0715654
integra actualizaciones en la interfaz
```

Spec asociado en `pruebas`:

```text
83a334ca5e433bf77610ef7c5b5820753a0d6352
define experiencia gráfica de actualizaciones
```

`main` sigue intacta en:

```text
cc2ff5ba028e49258be0b6dfb6eb4ba4ad8dfb6d
```

Generación activa y persistente:

```text
/nix/store/6px8pcvsjfqr7r8vs6d0i3i3n30bk6jp-nixos-system-korunix-26.11.20260905.c043004
```

Última puerta funcional:

```text
115 tests de Rust
nix build del motor con la suite completa dentro de Nix
Apply real de actualización: exacto, sin rebuild
Rollback real: generación + configuracion.toml + flake.lock
Apply del mismo preview: exacto e inocuo al repetir
GUI real: estado local al abrir + Buscar en segundo plano
búsqueda de la GUI parte del flake.lock actual
cero unidades systemd fallidas
activa = persistente = generación revisada
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
- gestor humano de actualizaciones con búsqueda, Preview, Apply, Rollback y GUI en segundo plano.

## Avance comprobado sin cierre: Copias → Historial → Restauración

Avance funcional publicado en `desde-cero`:

```text
7e8d8881a7937e185c337cc2004f9f4e04d27ab5
avanza copias historial y restauración
```

Este commit conserva el motor y la ruta gráfica principal ya probados. **No cierra el bloque**: los pendientes obligatorios de UX listados más abajo siguen abiertos hasta la puerta final.

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

## Cierre reciente

### Gestor humano de actualizaciones

Avance publicado:

```text
040bf76ae1d8a20fe097a41fe5af901dc578bfe0
añade búsqueda segura de actualizaciones
```

Ya funciona:

```text
estado local
→ lee canal + revisión actual desde configuracion.toml y flake.lock
→ funciona sin Internet

buscar
→ usa nix flake update con un flake.lock candidato fuera del repositorio
→ compara entradas principales y cambios internos
→ guarda la búsqueda localmente
→ si la búsqueda falla conserva la última válida
→ nunca presenta una búsqueda vieja como vigente
→ no cambia flake.lock
→ no cambia NixOS
```

Validación de este avance:

```text
112 tests de Rust en paralelo
nix build del motor
búsqueda real por Internet
flake.lock byte por byte intacto
configuracion.toml intacto
hardware.nix intacto
NixOS activo y persistente intactos
```

Segundo avance publicado:

```text
7b41c2490923ec2f609baeabe905793826e3c3dd
construye preview de actualizaciones
```

Ya se comprobó además:

```text
candidata guardada
→ Preview la usa mediante --reference-lock-file
→ guarda lock base + lock usado junto a la generación
→ flake.lock real queda intacto
→ configuracion.toml y hardware.nix quedan intactos
→ NixOS activo y persistente quedan intactos
→ si flake.lock cambia después, el preview deja de ser válido
→ Apply normal rechaza el preview candidato para impedir una aplicación a medias
```

Tercer avance publicado:

```text
4cafd043ec68b2d4d91b5bce1fc1b03d0a730550
aplica actualizaciones con rollback exacto
```

El ciclo completo ya quedó conectado:

```text
Preview candidato
→ Apply activa exactamente esa generación, sin rebuild
→ publica exactamente el flake.lock usado por el Preview
→ guarda generación + configuracion.toml + flake.lock anteriores
→ Rollback devuelve las tres piezas
→ Apply del mismo Preview puede repetirse
```

Validación real:

```text
Apply candidato real
→ activa = persistente = preview
→ flake.lock = lock del preview

Rollback real
→ activa = persistente = generación anterior
→ configuracion.toml anterior
→ flake.lock anterior

Apply real del MISMO preview otra vez
→ activa = persistente = preview
→ flake.lock = lock del preview

Apply repetido
→ inocuo

cero unidades systemd fallidas
```

Generación activa y persistente después de la prueba:

```text
/nix/store/6px8pcvsjfqr7r8vs6d0i3i3n30bk6jp-nixos-system-korunix-26.11.20260905.c043004
```

Siguiente dentro de Actualizaciones:

```text
GUI
→ estado local inmediato
→ Buscar en segundo plano
→ mostrar cambios humanos
→ Preview en segundo plano
→ Aplicar usa el mismo Apply exacto
→ mostrar reinicio / nueva sesión
```

El bloque de Actualizaciones todavía no se mueve a «Cerrado» hasta probar esa experiencia gráfica.

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

Cuarto avance y cierre publicado:

```text
d010427200e0b7dd35747eccda706cead0715654
integra actualizaciones en la interfaz
```

Puerta gráfica:

```text
la ventana aparece antes de consultar nada remoto
estado local visible al abrir
Aplicar actualización reutiliza el mismo Apply exacto
Buscar actualizaciones corre en segundo plano
la búsqueda iniciada desde la GUI queda asociada al flake.lock actual
NixOS activo y persistente no cambian durante Buscar
```

**Gestor humano de actualizaciones: cerrado.** No se reabre sin una regresión nueva.

## Ahora

### Estructura visual por áreas + Aplicaciones como primera página final

Primer avance de implementación publicado:

```text
bf2068c56c9bdec872eb54bf8a2aa4feefaf6f92
organiza la interfaz por áreas
```

La página interminable ya se separó en áreas reales con navegación desde Inicio. Los motores existentes no se reescribieron. Este avance **no cierra el frente**: todavía falta convertir Aplicaciones en el catálogo final, comprobar sus búsquedas y fichas, y pasar la puerta gráfica integral del bloque.

La experiencia gráfica ya no se amplía como una única página larga. El motor probado se conserva; lo que cambia es cómo se presenta.

Objetivo general:

```text
abrir Korunix
→ ver una navegación clara por áreas
→ entrar a una tarea sin recorrer controles ajenos
→ cada área usa la composición que mejor explica esa tarea
→ las lecturas locales aparecen primero
→ lo lento trabaja en segundo plano
→ GUI y CLI siguen usando el mismo motor Rust
```

Estructura visual de referencia:

```text
General
  Inicio
  Aplicaciones
  Apariencia

Equipo
  Sistema
  Hardware
  Almacenamiento
  Personas

Mantenimiento
  Actualizaciones
  Copias y recuperación

Más adelante
  Acceso remoto
```

No es una obligación de crear nueve módulos técnicos ni nueve archivos. Son **áreas visibles para la persona**. El árbol de Rust se divide solo cuando dividirlo mejora de verdad la comprensión.

Cada área puede verse distinta:

```text
Inicio           → resumen y accesos importantes
Aplicaciones     → catálogo, búsqueda y fichas
Apariencia       → previsualización y controles visuales
Hardware         → dispositivos y estado
Almacenamiento   → unidades, transferencias y expulsión
Personas         → perfiles, idiomas y teclados
Actualizaciones  → estado → buscar → revisar → aplicar
Copias           → copias, historial, restauración y recuperación
```

Reglas de migración:

- no reescribir motores cerrados para acomodar la GUI;
- mover los controles existentes a su área sin cambiar su comportamiento;
- no duplicar acciones entre páginas salvo que sea un acceso rápido claro;
- una salida importante aparece dentro del área que la produjo;
- una operación lenta no congela la navegación;
- una página no espera a otra página para poder mostrarse;
- en ventana angosta, la navegación debe seguir siendo cómoda y no convertirse en una barra lateral inútil.

#### Primera página que define el patrón: Aplicaciones

Aplicaciones se implementa ya con la composición final, no como otro bloque añadido a la página larga:

```text
Aplicaciones

[ Buscar aplicaciones... ]

Instaladas   Todas   Juegos   Multimedia   Oficina   ...

Firefox
Navegador web
✓ Instalado

Blender
Creación 3D
[ Instalar ]

Karere
Paquete encontrado en Nix
[ Instalar ]
```

Comportamiento:

```text
abrir Aplicaciones
→ mostrar inmediatamente lo elegido + catálogo/caché local
→ enriquecer con AppStream local
→ buscar por nombre y descripción sin nix eval repetido
→ si el nombre no está en el catálogo, permitir resolverlo explícitamente con Nix
→ agregar o quitar modifica configuracion.toml
→ las opciones especiales aparecen en la ficha de esa aplicación
→ Preview y Apply siguen siendo los mismos del sistema
```

AppStream mejora nombre, descripción, icono y categoría. **No es una lista blanca.** Una aplicación elegida por la persona no desaparece por no tener ficha.

La caché local existe para abrir y navegar rápido. No se convierte en una base de datos nueva ni en otra fuente de decisiones humanas.

#### Puerta de este frente

Antes de dar por cerrada esta reorientación:

```text
navegación real por áreas
→ Aplicaciones con catálogo local inmediato
→ búsqueda local instantánea
→ app curada instalable
→ app libre resoluble instalable
→ quitar app
→ opciones especiales siguen funcionando
→ Actualizaciones conserva su motor cerrado dentro de su área
→ ventana angosta usable
→ tareas lentas no congelan GTK
→ prueba gráfica real
```

No se declara cerrada solamente porque la navegación o Aplicaciones compilen.

## Después

Orden previsto después de la estructura visual + Aplicaciones:

1. detección y adaptación a otros equipos;
2. ampliar idiomas, teclados, métodos de entrada y Personas;
3. acceso remoto completo con Sunshine/Moonlight + Tailscale;
4. accesibilidad y modo compacto;
5. volver a los pendientes obligatorios de UX acumulados;
6. puerta final integral: ningún pendiente marcado puede quedar abierto.

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

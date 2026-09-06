# Korunix — guía viva

> Este archivo manda sobre la implementación actual.
>
> Primero se revisan las decisiones y quejas más recientes del proyecto. Después se cruza eso con este archivo. Una decisión humana reciente puede corregir una idea vieja del spec.

## 1. Qué queremos

Korunix es una capa sencilla sobre NixOS para que una persona no técnica pueda configurar, mantener, entender y recuperar su sistema sin aprender toda la parte interna de NixOS.

Prioridades:

- rápido;
- seguro;
- fácil de entender;
- fácil de cambiar;
- plano;
- poco burocrático.

La exageración es intencional: una persona muy poco técnica debería poder abrir la configuración, entender qué significa y cambiar una opción sin romperse la cabeza.

Korunix debe ocultar complejidad de NixOS, no crear complejidad nueva.

## 2. Ramas y forma de trabajar

- `main` no se toca. Es la configuración personal original y nuestra referencia de simpleza.
- `pruebas` no se borra. Conserva la implementación anterior, soluciones comprobadas y esta guía viva.
- `desde-cero` es la rama de la nueva implementación.

La nueva versión no copia automáticamente la arquitectura de `pruebas`. Se rescata el comportamiento útil y se pregunta siempre: **¿podemos hacerlo ahora más sencillo?**

Cada cambio importante actualiza este `spec.md` en el mismo trabajo. Si se añade, elimina, modifica o corrige una decisión del producto, el spec cambia también.

Un cambio válido termina con:

1. validación proporcional al cambio;
2. commit;
3. push a la rama correspondiente.

`desde-cero` mantiene además `RUTA.md` como hoja de continuidad operativa. `spec.md` sigue siendo la guía del comportamiento; `RUTA.md` registra dónde está el trabajo: último cierre funcional, generación aplicada cuando corresponda, validación real, bloque actual, pendientes y siguiente frente.

Todo push a `desde-cero` debe revisar `RUTA.md`. Cuando una publicación cierra o cambia un bloque funcional, la misma publicación actualiza esa hoja para que el estado pueda reconstruirse desde Git aunque se pierda el contexto de un chat. El flujo normal es: código validado → commit funcional → actualización de `RUTA.md` → actualización de `spec.md` si cambió comportamiento → push atómico de `desde-cero` y `pruebas`.

Una decisión humana reciente sigue teniendo prioridad sobre una nota vieja de la hoja de ruta. `RUTA.md` no se usa para reabrir comportamientos que ya quedaron cerrados sin una regresión nueva.

No se declara terminado algo solamente porque compila.

## 3. Tecnologías base

La base es pequeña:

- **Rust** hace el programa.
- **Nix** arma y configura NixOS.
- **TOML** guarda las decisiones humanas.
- **GTK4 + libadwaita** hacen la interfaz gráfica.

No se añade otra tecnología salvo que resuelva un problema real.

Home Manager no forma parte de la arquitectura nueva.

No se empieza con daemon propio, DBus propio, base de datos, varios crates ni capas de arquitectura porque sí.

## 4. El modelo Lego, entendido como propiedades de un componente

La idea central es:

```text
lo que la persona quiere
→ Korunix lo entiende y valida
→ Nix deriva la parte técnica
→ NixOS aplica el resultado
```

Una decisión humana se expresa una sola vez.

La comparación útil es Figma: `configuracion.toml` funciona como el panel de propiedades o variantes de un componente. Rust comprueba que la combinación tenga sentido. Nix es la parte técnica que sabe producir el resultado correcto a partir de esas propiedades.

Cambiar una propiedad no significa que Korunix deba abrir y reescribir a mano muchos archivos técnicos. La implementación debe estar preparada para **leer las decisiones y derivar el resultado**.

```text
configuracion.toml
        ↓
propiedades humanas
        ↓
Rust valida
        ↓
Nix deriva paquetes, servicios y archivos
        ↓
NixOS resultante
```

Ejemplos:

```toml
canal = "estable"
```

Eso ajusta todo lo necesario para usar el canal estable.

```toml
[escritorio]
principal = "niri"
```

Eso ajusta lo necesario para Niri.

```toml
[aplicaciones]
instaladas = [
  "firefox",
  "karere",
  "blender",
]
```

Eso instala esas aplicaciones si pueden resolverse de forma fiable, aunque alguna no tenga ficha curada.

La persona no debe editar varios archivos para expresar una sola decisión.

## 5. TOML es la configuración humana

TOML es la entrada manual normal. GUI, CLI y edición manual representan las mismas decisiones.

Cuando Korunix cambie `configuracion.toml`, toca solamente lo que se pidió. Los comentarios y las opciones que no tienen nada que ver se conservan. La GUI y la CLI no deben convertir un archivo fácil de leer en texto generado difícil de editar.

El nombre con el que aparece la computadora en la red también es una decisión humana. Se escribe una sola vez en TOML, por ejemplo `nombre = "korunix"`, y Nix lo usa como nombre del equipo.

Ejemplo:

```toml
canal = "inestable"

[escritorio]
principal = "niri"
instalados = ["niri", "hyprland", "plasma", "cinnamon"]

[apariencia]
estilo = "dinamico"
modo = "automatico"

[aplicaciones]
instaladas = [
  "firefox",
  "karere",
  "blender",
]

[sunshine]
activo = true
autoinicio = false

[steam]
activo = true
remote_play = true
servidor_dedicado = false
```

Korunix deriva paquetes, servicios, permisos, puertos, asociaciones, archivos y otros detalles.

La persona no debe escribir rutas de atributos de Nixpkgs, nombres de systemd, grupos UNIX, reglas de firewall o detalles parecidos cuando Korunix pueda deducirlos.

### 5.1. Controles principales y subopciones

Una función puede tener un control principal y opciones internas.

Apagar una subopción no apaga necesariamente la función principal. Apagar el control principal no borra las preferencias internas; simplemente deja de aplicarlas mientras esté apagado.

```toml
[sunshine]
activo = true
autoinicio = false
```

Sunshine sigue disponible, pero no arranca solo.

```toml
[steam]
activo = false
remote_play = true
```

`remote_play = true` se conserva como preferencia, pero no se aplica mientras Steam esté apagado.

## 6. Nombres, comentarios y textos humanos

Todo lo que una persona pueda leer debe hablar en español humano, directo y sencillo.

Esto incluye nombres de archivos y opciones, comentarios, documentación, mensajes, errores, Rust, Nix, TOML, Niri, Noctalia, scripts y configuración de aplicaciones.

No usar nombres pretenciosos porque “así se programa”. Evitar `orchestrator`, `provider`, `repository`, `facade`, `adapter`, `domain`, `materialization` y similares cuando una palabra normal explica mejor lo mismo.

Si administra aplicaciones: `aplicaciones`.

Si configura el sistema: `sistema`.

Si muestra la ventana: `interfaz`.

Si guarda lo elegido: `configuracion`.

Los comentarios no tienen que sonar a manual ni a texto de IA. Si algo merece una explicación, se explica como si se lo estuvieras contando a alguien al costado: natural, directo y con el contexto necesario.

No usar un “nosotros” de cortesía cuando la acción la hace Korunix. El usuario no tiene por qué quedar metido en una responsabilidad que no tiene.

Raro:

> Primero revisamos el archivo. Si está mal, no guardamos nada.

Mejor:

> Si el archivo está mal, no se guarda el cambio.

Un comentario también tiene que dejar claro qué está explicando. Una frase natural pero sin contexto tampoco ayuda.

Bien:

> Este puerto solo se abre cuando Sunshine está encendido.

No comentar lo obvio. Si el nombre o la línea ya dejan claro qué pasa, no hace falta narrarlo otra vez.

Un comentario sirve cuando aclara qué hace algo, por qué está ahí o qué pasa si se cambia.

## 7. El árbol también es UX

El árbol debe ser lo más plano posible.

Punto de partida deseado:

```text
korunix/
├── flake.nix
├── configuracion.toml
├── sistema.nix
├── Cargo.toml
└── src/
    ├── main.rs
    ├── configuracion.rs
    ├── sistema.rs
    └── interfaz.rs
```

Una carpeta nueva solo aparece cuando de verdad existe una colección que la necesita. Si un archivo crece tanto que dividirlo mejora la comprensión, recién se divide.

No crear carpetas profundas ni capas preventivas “por buenas prácticas”.

## 8. Rust y Nix no compiten

Rust administra decisiones, estado, validación, flujo, errores, GUI y CLI.

Nix sigue siendo quien configura NixOS.

```text
GUI → mismo Rust → Nix → NixOS
CLI → mismo Rust → Nix → NixOS

La interfaz gráfica confirma con acciones normales de la propia interfaz, por ejemplo el botón «Aplicar cambios» o «Volver». No obliga a escribir palabras especiales. La CLI tampoco añade una confirmación textual duplicada después de que la persona ya ejecutó la operación.
```

La GUI no implementa NixOS por su cuenta. La CLI tampoco. No usar Rust para reinventar lo que NixOS ya hace bien.

La CLI no debe depender de abrir una terminal en una carpeta exacta. Si Korunix está en `~/.korunix`, el ejecutable tiene que encontrarlo desde cualquier carpeta.

## 9. Rapidez

Korunix debe sentirse inmediato. Usar Rust no basta: hay que evitar trabajo inútil.

Al arrancar:

1. leer `configuracion.toml`;
2. leer una vez el estado local necesario;
3. mostrar la GUI;
4. precalentar y actualizar en segundo plano lo secundario.

Regla aprendida:

```text
abrir primero
→ precalentar después
→ navegar instantáneamente
```

No ejecutar `nix eval`, `localectl`, `pgrep` u otros procesos repetidamente al abrir cada página.

Las tareas lentas no pueden congelar GTK. Las páginas no deben serializarse detrás de una única espera global. Las operaciones remotas, firmware, actualizaciones y otras lecturas lentas trabajan aparte cuando sea posible.

Korunix es offline-first: si una operación puede resolverse con datos locales, Internet no debe ser requisito.

## 10. Errores humanos y seguridad

Un typo no rompe el sistema.

Si alguien pone:

```toml
principal = "niry"
```

Korunix explica el problema, sugiere `niri` cuando sea razonable y conserva el último estado válido.

Antes de un cambio importante:

- mostrar qué cambia;
- explicar el efecto;
- decir si necesita reinicio o volver a iniciar sesión;
- explicar si necesita autorización;
- pedir privilegios solo cuando hagan falta.

Korunix no llama `preview` a una lista de intenciones. Mientras todavía no exista una generación completa de NixOS que pueda aplicarse después, se llama `plan`.

El plan puede pedirle a Nix que resuelva paquetes y otras consecuencias técnicas, pero no promete ser todavía la diferencia contra el sistema activo.

Preview no modifica el sistema. Representa una generación completa y concreta que puede aplicarse después sin volver a construirla.

Korunix conserva la generación revisada en el estado local de la persona, usando `XDG_STATE_HOME/korunix/preview` o, normalmente, `~/.local/state/korunix/preview`. Ese enlace es una raíz de GC registrada con Nix; un enlace normal por sí solo no cuenta como protección suficiente.

Cuando ya existe un preview válido, el siguiente se construye bajo un enlace temporal que mantiene viva la generación nueva. El enlace estable solo se reemplaza y registra como raíz de GC cuando la construcción terminó bien. Si la construcción o el registro de la raíz falla, el último preview válido se conserva. No existe un segundo comando `construir`: `korunix preview` es el único flujo previo a apply.

Junto al enlace se guardan la ruta exacta de la generación y una copia exacta de `configuracion.toml`. Si la configuración humana cambia después, `korunix aplicar` rechaza ese preview y pide crear uno nuevo. Así un preview no se convierte silenciosamente en permiso para aplicar decisiones distintas.

Apply activa exactamente la generación revisada a la que apunta ese preview, **sin ejecutar `nix build`**, la deja persistente para el siguiente arranque y verifica `activa = persistente = preview`. Si esa generación ya está activa y persistente, el comando es inocuo y no pide privilegios.

Antes de tocar NixOS, Korunix protege la generación activa como `XDG_STATE_HOME/korunix/anterior` mediante una raíz de GC real. Después cruza privilegios con el wrapper real de NixOS (`/run/wrappers/bin/sudo`) y una unidad temporal de systemd. Las rutas de programas críticos se conservan sin resolver sus enlaces simbólicos: herramientas de Nix como `nix-store` y `nix-env` pueden compartir un ejecutable multicall y el nombre con el que se invocan forma parte de su comportamiento.

La versión actual de NixOS exige root incluso para `switch-to-configuration dry-activate`, así que la autorización puede aparecer antes de enseñar la simulación. Esa misma operación privilegiada ejecuta `check`, muestra `dry-activate`, explica kernel/reinicio/sesión/persistencia y aplica exactamente el preview revisado. Ejecutar `korunix aplicar` ya expresa la intención humana de aplicar; si NixOS necesita privilegios, `sudo` es la autorización técnica. No se pide escribir `APLICAR` ni una segunda confirmación equivalente.

Si la activación falla después de empezar el cambio, Korunix intenta volver a la generación protegida como anterior y verifica que vuelva a quedar activa y persistente. Un estado dividido nunca se presenta como éxito.

Rollback es una función normal del producto. `korunix rollback` usa el punto de regreso protegido en `XDG_STATE_HOME/korunix/anterior`, comprueba primero que activa y persistente no estén divididas y no ejecuta `nix build`.

La generación anterior conserva también una copia de la `configuracion.toml` humana asociada. Antes de volver, Korunix ejecuta `check` y `dry-activate`, muestra qué generación quedará activa, si cambia el kernel, si puede requerir volver a iniciar sesión y que la configuración humana volverá con esa generación. Ejecutar `korunix rollback` ya expresa la intención humana de volver; si NixOS necesita privilegios, `sudo` es la autorización técnica. No se pide escribir `VOLVER`. Korunix deja exactamente esa generación como persistente, la activa y restaura la copia humana asociada.

Rollback termina correctamente solo si `activa = persistente = anterior` y `configuracion.toml` vuelve a la copia guardada. Si el sistema ya está en ese punto de regreso, el comando es inocuo y no pide privilegios. Las operaciones largas muestran su fase y no dejan la interfaz muda.

## 11. Aplicaciones

El catálogo curado sirve para nombre bonito, descripción, categoría, opciones especiales e integración adicional. No limita lo instalable.

Una aplicación elegida por la persona debe seguir visible aunque no tenga ficha curada.

Korunix intenta resolver primero una selección humana sencilla, por ejemplo:

```toml
"karere"
```

Si puede resolverse de forma fiable en Nixpkgs, se instala sin obligar a escribir una ruta técnica. Flatpak puede servir como segunda fuente cuando corresponda.

La fuente de instalación y la fuente de información visual son cosas distintas. La futura interfaz de Aplicaciones usa **AppStream** como fuente preferida para enriquecer fichas con nombre, resumen, descripción, icono, capturas, categorías y enlaces. Los metadatos AppStream publicados por Flathub se pueden aprovechar aunque Korunix instale esa misma aplicación desde Nixpkgs.

Ejemplo:

```text
Darktable
→ instalación: Nixpkgs
→ ficha visual: AppStream de Flathub
```

Usar datos de Flathub no convierte automáticamente la instalación en Flatpak.

El emparejamiento tiene que ser fiable: se prefieren AppStream ID, desktop ID o una correspondencia curada. No se asume que dos programas sean el mismo únicamente porque sus nombres se parecen.

Los metadatos y las imágenes se guardan localmente para que la vista abra rápido y siga siendo útil sin conexión. La actualización remota ocurre después, en segundo plano. Si una aplicación no tiene ficha AppStream o no puede emparejarse con seguridad, sigue apareciendo y sigue siendo instalable; Korunix usa la información local disponible y una ficha sencilla.

El catálogo curado queda para lo que Korunix realmente necesita corregir o añadir: nombre humano, categoría, explicación especial, correspondencia AppStream y controles propios. No tiene que duplicar manualmente descripciones y capturas que AppStream ya ofrece bien.

Las dependencias internas que la persona no eligió no deben convertirse automáticamente en aplicaciones visibles.

Las aplicaciones con una fuente especial tampoco obligan a escribir esa fuente en TOML. La elección humana sigue siendo el nombre de la aplicación y Korunix deriva la integración:

```text
cohesion          → Flatpak
figma-linux-next  → Figma Linux Next
genshin-impact    → AAGL
honkai-star-rail  → AAGL
spotify           → Spicetify
```

AAGL sigue el canal del sistema: estable usa su revisión para NixOS estable e inestable usa la correspondiente al canal inestable. Su caché y su clave firmada se derivan únicamente cuando se elige uno de esos juegos.

Cohesion se declara desde Flathub, pero no se actualiza semanalmente por su cuenta. Las actualizaciones de Flatpak forman parte del flujo de actualizaciones de Korunix, igual que las demás.

Un `preview` aplicable también tiene que fijar los artefactos que NixOS instalará fuera del store cuando eso pueda cambiar el resultado entre preview y apply. Para Cohesion, Korunix conserva un commit concreto de Flatpak como detalle técnico; la persona sigue escribiendo solamente `"cohesion"`.

En el equipo actual se adoptó el mismo commit que ya estaba instalado y que Flathub ofrecía en ese momento:

```text
a476d7d1dbee231266f9e904d878ec931bcafe6e37b14191430b5feb1d3da21e
```

`nix-flatpak` queda con actualización durante activación apagada, temporizador apagado y sin borrar Flatpaks que Korunix no administra.

Figma Linux Next se conserva mientras siga siendo la elección comprobada porque aporta la integración `figma://` y comportamiento específico que no se sustituye automáticamente solo porque exista otro paquete llamado Figma en Nixpkgs.

Elegir Spotify deriva Spotify con Spicetify y su funcionamiento Wayland. En Niri/Hyprland, esa misma elección añade la plantilla comunitaria `spicetify` a Noctalia; Noctalia genera la paleta Comfy y Korunix sincroniza sus colores con la copia de ejecución de Spotify. No existe una segunda opción humana para activar Spicetify ni otra plantilla duplicada. Al apagar Spotify, Korunix retira únicamente esa integración y conserva las demás plantillas de Noctalia.

## 12. Escritorios y apariencia

Los escritorios soportados son Niri, Hyprland, Cinnamon y KDE Plasma. GNOME puede aportar aplicaciones o integraciones, pero no es un escritorio soportado de Korunix.

`[escritorio].principal` elige la sesión que se abre por defecto. GDM puede servir como pantalla común de inicio de sesión sin convertir GNOME en un escritorio soportado.

Estas ideas son distintas y no se acoplan:

- escritorio instalado;
- escritorio principal;
- escritorio usado para vista previa;
- compatibilidad de un estilo.

Niri y Hyprland comparten la familia Noctalia. Plasma y Cinnamon conservan su apariencia nativa o neutral cuando una integración equivalente no existe.

Tener Plasma o Cinnamon instalados no bloquea Dinámico o Everforest para Niri/Hyprland.

Los ejes de apariencia son distintos:

```text
Predeterminado / Dinámico / Everforest
Claro / Oscuro / Automático
```

Cuando se adopta una sesión Noctalia existente, `settings.toml` representa los cambios hechos desde la GUI y tiene prioridad efectiva sobre la base de `config.toml`. Una decisión reciente de la GUI no se reemplaza por un valor histórico de Korunix durante la migración.

En el equipo actual la apariencia efectiva adoptada es:

```toml
[apariencia]
estilo = "dinamico"
modo = "automatico"
```

Korunix traduce eso para Noctalia como `source = "wallpaper"` y `mode = "auto"`. Como esas propiedades ya tienen una decisión en TOML, Korunix las alinea en las dos capas de Noctalia; las demás preferencias de Noctalia se conservan.

Las plantillas visuales de Noctalia no deben contaminar Plasma o Cinnamon.

La separación también funciona en sentido contrario. DrKonqi pertenece a Plasma: si Plasma está instalado junto con Niri, Hyprland o Cinnamon, el lanzador gráfico de informes de fallos solo puede arrancar cuando `plasma-workspace.target` está activo. `systemd-coredump` puede seguir registrando el fallo técnicamente, pero fuera de Plasma no aparecen notificaciones gráficas de KDE por esos coredumps.

Este límite quedó restaurado y probado en:

```text
30a692a686e33fe0096719ea43bbae5021ade830
```


Instalar varios escritorios tampoco significa mezclar sus aplicaciones visualmente. Korunix conserva las herramientas propias de cada familia y limita sus lanzadores al escritorio que corresponde:

```text
Niri / Hyprland → Nautilus, Loupe, Papers, editor GNOME y utilidades Noctalia
Cinnamon        → Nemo, Xviewer, Xreader, Xed y su suite nativa
Plasma          → Dolphin, Gwenview, Okular y su suite nativa
```

Una aplicación que la persona eligió explícitamente, como Kate, sigue siendo una aplicación general y no se oculta por pertenecer también a un escritorio.

Blueman y el applet de NetworkManager pueden estar instalados porque Cinnamon los necesita, pero su autoinicio visual se limita a Cinnamon. Niri y Hyprland usan Noctalia para esas funciones.

La integración con teléfonos también se deriva del escritorio: Niri, Hyprland y Cinnamon usan Valent; Plasma usa KDE Connect. Si están instaladas ambas familias, ambas implementaciones pueden coexistir, pero cada una aparece y arranca únicamente en sus sesiones.

Elegir Niri o Hyprland deriva Noctalia como parte de esa familia de escritorio. Cinnamon y Plasma no reciben su servicio ni su configuración. Niri conserva la experiencia comprobada: desenfoque, esquinas redondeadas, cursor Bibata, PiP, atajos, launcher, controles, bloqueo, capturas localizadas, terminal, archivos y navegación.

Noctalia conserva sus valores predeterminados y las preferencias cambiadas desde su interfaz. Korunix no reemplaza un `~/.config/noctalia/config.toml` existente: fusiona únicamente la política que administra. La misma política se aplica a `~/.local/state/noctalia/settings.toml` cuando ese archivo existe.

La familia Noctalia instala los 19 fondos comprobados sin reemplazar la elección actual de la persona. Los iconos usan Hatter: Everforest deriva `Hatter-Green` y Predeterminado/Dinámico usan `Hatter-Slate`. Los cambios visuales que Noctalia puede recoger en vivo no obligan a cerrar sesión.

Las plantillas de aplicaciones se derivan de las aplicaciones elegidas. Steam usa la plantilla comunitaria `steam`; Spotify usa la plantilla comunitaria `spicetify` y sincroniza los 26 colores Comfy con Spotify. Plasma y Cinnamon permanecen fuera de estas plantillas.

Las capturas usan el directorio XDG de imágenes más `Capturas de pantalla` y el patrón `Captura de pantalla del %Y-%m-%d %H-%M-%S`.

NixOS 26.05 todavía no trae Noctalia. Mientras siga así, el canal estable toma solo el paquete Noctalia del input `nixpkgs-inestable` que el flake ya tiene; el resto del sistema continúa usando el canal estable. No se añade un tercer input para resolver esta excepción.

## 13. Servicios y funciones granulares

Funciones como Steam y Sunshine permiten opciones internas sin convertir cada detalle técnico en una pregunta.

Puertos y permisos se derivan de la función. El firewall permanece activo. Un puerto solo se abre cuando una función que realmente lo necesita está activa.

SSH forma parte permanente de la base de Korunix y abre únicamente su regla en el firewall. Avahi también forma parte de la base para el descubrimiento local.

Bluetooth no depende del escritorio instalado. Si la persona lo mantiene activo, sigue activo en Niri, Hyprland, Plasma y Cinnamon. En el equipo actual se expresa así:

```toml
[bluetooth]
activo = true
```

Cuando Bluetooth está activo, Korunix prepara también xpadneo para mandos Xbox compatibles. No se convierte en otra pregunta: es una consecuencia técnica de haber activado Bluetooth.

Flatpak y AppImage son capacidades del sistema aunque en ese momento no haya ninguna aplicación elegida desde esas fuentes. Nautilus dispone de UDisks2 y GVfs para el uso cotidiano de unidades extraíbles.

Sunshine pertenece al acceso/transmisión remota y puede tener autoinicio independiente. En el equipo migrado se expresa como:

```toml
[sunshine]
activo = true
autoinicio = true
```

Impresión y virtualización también son decisiones humanas. El controlador de impresión actual se conserva mientras todavía no exista detección fiable que permita derivarlo sin preguntar.

Steam tiene un control propio y conserva sus preferencias internas aunque esté apagado:

```toml
[steam]
activo = true
remote_play = true
servidor_dedicado = true
```

Steam deriva GameMode. Remote Play y servidor dedicado abren sus reglas únicamente cuando Steam está activo y la subopción correspondiente está encendida. Cuando Steam está activo, Korunix deriva también Millennium como detalle técnico. En Niri/Hyprland, Noctalia habilita la plantilla comunitaria `steam`. Millennium y esa plantilla no son preguntas humanas adicionales.

El acceso remoto más amplio puede integrar Sunshine/Moonlight y Tailscale cuando se trabaje ese frente; no es requisito del primer corte desde cero.

## 14. Hardware y sistema

Korunix debe detectar antes de preguntar cuando sea fiable.

Debe contemplar x86_64, aarch64, UEFI, BIOS, portátil o sobremesa, CPU, GPU, memoria, almacenamiento, firmware, audio, micrófonos y cámaras.

La detección no debe convertirse en una excusa para llenar el arranque de procesos repetidos.

Los UUID de discos, módulos de arranque, arquitectura y otros hechos que NixOS necesita para arrancar no son preferencias humanas y no se meten en TOML. En el primer corte local de `desde-cero` viven en un `hardware.nix` plano. Korunix puede volver a detectarlos más adelante, pero no reemplaza silenciosamente un hardware ya comprobado.

No se crea una carpeta como `generado/equipos/` mientras un solo `hardware.nix` sea suficiente para entender el sistema.

La base habilita firmware redistribuible, fwupd y gráficos de 32 bits cuando la arquitectura es x86_64. Korunix controla cuándo consulta actualizaciones de firmware; el refresco automático de fwupd no se usa como sustituto del flujo de actualizaciones de Korunix.

El arranque comprobado del equipo actual usa `linuxPackages_latest`, Plymouth, los parámetros `quiet`, `splash` y `boot.shell_on_fail`, y mantiene el menú del cargador visible durante 5 segundos. Ese tiempo es una vía normal de recuperación y no se elimina para acelerar unos segundos el arranque.

## 15. Idioma, teclado y personas

Korunix mantiene separadas estas decisiones:

- idioma de la interfaz;
- idiomas preferidos;
- región;
- formatos;
- zona horaria;
- teclados;
- variantes;
- métodos de entrada.

Los nombres visibles son humanos. Los identificadores técnicos quedan debajo.

Se conserva como referencia el trabajo comprobado con IBus y composición Wayland para Niri/Hyprland, pero la nueva implementación debe buscar la forma más simple de cumplir el comportamiento.

La corrección más reciente de ese frente es vinculante: XKB administra las distribuciones normales y IBus es el backend normal para composición y diacríticos. En Niri y Hyprland IBus usa su frontend Wayland. `XMODIFIERS=@im=ibus` sigue disponible en la sesión mientras IBus sea el backend normal, pero Korunix no fuerza `GTK_IM_MODULE` ni `QT_IM_MODULE`. `NIRI_CONFIG` sigue siendo exclusivo de Niri. Fcitx5 queda reservado para métodos de entrada avanzados cuando se implementen.

La configuración humana no expone `es_PE.UTF-8`, `deadtilde`, `grp:alt_shift_toggle` ni el nombre del conector de vídeo si Korunix puede derivarlos. El primer corte de `desde-cero` expresa las decisiones actuales así:

```toml
[idioma]
sistema = "español"
region = "Perú"

[teclado]
distribuciones = ["españa", "latinoamérica"]
cambio = "alt+shift"

[monitor]
resolucion = "1920x1080"
hz = 120
```

De esas propiedades se derivan `es_PE.UTF-8`, `America/Lima`, los teclados XKB España `deadtilde` + Latinoamérica, Alt+Shift y el modo de monitor. `DP-1` permanece como hecho detectado de este equipo dentro de `hardware.nix`, no como decisión humana.

Este primer catálogo solo contiene las elecciones que usa el equipo actual. No cambia el requisito del producto de poder ofrecer todos los idiomas y distribuciones de teclado de forma humana; ese catálogo se ampliará sin obligar a escribir identificadores XKB.

Personas debe permitir gestionar usuarios y preferencias sin pedir rutas o identificadores técnicos cuando una selección gráfica pueda resolverlo.

Las cuentas locales se expresan como bloques `[[personas]]`. Cada bloque puede indicar el nombre de la cuenta, el nombre visible y si es administradora.

Una persona también puede conservar un avatar y qué clave local usa con GitHub:

```toml
[[personas]]
cuenta = "koru"
nombre = "André"
administrador = true
avatar = "avatar-koru.jpg"
clave_github = ".ssh/blep"
```

El avatar puede formar parte de Korunix porque no contiene credenciales. Korunix prepara `~/.face` sin reemplazar un archivo manual. `clave_github` guarda únicamente una ruta relativa dentro de la carpeta personal: la clave privada sigue fuera de Nix y de Git. Nix deriva la configuración de OpenSSH para `github.com`.

Las contraseñas y sus hashes no se guardan en TOML ni en Git. Mientras se adopta una cuenta que ya existe, NixOS mantiene `users.mutableUsers = true` y Korunix no declara una contraseña.

## 16. Almacenamiento, copias e historial

La configuración humana puede elegir unidades detectadas, pero Korunix no inventa un alias y lo presenta como si la persona lo hubiera elegido.

La interfaz usa una identidad reconocible. Si el volumen tiene una etiqueta humana útil, puede usarse. Si no la tiene, se muestran datos que permitan reconocer físicamente la unidad, por ejemplo modelo y capacidad. UUID, dispositivo, formato, UID/GID y ruta de montaje siguen siendo detalles técnicos.

En el equipo actual se comprobó:

```text
ST3500413AS · 500 GB
NTFS
UUID interno: 036F8E656FF00FB2
ruta técnica actual: /mnt/datos
```

La persona no eligió el alias `datos`; apareció durante la reimplementación y quedó corregido. El TOML usa la identidad reconocible:

```toml
[almacenamiento]
disponibles = ["ST3500413AS · 500 GB"]
```

`hardware.nix` conserva el UUID y la ruta técnica. Nix sigue montando exactamente la misma partición. Cambiar el nombre visible no puede cambiar de disco.

El plan de Korunix muestra la identidad humana de la unidad y no obliga a enseñar `/mnt/datos`. La ruta sigue disponible por dentro para NixOS, pero no forma parte de la decisión que la persona tiene que entender.

Almacenamiento tiene una sección propia en la GUI y no se mezcla con «Sesión y equipo».

La ventana se presenta primero y después lee el estado local de discos una sola vez mediante `lsblk`, en segundo plano. No ejecuta `nix eval` para abrir esta sección ni repite la lectura al refrescar controles después de preview, apply o rollback.

El disco que contiene NixOS no se ofrece como almacenamiento adicional. En el equipo actual se comprobó que el sistema está en `ADATA LEGEND 800 · 1 TB`; contiene `/`, `/home`, `/nix` y `/boot`, y queda fuera de esta lista.

La unidad ya adoptada se muestra así:

```text
ST3500413AS · 500 GB
SATA · NTFS · Disponible en Korunix · se monta al usarlo
```

El texto «se monta al usarlo» representa el automount real de `/mnt/datos`. Korunix no presenta ese estado como si el disco estuviera desconectado o tuviera un problema.

Las unidades conectadas que Korunix todavía no sabe adoptar de forma segura sí pueden mostrarse, pero sin un interruptor que prometa una acción inexistente. Durante la prueba se detectó:

```text
DataTraveler 2.0 · 16 GB
USB · exFAT · Etiqueta: Ventoy
Detectado · todavía no administrado por Korunix
```

Intentar activarlo por CLI se rechaza sin modificar `configuracion.toml`. UUID, nombres `/dev/...` y rutas de montaje no aparecen en la GUI ni en la salida humana de la CLI.

La prueba apagar → encender del ST3500413AS fue reversible byte por byte: `configuracion.toml` terminó con el mismo SHA-256 que tenía antes:

```text
dcabb9d73a2b94de078b0532b60b7e128f989d12ddec8f27abbcfd8a22e1fa0e
```

Los 89 tests de Rust pasaron en serie, la GUI se probó visualmente y el motor empaquetado por Nix confirmó el mismo inventario local.

Código publicado:

```text
eda1d6e8aa9c4f4e1954e8f65ad39131775f0617
```

Generación aplicada:

```text
/nix/store/x8dylkhdlvsx5s3g2i8pj5l3gz02pgbc-nixos-system-korunix-26.11.20260831.34ab990
```

La adopción segura de unidades nuevas quedó implementada y probada con una DataTraveler que usa Ventoy.

La persona ve:

```text
DataTraveler 2.0 · 16 GB
USB · exFAT · Etiqueta: Ventoy
Administrar
```

Al pulsar «Administrar», Korunix hace una sola lectura local de la unidad, elige la partición de datos cuando existe una diferencia inequívoca y guarda por dentro la identidad técnica necesaria. En la prueba real, la unidad tenía:

```text
partición de datos: exFAT · Ventoy · 14,4 GiB
partición auxiliar: VTOYEFI · 32 MiB
```

Korunix eligió automáticamente la partición grande. La persona no tuvo que escribir ni elegir UUID, `/dev/sdb1` ni una ruta de montaje.

La identidad técnica temporal comprobada fue:

```text
UUID: BAF1-579A
ruta derivada: /mnt/korunix/baf1579a
```

Esos datos no aparecen en la GUI ni en la salida humana normal. Si dos particiones tienen tamaño importante, Korunix se niega a adivinar. Si dos discos tienen el mismo modelo y capacidad, solo añade un sufijo corto de serie cuando hace falta para distinguirlos; no expone la serie completa de manera normal.

La adopción modifica `configuracion.toml` y `hardware.nix` como una sola operación lógica, valida el resultado con Nix y restaura ambos archivos si la validación falla. No crea preview ni aplica NixOS automáticamente.

La GUI muestra progreso en la propia fila: «Comprobando…» y después «Administrada» o el error correspondiente. Una unidad que Korunix no pueda adoptar de forma segura no recibe un botón que prometa una acción inexistente.

Durante este frente apareció una regresión importante: `lsblk` sin árbol explícito separaba las particiones de su disco padre. Eso hacía aparecer incorrectamente el ADATA del sistema y ocultaba la partición exFAT de Ventoy. Quedó corregido pidiendo el árbol explícitamente. El ADATA que contiene `/`, `/home`, `/nix` y `/boot` vuelve a quedar fuera del almacenamiento adicional.

La prueba real de «Administrar» funcionó. Después se restauraron byte por byte `configuracion.toml` y `hardware.nix`, porque probar la función no significa que la persona haya decidido conservar esa memoria USB. La DataTraveler queda detectada pero no administrada hasta que se elija de nuevo de forma intencional.

Los 94 tests de Rust pasaron en serie antes de la prueba gráfica. Después se verificó la adopción real con Nix, se restauró el estado humano anterior, se construyó un preview completo y Apply activó exactamente ese preview sin reconstruir NixOS.

Código publicado:

```text
eb40bafe536bf2510ca6b5cbb1227e706d1559e9
```

Generación aplicada:

```text
/nix/store/scj9sffw5gnr6zzcag7wyyvqp8ijxbas-nixos-system-korunix-26.11.20260831.34ab990
```

Las transferencias pesadas y la expulsión segura ya quedaron implementadas en este frente. Siguen pendientes dentro de Almacenamiento las copias, el historial y la restauración. El rollback de generaciones de NixOS ya existe como función general de Korunix.

### Transferencias pesadas y expulsión segura probadas

CLI y GUI usan el mismo Rust. La persona elige un archivo y una unidad por su nombre humano; UUID, `/dev/...` y la ruta interna de montaje no se convierten en preguntas.

La primera versión transfiere un archivo normal cada vez hacia una unidad administrada y ya aplicada en la generación activa. Una unidad elegida solo en TOML todavía no puede recibir archivos: primero debe existir su `mount` o `automount` en NixOS. Esto conserva el comportamiento del ST3500413AS de «se monta al usarlo» sin confundir «desmontado ahora» con «no aplicado».

La copia se hace primero bajo un nombre temporal oculto. Korunix escribe por bloques, calcula el progreso con bytes realmente escritos y muestra porcentaje, velocidad y tiempo restante cuando ya puede medirlo de forma razonable. El `100%` se reserva para después de sincronizar el archivo individual, publicar el nombre final y verificar el tamaño. No se ejecuta `sync` ni `syncfs` global.

Si ya existe un archivo con el mismo nombre, Korunix se niega a sobrescribirlo. Un error normal limpia el temporal y nunca presenta el nombre final como si la copia incompleta hubiera terminado.

La prueba real por CLI copió 64 MiB al ST3500413AS, comparó el contenido y observó con `strace` `fsync`/`fdatasync` del archivo sin `sync()` ni `syncfs()`. La prueba gráfica copió 256 MiB, mostró progreso real durante la operación, terminó en `100%` solo al final, verificó contenido idéntico y no dejó residuos parciales.

Las unidades USB muestran «Expulsar» tanto si ya están administradas como si solo están detectadas de forma segura. Korunix reutiliza UDisks2, que ya forma parte del sistema: desmonta las particiones montadas y después pide apagar el dispositivo físico. Si una unidad está ocupada o UDisks devuelve un error, Korunix se detiene y no fuerza la expulsión.

La DataTraveler real se expulsó correctamente: desapareció de `lsblk` y del inventario de Korunix antes de desconectarla físicamente. Después de varias pruebas rápidas, el controlador USB llegó a devolver `error -71` al reenumerarla; eso ocurrió antes de que existiera un dispositivo de bloques y no se trató como un error de Korunix. Tras dejarla desconectada unos segundos volvió a enumerar en el mismo puerto.

También se midió una condición transitoria normal al reconectar: el disco físico puede aparecer alrededor de un segundo antes que `sdb1` y `sdb2`. La lectura local espera una vez a udev y, únicamente si detecta una USB incompleta, da una segunda oportunidad breve antes de hacer una única segunda lectura de `lsblk`. El caso normal sigue usando una sola lectura y la GUI continúa haciéndola fuera del hilo de GTK.

La GUI presenta «Administrar» y «Expulsar» en la DataTraveler, y «Transferir un archivo» con selector de archivo, destino humano y barra de progreso. Las operaciones se ejecutan fuera del hilo de GTK. La prueba visual confirmó que la ventana siguió respondiendo mientras copiaba.


La interfaz debe dejar inequívoco que el selector de archivos elige **el archivo de origen** y puede navegar por cualquier ubicación accesible. El selector de unidad elige únicamente **a dónde se copiará**. La presentación humana debe separar:

```text
Archivo
[ Elegir archivo ]

Copiar a
[ unidad de destino ]

[ Copiar archivo ]
```

No se restringe el selector de archivos al disco elegido como destino.

Los 100 tests de Rust pasaron en serie antes del cierre. El preview completo se revisó y Apply activó exactamente esa generación sin reconstruirla. Después se verificó `activa = persistente = preview` y que no quedaran unidades systemd fallidas.

Código publicado en `desde-cero`:

```text
a980aaa4704eace16294d569af00e16bed881b23
```

Generación aplicada:

```text
/nix/store/2f04ngymw4l9i4zdn7lzjrbvd7qgvf8f-nixos-system-korunix-26.11.20260831.34ab990
```

La DataTraveler sigue detectada y no administrada; probar expulsión o adopción no convierte una memoria temporal en una decisión humana permanente.

### Pendientes obligatorios antes de cerrar Almacenamiento/Copias

Estos puntos quedan aplazados para no seguir bloqueados visualmente en este frente, pero **no son opcionales** y deben resolverse antes del cierre integral:

- `Ver historial` debe mostrar el contenido claramente dentro de «Copias e historial»;
- una unidad reconectada debe poder actualizarse sin cerrar y abrir Korunix;
- una transferencia o expulsión larga no debe deshabilitar controles ajenos sin necesidad;
- el cierre visual debe comprobar de nuevo que «Transferir un archivo» distingue origen y destino sin ambigüedad.

Nada marcado aquí puede quedar inconcluso al declarar Korunix terminado.

## 17. Actualizaciones

Korunix administra también las actualizaciones del sistema y soporta canal estable e inestable.

La persona elige una vez y Korunix deriva los inputs y detalles relacionados.

Antes de actualizar se explica qué va a cambiar y si el resultado necesita reinicio o volver a iniciar sesión.


Buscar actualizaciones no modifica NixOS. La GUI muestra primero el estado local conocido y solo necesita Internet para buscar información nueva. Las tareas de búsqueda, resolución y construcción trabajan fuera del hilo de GTK.


La sección gráfica de Actualizaciones muestra su salida dentro de la propia sección. Al abrir Korunix lee primero el estado local, sin red. `Buscar actualizaciones` usa el mismo motor Rust de la CLI en segundo plano y no inventa porcentajes. Una búsqueda nueva exige volver a revisar antes de habilitar su aplicación.

`Revisar actualización` construye el preview completo fuera del hilo de GTK. Mientras se construye una generación concreta, Korunix bloquea los controles que podrían cambiar esa configuración a mitad de la construcción. `Aplicar actualización` usa el mismo Apply exacto ya revisado; no existe una ruta de activación aparte para la GUI.


La búsqueda tampoco reescribe `flake.lock`. Korunix pide a Nix un `flake.lock` candidato y lo guarda en su estado local junto con la base exacta desde la que se buscó. Si la búsqueda falla, la última búsqueda válida no se borra. Si `flake.lock` cambia después, esa búsqueda se considera antigua y no se presenta como vigente.


El preview de una actualización usa ese `flake.lock` candidato directamente, sin sustituir temporalmente el `flake.lock` del repositorio. Korunix guarda junto al preview tanto el lock base como el lock realmente usado para construir la generación. Si `configuracion.toml` o el lock base cambian después, ese preview queda inválido.

Una generación de actualización no puede pasar por un Apply que ignore el lock con el que fue construida. Apply activa exactamente la generación revisada, sin `nix build`, y publica exactamente el `flake.lock` usado por ese preview.

Antes del cambio, Korunix protege como rollback la generación activa, la `configuracion.toml` asociada y el `flake.lock` asociado. Si publicar el lock nuevo falla después de activar la generación, Korunix recupera la generación anterior y conserva el lock anterior. El éxito exige `activa = persistente = preview` y `flake.lock = lock usado por el preview`.

Rollback de una actualización devuelve en conjunto la generación anterior, su `configuracion.toml` y su `flake.lock`. Si existían ediciones locales sin aplicar, se conservan como borradores antes de volver. Un Apply del mismo preview ya aplicado es inocuo y no vuelve a construir.

No se inventan porcentajes mientras Nix está resolviendo o construyendo. El flujo reutiliza Preview y Apply: buscar o revisar no autoriza a aplicar otra generación distinta, y Apply sigue activando exactamente el preview revisado sin reconstruir.

No inventar porcentajes si Nix no puede ofrecerlos. Sí mostrar fase y actividad.

## 18. GUI

GTK4 + libadwaita son la base visual.

La GUI muestra, pregunta, manda decisiones al mismo Rust que usa la CLI y presenta resultados. No contiene una segunda implementación del sistema.

### Primer corte GTK4/libadwaita probado

La primera interfaz de `desde-cero` quedó en un único `src/interfaz.rs`, sin crear otra arquitectura alrededor. GTK4 y libadwaita son dependencias opcionales del binario gráfico; la CLI sigue pudiendo compilar y funcionar sin arrastrarlas.

La ventana abre leyendo `configuracion.toml` y estado local. No ejecuta `nix eval` al arrancar. Las operaciones largas se ejecutan fuera del hilo de GTK y la ventana sigue respondiendo mientras Korunix trabaja.

Los botones «Crear preview», «Aplicar cambios» y «Volver a la generación anterior» llaman al mismo motor público que usa la CLI. La autorización de apply/rollback se muestra gráficamente con `pkexec` cuando hace falta; no se pide escribir `APLICAR` ni `VOLVER`.

El flujo completo quedó probado desde la GUI:

```text
preview
→ no cambió NixOS

aplicar
→ activó exactamente el preview revisado
→ no ejecutó nix build
→ activa = persistente = preview

rollback
→ volvió exactamente a la generación protegida
→ restauró configuracion.toml
→ no ejecutó nix build
```

Base GTK probada:

```text
ddac20a2bf9e996bf6d2d866c54e38afaa18b37c
```

Generación final usada después del ajuste de integración de escritorios:

```text
/nix/store/2mi464gf6r3sx7kqrkv1wzczijmaymgy-nixos-system-korunix-26.11.20260831.34ab990
```


### La GUI edita la configuración humana

La GUI no interpreta `configuracion.toml` por su cuenta. `src/interfaz.rs` reutiliza directamente `src/configuracion.rs`, así que GUI y CLI comparten lectura, validación y guardado para las decisiones que ya están implementadas.

El primer editor gráfico permite cambiar:

- nombre del equipo;
- canal estable/inestable;
- escritorio principal;
- aplicaciones libres, tanto agregar como quitar.

Guardar una de estas opciones modifica solamente `configuracion.toml`. No ejecuta preview, no reconstruye NixOS y no aplica nada automáticamente.

Si `configuracion.toml` ya no coincide con la copia asociada al preview, la GUI lo muestra y desactiva «Aplicar cambios». La persona tiene que crear un preview nuevo antes de poder aplicar esas decisiones. «Crear preview» permanece disponible.

La prueba manual fue reversible:

```text
nombre:       korunix → korunix-prueba → korunix
canal:        inestable → estable → inestable
escritorio:   niri → hyprland → niri
aplicaciones: 37 → 38 → 37
```

La aplicación temporal usada fue `korunix-prueba`. Al terminar, `configuracion.toml` volvió exactamente a los mismos bytes con los que empezó la prueba:

```text
SHA-256 71d874421ff13d65e998e859e9727145883619e543250aae70edf9251e1302fb
```

La GUI editable quedó publicada en:

```text
12d1602f80b6e7bae0b740eb416aae71dbc1c7b1
```

y aplicada dentro de esta generación completa:

```text
/nix/store/zcm9hh969kkifyld6cd7lj9s9g168gva-nixos-system-korunix-26.11.20260831.34ab990
```


### Apariencia y funciones granulares desde GUI y CLI

La misma lógica de `src/configuracion.rs` permite ahora editar desde GUI y CLI:

- apariencia por estilo: Predeterminado, Dinámico o Everforest;
- modo: Claro, Oscuro o Automático;
- Bluetooth;
- Sunshine y su autoinicio;
- Steam, Remote Play y servidor dedicado;
- impresión;
- virtualización.

Las opciones principales y sus subopciones siguen siendo independientes. Apagar Sunshine conserva `autoinicio`. Apagar Steam conserva `remote_play` y `servidor_dedicado`. La interfaz mantiene esas preferencias visibles mientras la función principal está apagada.

Las ediciones siguen tocando solamente `configuracion.toml`; NixOS no cambia hasta crear y aplicar un preview.

La prueba gráfica comprobó específicamente:

```text
Sunshine apagado
→ autoinicio siguió activado

Steam apagado
→ Remote Play siguió activado
→ servidor dedicado siguió activado
```

También se probó el cambio reversible de apariencia y de los interruptores. Al terminar, `configuracion.toml` volvió exactamente al contenido inicial:

```text
SHA-256 71d874421ff13d65e998e859e9727145883619e543250aae70edf9251e1302fb
```

Los 84 tests de Rust pasaron en serie. El test de reemplazo de preview que primero devolvió `Text file busy` también pasó al ejecutarse solo; `preview.rs` no formaba parte de este cambio.

Código publicado:

```text
ba53dfc4c0273a0cf14417a4a3cbfa1bb5acc6e3
```

Generación aplicada:

```text
/nix/store/9v0zbbnv88gdyxbng725lpdg2rld7574-nixos-system-korunix-26.11.20260831.34ab990
```


### Sesión y equipo desde GUI y CLI

La GUI y la CLI permiten ahora cambiar mediante la misma lógica de `src/configuracion.rs`:

- qué escritorios quedan instalados entre Niri, Hyprland, Plasma y Cinnamon;
- qué teclados normales quedan disponibles entre España y Latinoamérica en el catálogo actual;
- resolución y frecuencia del monitor.

El escritorio principal no puede quitarse de los escritorios instalados. Tampoco se acepta dejar el equipo sin ninguna distribución de teclado. La combinación Alt+Shift se conserva mientras todavía no exista un selector humano para cambiarla.

La primera prueba gráfica incluyó un interruptor llamado «Unidad datos». La persona señaló que ese nombre no permitía saber qué disco era y que nunca había elegido ese alias. Esa fila no se publicó.

La lectura real del equipo identificó:

```text
/dev/sda
ST3500413AS
465,8 GiB
SATA

/dev/sda1
NTFS
UUID 036F8E656FF00FB2
/mnt/datos
```

La corrección separa almacenamiento de «Sesión y equipo». `ST3500413AS · 500 GB` es la identidad humana; UUID y `/mnt/datos` quedan internos.

El cierre detectó además que el plan de Nix ya había dejado de emitir la ruta técnica, pero Rust todavía la exigía. Eso hacía fallar preview con `missing field ruta`. El contrato se corrigió: el plan visible de almacenamiento contiene la identidad humana, no la ruta técnica.

Los 87 tests de Rust pasaron en serie. El motor nuevo entendió el plan real, el preview completo se construyó y Nix comprobó que `/mnt/datos` sigue resolviendo al UUID original.

Código publicado:

```text
05e3ee3bc04536f64e7db1bad88dd36880e72d2a
```

Generación aplicada:

```text
/nix/store/c36riszpnm8b5iyv241igj8x2gm8cc58-nixos-system-korunix-26.11.20260831.34ab990
```


Debe respetar navegación por teclado, foco visible, lectores de pantalla, escalado de texto, contraste, traducciones largas, diacríticos, CJK, preparación para RTL, modo compacto y ausencia de clipping.

Las secciones pueden tener composiciones diferentes según la tarea. Compartir libadwaita no significa clonar la misma página doce veces.

## 19. Tests

Se prueba comportamiento real.

Ejemplos:

- una configuración válida funciona;
- una inválida no se aplica;
- una app se añade y se quita;
- una aplicación no curada sigue visible;
- opciones granulares funcionan;
- preview no modifica;
- un preview fallido conserva el último preview válido;
- el preview concreto queda protegido frente al recolector de basura;
- cambiar `configuracion.toml` después del preview lo invalida;
- los ejecutables multicall conservan el nombre con el que deben invocarse;
- apply rechaza un estado dividido entre activa y persistente;
- apply activa exactamente lo revisado sin reconstruir;
- un segundo apply sobre la misma generación es inocuo;
- un fallo de activación intenta recuperar la generación anterior;
- activa y persistente coinciden;
- rollback vuelve exactamente a la generación protegida sin reconstruir;
- rollback restaura la `configuracion.toml` asociada;
- rollback repetido sobre la misma generación es inocuo;
- una lectura normal no reevalúa Nix sin necesidad;
- la GUI no se congela;
- una página lenta no bloquea las demás.

Evitar tests burocráticos que solo protegen una frase, una posición o un detalle accidental.

## 20. Qué rescatamos de `pruebas`

`pruebas` es referencia de comportamiento, no plantilla de arquitectura.

Cosas que ya demostraron ser útiles:

- Rust + Nix + GTK4/libadwaita;
- GUI y CLI compartiendo lógica;
- preview/apply/rollback;
- autorización concentrada;
- activa = persistente después de apply;
- offline-first;
- lecturas locales rápidas;
- trabajo lento en segundo plano;
- Niri/Hyprland/Cinnamon/Plasma;
- Noctalia aislado a Niri/Hyprland;
- apariencia por estilo y modo separados;
- apps curadas + libres;
- opciones granulares de Steam/Sunshine;
- hardware y arquitecturas;
- usuarios y localización;
- multimedia;
- firmware;
- almacenamiento;
- copias e historial;
- firewall y puertos derivados;
- canales estable/inestable;
- actualizaciones humanas;
- accesibilidad y modo compacto.

Corte histórico de referencia:

```text
d0b40b682fcc6e70f9181a5b2f4b93175cbbe609
```

Ese corte demuestra soluciones que funcionaron. No obliga a copiar su estructura.

## 21. Estado de `desde-cero`

Primer Lego:

```text
2452268a138c54eeb89b119e803a4ffe964a92be
```

Primera validación en Rust:

```text
cc062f9a8d4f8b68350f1053f4749e94a55bc381
```

Edición de aplicaciones:

```text
a8dc1c7de221ef8bf10555a5b40bab798a6df3fc
```

Edición del canal:

```text
63685719b5b2067a5348abc0f9c17c7f008f544d
```

Primer plan resuelto por Nix:

```text
b9b922901214be5940992acc7f17bddeba9bffba
```

Primera generación completa construida sin activar:

```text
840e294748903a5d67c7a4b0464eaf6476f74ea6
```

Cuenta local y escritorio principal:

```text
1604fcd80db7d892e57d2505ab3206708eae7936
```

Sesión Niri básica con Noctalia:

```text
3791dc183ff838097089e282626d83a69a0b8f7a
```

Idioma, teclado, IBus Wayland y monitor:

```text
6e7c84be34a99ec9e44f682388b7d344796cd4ef
```

La configuración humana ya expresa idioma, región, teclados, combinación para alternarlos y modo del monitor con nombres entendibles. Nix deriva los detalles técnicos.

El equipo actual produce:

```text
locale        → es_PE.UTF-8
zona horaria  → America/Lima
teclados      → España + Latinoamérica
cambio        → Alt+Shift
entrada       → IBus Wayland
monitor       → DP-1 · 1920x1080 @ 120 Hz
```

`DP-1` no vive en TOML: es un hecho del equipo. La resolución y los Hz sí son decisiones humanas.

Niri carga teclado y monitor mediante fragmentos generados por Nix y la combinación completa pasa `niri validate`.

IBus conserva la corrección ya demostrada en `pruebas`: frontend Wayland en Niri/Hyprland, `XMODIFIERS=@im=ibus` y sin forzar módulos GTK/Qt. No se repite la prueba viva de diacríticos porque ese comportamiento ya estaba cerrado.

Auditoría activo vs candidata:

```text
SSH / Avahi / Flatpak / AppImage / UDisks2 / GVfs / fwupd
→ faltaban en la primera candidata

git / just / tree / wget
→ estaban en el sistema activo y forman parte de la base de producto
```

Base cotidiana recuperada:

```text
d487d602b0a2fc98b2c3242978133fc82d2d37a0
```

Estas capacidades no se añaden al TOML como preguntas nuevas: Korunix las deriva como base. El firewall sigue activo y SSH/Avahi abren únicamente las reglas que les corresponden.

La auditoría también confirmó una regla distinta: el sistema activo sirve para descubrir posibles regresiones, pero no se copia ciegamente. La fuente para conservar intención humana es la configuración explícita más reciente.

Antes de llamar preview aplicable a una generación de `desde-cero`, hay que migrar las decisiones humanas vigentes de `pruebas` que todavía no están en TOML. En este equipo incluyen:

- Hyprland, Plasma y Cinnamon como escritorios adicionales;
- la unidad de datos adoptada y su preferencia de disponibilidad;
- las aplicaciones generales elegidas, incluidas Steam y las integraciones especiales;
- Sunshine, impresión y virtualización activas;
- el controlador de impresión ya elegido.

Los UUID, UID/GID, conector de monitor y otros hechos físicos no se convierten en preguntas humanas durante esa migración.

Primer bloque de decisiones del equipo migrado:

```text
5771e6884e733f1ba4db4a6b5696791a94d6e863
```

`configuracion.toml` ya conserva:

```text
Niri como principal
+ Hyprland
+ Plasma
+ Cinnamon

datos disponible
Sunshine activo + autoinicio
impresión activa + controlador actual
virtualización activa
```

Los hechos de `datos` siguen fuera del TOML. Antes de adoptarlos se comprobó que la UUID `036F8E656FF00FB2` sigue presente como NTFS y que la cuenta local conserva los identificadores esperados.

Nix deriva `/mnt/datos`, automount, acceso, servicio y firewall de Sunshine, impresión, escáner, libvirt y los cuatro escritorios. Noctalia está instalado cuando existe Niri/Hyprland y su servicio tiene una condición de sesión para no arrancar en Plasma ni Cinnamon. Si `XDG_CURRENT_DESKTOP` identifica la sesión actual, esa señal tiene prioridad sobre variables viejas de una sesión anterior.

Aplicaciones normales y Steam migrados:

```text
c1b48366ecc5da508dcd4bf13e6f081e461aadff
```

La selección humana actual ya expresa 32 aplicaciones normales en `[aplicaciones].instaladas`. `git`, `just`, `tree` y `wget` permanecen en la base y no se duplican. `android-tools` se deriva al elegir `scrcpy`.

Polyglot dejó de necesitar Flatpak en esta reimplementación porque `polyglot` se resuelve directamente en Nixpkgs estable e inestable. Flathub puede seguir aportando su ficha AppStream sin decidir la fuente de instalación.

LocalSend deriva su módulo y firewall. OBS deriva plugins, `v4l2loopback` y cámara virtual. Steam vive una sola vez en `[steam]` y deriva GameMode, Remote Play y servidor dedicado.

Durante este bloque se completaron también dos consecuencias que la implementación anterior ya tenía comprobadas: Sunshine añade `input`/`uinput` y virtualización añade `libvirtd`/`kvm`.

La decisión visual para la futura GUI queda fijada: AppStream es la fuente preferida de fichas; Flathub puede enriquecer aplicaciones instaladas desde Nixpkgs; el resultado se cachea localmente y una ficha ausente nunca limita lo instalable.

Integraciones especiales migradas:

```text
2e8f905d5bbd38d677bc6dbc4c072d10fbe277f5
```

La selección humana del equipo contiene ahora 37 aplicaciones en `[aplicaciones].instaladas`, más Steam como función granular independiente. Las cinco elecciones que faltaban ya conservan sus consecuencias sin exponer detalles técnicos en TOML:

```text
Cohesion          → Flathub
Figma             → Figma Linux Next 0.17.0
Genshin Impact    → AAGL
Honkai: Star Rail → AAGL
Spotify           → Spicetify + Wayland
```

Cohesion queda declarada para instalarse al aplicar la generación, pero la actualización automática de `nix-flatpak` permanece apagada para no saltarse el gestor de actualizaciones de Korunix.

AAGL usa la familia correspondiente al canal y añade su caché firmada únicamente cuando alguno de sus juegos está elegido. Figma conserva el handler `figma://`. Spotify conserva las extensiones funcionales actuales; la sincronización de tema con Noctalia pertenece al frente de apariencia.

La candidata completa de este bloque se construyó sin activarse:

```text
/nix/store/isk94y8bp0z0zpw3xvd64zcwy82qzj0p-nixos-system-korunix-26.11.20260831.34ab990
```

El sistema activo y el perfil persistente permanecieron intactos. El canal estable también produjo una derivación válida con las cinco integraciones.

La primera pasada de esa auditoría encontró cuatro decisiones humanas que todavía no estaban expresadas en `desde-cero`: apariencia, Bluetooth, avatar e identidad SSH de GitHub. Se revisó el estado vivo antes de migrarlas para no restaurar valores viejos por accidente.

El estado vivo confirmó:

```text
apariencia efectiva → Dinámico + Automático
Bluetooth           → activo
avatar              → coincide con koru.jpg de pruebas
GitHub              → ~/.ssh/blep sigue siendo la identidad efectiva
```

Estas decisiones quedaron migradas en:

```text
c533d474df92e7fffa8b200e904e697a1efc42c6
```

La candidata corregida se construyó sin activarse:

```text
/nix/store/zhys4vx6c4yc923wydzsw6fv3kfvfvd5-nixos-system-korunix-26.11.20260831.34ab990
```

Bluetooth ya no depende de que Niri o Hyprland estén instalados. El avatar prepara `~/.face` y conserva un archivo manual. La clave privada de GitHub no entra al repositorio. Noctalia recibe `wallpaper + auto` en las propiedades que Korunix administra y conserva el resto.

El sistema activo y el perfil persistente permanecieron intactos.

Antes de ejecutar esa auditoría se encontró una última diferencia entre "generación construida" y "preview reproducible": Cohesion estaba declarada por AppID pero no por commit de Flatpak.

Se comprobó que la instalación activa y Flathub coincidían exactamente en:

```text
a476d7d1dbee231266f9e904d878ec931bcafe6e37b14191430b5feb1d3da21e
```

Ese commit quedó fijado en:

```text
f284723b0009d80eb026b66dd60ab5e10285a656
```

La nueva candidata, todavía sin activar, es:

```text
/nix/store/rcy231mrvzz1ri6wbasqbyk45ig798ib-nixos-system-korunix-26.11.20260831.34ab990
```

El TOML humano no cambió. Cohesion sigue siendo una sola elección humana y el pin queda como detalle técnico derivado. El sistema activo y el perfil persistente permanecieron intactos.

La auditoría integral encontró una regresión concreta antes del primer preview: IBus y su frontend Wayland seguían activos, pero `XMODIFIERS=@im=ibus` no estaba materializado en `environment.sessionVariables`.

Se recuperó sin volver a forzar módulos GTK/Qt y sin acoplarlo a Niri:

```text
9573c32f32928507f1e2ef404670f48f1c53e649
```

La candidata corregida, todavía sin activar, es:

```text
/nix/store/lviixrls6k95c3fnrsc9qhjb2z6q4w25-nixos-system-korunix-26.11.20260831.34ab990
```

`XMODIFIERS=@im=ibus` permanece disponible también con Plasma/Cinnamon, mientras `NIRI_CONFIG` y Noctalia continúan aislados a los escritorios que les corresponden. El sistema activo y el perfil persistente permanecieron intactos.

La auditoría integral llegó al final funcionalmente, pero su comparación con el sistema activo reveló consecuencias de escritorio que todavía no estaban migradas: roles visuales de Noctalia, Valent/KDE Connect, aislamiento de Blueman/NetworkManager, xpadneo y parte del comportamiento de arranque.

También se detectó que la candidata había dejado de usar `linuxPackages_latest`, lo que explicaba la bajada de kernel visible en `diff-closures`.

El bloque se recuperó de forma plana en `sistema.nix`, sin traer de vuelta la arquitectura de roles anterior:

```text
442d6d1ae5ad6fd050ba9adfb7dadc9b5c33c327
```

La candidata corregida, todavía sin activar, es:

```text
/nix/store/a2idnnbx04xqbkidjbbxy6wahx6l5zc4-nixos-system-korunix-26.11.20260831.34ab990
```

El TOML humano no ganó nuevas preguntas. Los escritorios vuelven a derivar sus herramientas y su visibilidad, Bluetooth deriva xpadneo y el arranque recupera kernel latest + Plymouth + la vía de recuperación de 5 segundos.

La auditoría integral V6 de esa candidata llegó al final sin encontrar otra omisión funcional. Reprodujo la misma generación, volvió a comprobar decisiones humanas, hardware, escritorios, aislamiento visual, servicios, permisos, puertos, las 37 aplicaciones más Steam y las integraciones especiales. El sistema activo y el perfil persistente permanecieron intactos.

Con esa puerta cerrada se implementó el primer preview aplicable. El corte corregido es:

```text
ccb3ff02573e48f336478bd64c17668a16680055
```

`korunix preview` valida primero la configuración y el plan, construye una generación NixOS completa y conserva exactamente esa generación en un enlace estable dentro del estado local. Si ya existe un preview válido, el nuevo se construye en un enlace aparte y solo lo reemplaza cuando termina correctamente. Un fallo conserva el último preview válido.

La vía temporal anterior `korunix construir` se retiró para no mantener dos caminos con significados distintos. Preview es el único paso que crea una generación revisable antes de apply.

El primer preview aplicable quedó en:

```text
/nix/store/1cwvxaf0db189h3dq4r7r1p19ipi7a0a-nixos-system-korunix-26.11.20260831.34ab990
```

y sigue protegido por:

```text
~/.local/state/korunix/preview
```

La validación del corte terminó con 65 tests de Rust aprobados, incluidos tres comportamientos propios de preview: crear la primera generación exacta, conservar la anterior ante un fallo y reemplazarla solo cuando la nueva construcción termina bien.

La comparación fuerte del closure contra la candidata integral V6 mostró 20 paths distintos por lado. Son las mismas 20 piezas reconstruidas con hashes nuevos —Korunix, `system-path`, `etc`, completados de Fish y las unidades derivadas de systemd— porque el binario de Korunix forma parte de la generación. No desapareció ningún paquete, servicio o capacidad funcional de la candidata auditada.

El primer apply real se hizo **sin reconstruir**: se conservó como regreso la generación anterior, se ejecutó `check` y `dry-activate`, se mostró el efecto y la necesidad de reinicio, y después de autorización se puso exactamente el preview revisado como perfil persistente y se activó esa misma generación.

La generación anterior quedó protegida en:

```text
~/.local/state/korunix/anterior
→ /nix/store/5h1pm4w6ji6vz5bab5rbp0l5c51bmi1m-nixos-system-korunix-26.11.20260813.0e251e2
```

El apply terminó con:

```text
activa      = /nix/store/1cwvxaf0db189h3dq4r7r1p19ipi7a0a-nixos-system-korunix-26.11.20260831.34ab990
persistente = /nix/store/1cwvxaf0db189h3dq4r7r1p19ipi7a0a-nixos-system-korunix-26.11.20260831.34ab990
```

No aparecieron unidades systemd fallidas nuevas. Como el cambio llevaba el kernel de `7.1.8` a `7.2.2`, se reinició para cerrar la prueba de persistencia. Tras arrancar de nuevo, `/run/booted-system`, `/run/current-system` y `/nix/var/nix/profiles/system` coincidieron con el mismo preview, `uname -r` mostró `7.2.2`, los servicios básicos revisados siguieron activos y la sesión Niri conservó Noctalia, Sunshine e IBus.

El apply permanente quedó incorporado en Rust en:

```text
8aa08f9095bd3f3e443f73ee4e1f1b93b2888276
```

`korunix preview` guarda la generación exacta y la copia de `configuracion.toml` con la que se construyó. El preview nuevo se mantiene protegido durante la construcción con un enlace temporal y el enlace estable se registra explícitamente como raíz de GC.

La primera prueba de desarrollo encontró antes de tocar NixOS un detalle importante: resolver con `canonicalize` el enlace `nix-store` lo convertía en el ejecutable multicall `nix`, cambiando el significado del comando. Se corrigió conservando el nombre del ejecutable y quedó cubierto por un test específico.

La prueba válida del comando permanente pasó 72 tests de Rust y construyó un preview nuevo:

```text
/nix/store/yc3bwpvcsdlxsz6d6b5pjcr695ysxy15-nixos-system-korunix-26.11.20260831.34ab990
```

`korunix aplicar` comprobó ese preview, protegió como `anterior` la generación activa previa, ejecutó `check` y `dry-activate`, esperó la autorización humana y activó exactamente la misma generación sin ejecutar `nix build`. El cierre en vivo quedó con:

```text
activa      = /nix/store/yc3bwpvcsdlxsz6d6b5pjcr695ysxy15-nixos-system-korunix-26.11.20260831.34ab990
persistente = /nix/store/yc3bwpvcsdlxsz6d6b5pjcr695ysxy15-nixos-system-korunix-26.11.20260831.34ab990
anterior    = /nix/store/1cwvxaf0db189h3dq4r7r1p19ipi7a0a-nixos-system-korunix-26.11.20260831.34ab990
```

No aparecieron unidades systemd fallidas nuevas. El kernel siguió siendo `7.2.2`, por lo que este cambio no necesitó otro reinicio. Una segunda ejecución del `korunix aplicar` instalado en la generación nueva reconoció que el preview ya estaba activo y persistente, no pidió privilegios y no cambió nada.

Rollback quedó incorporado y probado de forma real en:

```text
9e8b9e467d89025419a60c7c0f475a63c1f24708
```

El preview que contenía rollback fue:

```text
/nix/store/4pyq6xbw7vhl6pbmqvk8c7rkddrz2qs5-nixos-system-korunix-26.11.20260831.34ab990
```

y se probó contra la generación anterior protegida:

```text
/nix/store/yc3bwpvcsdlxsz6d6b5pjcr695ysxy15-nixos-system-korunix-26.11.20260831.34ab990
```

La puerta de Rust llegó a 78 tests aprobados antes de la prueba viva. Después se aplicó exactamente el preview nuevo sin rebuild, se comprobó `activa = persistente = preview` y que la generación de origen quedara protegida junto con su copia humana de `configuracion.toml`.

`korunix rollback` ejecutó `check` y `dry-activate`, mostró el efecto y esperó `VOLVER`. Volvió exactamente a la generación anterior sin reconstruir, dejó `activa = persistente = anterior` y restauró la `configuracion.toml` asociada. Una segunda ejecución reconoció que el sistema ya estaba en ese punto y no cambió nada.

Para cerrar la prueba se reaplicó el mismo preview nuevo. El estado final quedó:

```text
activa      = /nix/store/4pyq6xbw7vhl6pbmqvk8c7rkddrz2qs5-nixos-system-korunix-26.11.20260831.34ab990
persistente = /nix/store/4pyq6xbw7vhl6pbmqvk8c7rkddrz2qs5-nixos-system-korunix-26.11.20260831.34ab990
preview     = /nix/store/4pyq6xbw7vhl6pbmqvk8c7rkddrz2qs5-nixos-system-korunix-26.11.20260831.34ab990
anterior    = /nix/store/yc3bwpvcsdlxsz6d6b5pjcr695ysxy15-nixos-system-korunix-26.11.20260831.34ab990
kernel      = 7.2.2
```

No aparecieron unidades systemd fallidas nuevas.

La recuperación de la experiencia visual completa quedó cerrada en:

```text
1356982c5c2453cf21ce1493750e9c3b677d3318
```

La generación final comprobada y aplicada exactamente fue:

```text
/nix/store/7xhk5rmhl23439qgwylmljz5n0pqcgh6-nixos-system-korunix-26.11.20260831.34ab990
```

En Niri quedaron comprobados en uso real el desenfoque, las esquinas redondeadas, Bibata, PiP, atajos, capturas localizadas, Hatter y los 19 fondos. Los cambios visuales de Noctalia pudieron recogerse sin cerrar sesión.

Steam deriva Millennium automáticamente y la plantilla comunitaria `steam` de Noctalia quedó comprobada en uso real.

Spotify sigue siendo una sola elección humana. Esa elección deriva Spicetify, Wayland y, en Niri/Hyprland, la plantilla comunitaria `spicetify`. Noctalia genera la paleta Comfy y Korunix sincroniza sus 26 colores con Spotify; la integración quedó comprobada en uso real. El bloque terminó con 80 tests de Rust aprobados.

Durante el cierre apareció una unidad transitoria de `drkonqi-coredump-processor` en estado fallido por timeout mientras DrKonqi procesaba un coredump. El diagnóstico confirmó que era el ayudante de informes de fallos de KDE, no una unidad de Korunix ni un fallo persistente del sistema. Se limpió únicamente ese estado transitorio y el cierre terminó sin unidades systemd fallidas.

Con `preview`, `aplicar` y `rollback` ya probados sobre generaciones reales y el frente visual completo nuevamente funcional, el siguiente frente de la reimplementación es llevar estos mismos flujos a GTK4/libadwaita sin duplicar la lógica: la interfaz muestra y pregunta; el mismo Rust sigue haciendo el trabajo.

## 22. Regla final

Si hay que elegir entre:

> se ve muy profesional

y:

> lo entiende una persona normal

se elige lo segundo.

Si hay que elegir entre otra capa de arquitectura y unas pocas líneas claras que hacen exactamente lo necesario, se prefieren las líneas claras mientras sigan siendo seguras y mantenibles.

El código no tiene que demostrar que sabemos programar. Tiene que ayudar a una persona a usar NixOS.

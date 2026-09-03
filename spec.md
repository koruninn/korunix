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

Preview no modifica el sistema. Cuando exista, tiene que representar una generación completa y concreta.

Apply activa exactamente la generación revisada, la deja persistente para el siguiente arranque y lo verifica. La generación activa y la persistente deben coincidir al terminar correctamente.

Rollback es una función normal del producto. Una operación lógica cruza la frontera de privilegios el menor número de veces posible. Las operaciones largas muestran su fase y no dejan la interfaz muda.

## 11. Aplicaciones

El catálogo curado sirve para nombre bonito, descripción, categoría, opciones especiales e integración adicional. No limita lo instalable.

Una aplicación elegida por la persona debe seguir visible aunque no tenga ficha curada.

Korunix intenta resolver primero una selección humana sencilla, por ejemplo:

```toml
"karere"
```

Si puede resolverse de forma fiable en Nixpkgs, se instala sin obligar a escribir una ruta técnica. Flatpak puede servir como segunda fuente cuando corresponda.

Las dependencias internas que la persona no eligió no deben convertirse automáticamente en aplicaciones visibles.

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

Las plantillas visuales de Noctalia no deben contaminar Plasma o Cinnamon.

Elegir Niri o Hyprland deriva Noctalia como parte de esa familia de escritorio. Cinnamon y Plasma no reciben su servicio ni su configuración. En el primer corte de `desde-cero`, Niri usa una configuración KDL pequeña y explícita con launcher, controles, bloqueo, capturas, terminal, archivos y navegación básica.

Noctalia conserva sus valores predeterminados y las preferencias cambiadas desde su interfaz. Korunix no reemplaza un `~/.config/noctalia/config.toml` existente: fusiona únicamente la política que administra. La misma política se aplica a `~/.local/state/noctalia/settings.toml` cuando ese archivo existe.

Las capturas usan el directorio XDG de imágenes más `Capturas de pantalla` y el patrón `Captura de pantalla del %Y-%m-%d %H-%M-%S`.

NixOS 26.05 todavía no trae Noctalia. Mientras siga así, el canal estable toma solo el paquete Noctalia del input `nixpkgs-inestable` que el flake ya tiene; el resto del sistema continúa usando el canal estable. No se añade un tercer input para resolver esta excepción.

## 13. Servicios y funciones granulares

Funciones como Steam y Sunshine permiten opciones internas sin convertir cada detalle técnico en una pregunta.

Puertos y permisos se derivan de la función. El firewall permanece activo. Un puerto solo se abre cuando una función que realmente lo necesita está activa.

SSH forma parte permanente de la base de Korunix y abre únicamente su regla en el firewall. Avahi también forma parte de la base para el descubrimiento local.

Flatpak y AppImage son capacidades del sistema aunque en ese momento no haya ninguna aplicación elegida desde esas fuentes. Nautilus dispone de UDisks2 y GVfs para el uso cotidiano de unidades extraíbles.

Sunshine pertenece al acceso/transmisión remota y puede tener autoinicio independiente. Steam puede tener Remote Play y servidor dedicado como preferencias independientes.

El acceso remoto más amplio puede integrar Sunshine/Moonlight y Tailscale cuando se trabaje ese frente; no es requisito del primer corte desde cero.

## 14. Hardware y sistema

Korunix debe detectar antes de preguntar cuando sea fiable.

Debe contemplar x86_64, aarch64, UEFI, BIOS, portátil o sobremesa, CPU, GPU, memoria, almacenamiento, firmware, audio, micrófonos y cámaras.

La detección no debe convertirse en una excusa para llenar el arranque de procesos repetidos.

Los UUID de discos, módulos de arranque, arquitectura y otros hechos que NixOS necesita para arrancar no son preferencias humanas y no se meten en TOML. En el primer corte local de `desde-cero` viven en un `hardware.nix` plano. Korunix puede volver a detectarlos más adelante, pero no reemplaza silenciosamente un hardware ya comprobado.

No se crea una carpeta como `generado/equipos/` mientras un solo `hardware.nix` sea suficiente para entender el sistema.

La base habilita firmware redistribuible, fwupd y gráficos de 32 bits cuando la arquitectura es x86_64. Korunix controla cuándo consulta actualizaciones de firmware; el refresco automático de fwupd no se usa como sustituto del flujo de actualizaciones de Korunix.

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

La corrección más reciente de ese frente es vinculante: XKB administra las distribuciones normales y IBus es el backend normal para composición y diacríticos. En Niri y Hyprland IBus usa su frontend Wayland. `XMODIFIERS=@im=ibus` sigue disponible, pero Korunix no fuerza `GTK_IM_MODULE` ni `QT_IM_MODULE`. Fcitx5 queda reservado para métodos de entrada avanzados cuando se implementen.

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

Las cuentas locales se expresan como bloques `[[personas]]`. Cada bloque puede indicar el nombre de la cuenta, el nombre visible y si es administradora. Las contraseñas y sus hashes no se guardan en TOML ni en Git. Mientras se adopta una cuenta que ya existe, NixOS mantiene `users.mutableUsers = true` y Korunix no declara una contraseña.

## 16. Almacenamiento, copias e historial

Se conserva como comportamiento útil:

- acceso claro a discos adicionales;
- transferencias pesadas con progreso;
- porcentaje, velocidad y ETA cuando sean medibles;
- persistencia y verificación;
- evitar `sync` global innecesario;
- no presentar un archivo incompleto como terminado;
- no sobrescribir silenciosamente;
- ofrecer expulsión segura cuando corresponda;
- copias;
- historial;
- restauración;
- rollback.

## 17. Actualizaciones

Korunix administra también las actualizaciones del sistema y soporta canal estable e inestable.

La persona elige una vez y Korunix deriva los inputs y detalles relacionados.

Antes de actualizar se explica qué va a cambiar y si el resultado necesita reinicio o volver a iniciar sesión.

No inventar porcentajes si Nix no puede ofrecerlos. Sí mostrar fase y actividad.

## 18. GUI

GTK4 + libadwaita son la base visual.

La GUI muestra, pregunta, manda decisiones al mismo Rust que usa la CLI y presenta resultados. No contiene una segunda implementación del sistema.

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
- apply activa lo revisado;
- activa y persistente coinciden;
- rollback recupera;
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

La generación candidata sigue sin activarse. El siguiente bloque debe empezar la migración de esas decisiones humanas al TOML plano sin copiar la arquitectura de `pruebas`.

## 22. Regla final

Si hay que elegir entre:

> se ve muy profesional

y:

> lo entiende una persona normal

se elige lo segundo.

Si hay que elegir entre otra capa de arquitectura y unas pocas líneas claras que hacen exactamente lo necesario, se prefieren las líneas claras mientras sigan siendo seguras y mantenibles.

El código no tiene que demostrar que sabemos programar. Tiene que ayudar a una persona a usar NixOS.

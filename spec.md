# Korunix — guía viva

> Este archivo manda sobre la implementación actual.
>
> Primero se revisan las decisiones y quejas más recientes del proyecto. Después se cruza eso con este archivo. Una decisión humana reciente puede corregir una idea vieja del spec.

## 1. Qué queremos

Korunix es una capa sencilla sobre NixOS para que una persona no técnica pueda configurar, mantener, entender y recuperar su sistema sin aprender toda la parte interna de NixOS.

Prioridades, en este orden práctico:

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

## 4. El modelo Lego

La idea central es:

```text
lo que la persona quiere
→ Korunix lo entiende y valida
→ Nix deriva la parte técnica
→ NixOS aplica el resultado
```

Una decisión humana se expresa una sola vez.

Ejemplos:

```toml
canal = "estable"
```

Eso debe ajustar todo lo necesario para usar el canal estable.

```toml
[escritorio]
principal = "niri"
```

Eso debe ajustar lo necesario para Niri.

```toml
[aplicaciones]
instaladas = [
  "firefox",
  "karere",
  "blender",
]
```

Eso debe instalar esas aplicaciones si pueden resolverse de forma fiable, aunque alguna no tenga ficha curada.

La persona no debe editar varios archivos para expresar una sola decisión.

## 5. TOML es la configuración humana

TOML es la entrada manual normal.

GUI, CLI y edición manual representan las mismas decisiones.

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

Apagar una subopción no apaga necesariamente la función principal.

Apagar el control principal no borra las preferencias internas. Simplemente deja de aplicarlas mientras esté apagado.

Ejemplo:

```toml
[sunshine]
activo = true
autoinicio = false
```

Sunshine sigue disponible, pero no arranca solo.

Ejemplo:

```toml
[steam]
activo = false
remote_play = true
```

`remote_play = true` se conserva como preferencia, pero no se aplica mientras Steam esté apagado.

## 6. Nombres, comentarios y textos humanos

Todo lo que una persona pueda leer debe hablar en español humano, directo y sencillo.

Esto incluye:

- nombres de archivos y opciones;
- comentarios;
- documentación;
- mensajes y errores;
- Rust;
- Nix;
- TOML;
- Niri;
- Noctalia;
- scripts;
- configuración de aplicaciones.

No usar nombres pretenciosos porque “así se programa”.

Evitar palabras como `orchestrator`, `provider`, `repository`, `facade`, `adapter`, `domain`, `materialization` y similares cuando una palabra normal explica mejor lo mismo.

Si administra aplicaciones: `aplicaciones`.

Si configura el sistema: `sistema`.

Si muestra la ventana: `interfaz`.

Si guarda lo elegido: `configuracion`.

Mal:

> Materializa el contrato declarativo del subsistema.

Bien:

> Esto guarda lo que elegiste.

No comentar lo obvio. Un comentario explica qué hace algo, por qué existe o qué pasa si se cambia.

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

Una carpeta nueva solo aparece cuando de verdad existe una colección que la necesita.

Si un archivo crece tanto que dividirlo mejora la comprensión, recién se divide.

No crear carpetas profundas ni capas preventivas “por buenas prácticas”.

## 8. Rust y Nix no compiten

Rust administra decisiones, estado, validación, flujo, errores, GUI y CLI.

Nix sigue siendo quien configura NixOS.

La GUI no implementa NixOS por su cuenta. La CLI tampoco.

```text
GUI → mismo Rust → Nix → NixOS
CLI → mismo Rust → Nix → NixOS
```

No escribir una segunda lógica para la GUI.

No usar Rust para reinventar lo que NixOS ya hace bien.

## 9. Rapidez

Korunix debe sentirse inmediato.

Usar Rust no basta. El programa debe evitar trabajo inútil.

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

Las tareas lentas no pueden congelar GTK.

Las páginas no deben serializarse detrás de una única espera global.

Las operaciones remotas, firmware, actualizaciones y otras lecturas lentas trabajan aparte cuando sea posible.

Korunix es offline-first: si una operación puede resolverse con datos locales, Internet no debe ser requisito.

## 10. Errores humanos y seguridad

Un typo no rompe el sistema.

Si alguien pone:

```toml
principal = "niry"
```

Korunix debe explicar el problema, sugerir `niri` cuando sea razonable y conservar el último estado válido.

Antes de un cambio importante:

- mostrar qué cambia;
- explicar el efecto;
- decir si necesita reinicio o volver a iniciar sesión;
- explicar si necesita autorización;
- pedir privilegios solo cuando hagan falta.

Preview no modifica el sistema.

Apply activa exactamente la generación revisada, la deja persistente para el siguiente arranque y lo verifica.

La generación activa y la persistente deben coincidir al terminar correctamente.

Rollback es una función normal del producto.

Una operación lógica cruza la frontera de privilegios el menor número de veces posible.

Las operaciones largas muestran su fase y no dejan la interfaz muda.

## 11. Aplicaciones

El catálogo curado sirve para:

- nombre bonito;
- descripción;
- categoría;
- opciones especiales;
- integración adicional.

No limita lo instalable.

Una aplicación elegida por la persona debe seguir visible aunque no tenga ficha curada.

Korunix intenta resolver primero una selección humana sencilla, por ejemplo:

```toml
"karere"
```

Si puede resolverse de forma fiable en Nixpkgs, se instala sin obligar a escribir una ruta técnica.

Flatpak puede servir como segunda fuente cuando corresponda.

Las dependencias internas que la persona no eligió no deben convertirse automáticamente en aplicaciones visibles.

## 12. Escritorios y apariencia

Los escritorios soportados son:

- Niri;
- Hyprland;
- Cinnamon;
- KDE Plasma.

GNOME puede aportar aplicaciones o integraciones, pero no es un escritorio soportado de Korunix.

Estas ideas son distintas:

- escritorio instalado;
- escritorio principal;
- escritorio usado para vista previa;
- compatibilidad de un estilo.

No se deben acoplar.

Niri y Hyprland comparten la familia Noctalia.

Plasma y Cinnamon conservan su apariencia nativa o neutral cuando una integración equivalente no existe.

Tener Plasma o Cinnamon instalados no bloquea Dinámico o Everforest para Niri/Hyprland.

Los ejes de apariencia son distintos:

```text
Predeterminado / Dinámico / Everforest
```

y:

```text
Claro / Oscuro / Automático
```

Las plantillas visuales de Noctalia no deben contaminar Plasma o Cinnamon.

## 13. Servicios y funciones granulares

Funciones como Steam y Sunshine deben permitir opciones internas sin convertir cada detalle técnico en una pregunta.

Puertos y permisos se derivan de la función.

El firewall permanece activo.

Un puerto solo se abre cuando una función que realmente lo necesita está activa.

SSH sigue siendo una capacidad prevista del producto.

Sunshine pertenece al acceso/transmisión remota y puede tener autoinicio independiente.

Steam puede tener Remote Play y servidor dedicado como preferencias independientes.

El acceso remoto más amplio puede integrar Sunshine/Moonlight y Tailscale cuando se trabaje ese frente; no es requisito del primer corte desde cero.

## 14. Hardware y sistema

Korunix debe detectar antes de preguntar cuando sea fiable.

Debe contemplar:

- x86_64;
- aarch64;
- UEFI;
- BIOS;
- portátil o sobremesa;
- CPU;
- GPU;
- memoria;
- almacenamiento;
- firmware;
- dispositivos de audio;
- micrófonos;
- cámaras.

La detección no debe convertirse en una excusa para llenar el arranque de procesos repetidos.

## 15. Idioma, teclado y personas

Korunix debe mantener separadas las decisiones de:

- idioma de la interfaz;
- idiomas preferidos;
- región;
- formatos;
- zona horaria;
- teclados;
- variantes;
- métodos de entrada.

Los nombres visibles deben ser humanos. Los identificadores técnicos quedan debajo.

Se conserva como referencia el trabajo comprobado con IBus y composición Wayland para Niri/Hyprland, pero la nueva implementación debe buscar la forma más simple de cumplir el comportamiento.

Personas debe permitir gestionar usuarios y preferencias sin pedir rutas o identificadores técnicos cuando una selección gráfica pueda resolverlo.

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

Korunix administra también las actualizaciones del sistema.

Debe soportar:

- canal estable;
- canal inestable.

La persona elige una vez y Korunix deriva los inputs y detalles relacionados.

Antes de actualizar se explica qué va a cambiar y si el resultado necesita reinicio o volver a iniciar sesión.

No inventar porcentajes si Nix no puede ofrecerlos. Sí mostrar fase y actividad.

## 18. GUI

GTK4 + libadwaita son la base visual.

La GUI:

- muestra;
- pregunta;
- manda decisiones al mismo Rust que usa la CLI;
- presenta resultados;
- no contiene una segunda implementación del sistema.

Debe respetar:

- navegación por teclado;
- foco visible;
- lectores de pantalla;
- escalado de texto;
- contraste;
- traducciones largas;
- diacríticos;
- CJK;
- preparación para RTL;
- modo compacto;
- ausencia de clipping.

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
- una lectura normal no reevaluá Nix sin necesidad;
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

## 21. Primer objetivo de `desde-cero`

No intentar reconstruir todo Korunix de golpe.

Primero hacer una base pequeña que pueda:

1. leer `configuracion.toml`;
2. validarlo;
3. mostrar el estado;
4. añadir o quitar una aplicación;
5. cambiar una preferencia sencilla;
6. generar un plan;
7. previsualizar sin modificar;
8. aplicar exactamente lo revisado;
9. verificar la generación activa y persistente;
10. hacer rollback;
11. ofrecer la misma lógica por CLI y una GUI mínima.

Cuando eso sea rápido, claro y seguro, se amplía.

## 22. Regla final

Si hay que elegir entre:

> se ve muy profesional

y:

> lo entiende una persona normal

se elige lo segundo.

Si hay que elegir entre otra capa de arquitectura y unas pocas líneas claras que hacen exactamente lo necesario, se prefieren las líneas claras mientras sigan siendo seguras y mantenibles.

El código no tiene que demostrar que sabemos programar. Tiene que ayudar a una persona a usar NixOS.

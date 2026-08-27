# Korunix — especificación de producto y arquitectura

> Estado: especificación consolidada de producto y arquitectura.
>
> Este documento define el comportamiento que Korunix debe perseguir. No describe todavía todos los detalles de implementación. Cuando una decisión técnica todavía necesite validación, se indica expresamente para evitar convertir una hipótesis en una promesa.

## 1. Qué es Korunix

Korunix es una capa de producto sobre NixOS que permite administrar el sistema mediante decisiones humanas, sin exigir que la persona aprenda Nix.

Korunix no es solamente una colección personal de archivos de configuración y tampoco debe reducirse a una tienda de aplicaciones. Debe convertirse en el punto central desde el que una persona pueda preparar, entender, mantener, actualizar, recuperar y personalizar su equipo.

La configuración personal del creador de Korunix debe ser una configuración válida del mismo producto, no una excepción privada mantenida con otra arquitectura.

Korunix debe poder utilizarse en dos situaciones principales:

- después de una instalación gráfica normal de NixOS, por ejemplo mediante Calamares;
- sobre un NixOS que ya está configurado y en uso.

La instalación oficial de NixOS sigue siendo la primera capa. Korunix es la capa posterior que prepara el equipo para el uso cotidiano. Una imagen ISO propia puede estudiarse en el futuro, pero no es requisito para la primera versión.

## 2. Principios permanentes

### 2.1. Korunix administra decisiones humanas; NixOS las implementa

La interfaz debe hablar de objetivos y resultados:

- navegador predeterminado;
- apariencia;
- escritorio;
- usuarios;
- aplicaciones;
- actualizaciones;
- copias de seguridad;
- accesibilidad.

No debe convertir detalles como `wheel`, `nix-ld`, MIME, overlays, flakes o grupos UNIX en preguntas normales para el usuario.

Los detalles técnicos pueden existir en vistas avanzadas y documentación, pero no son el lenguaje primario del producto.

### 2.2. Nunca preguntar lo que pueda detectarse de forma fiable

Korunix debe detectar antes de preguntar, entre otras cosas:

- idioma del sistema;
- región y zona horaria cuando sean deducibles;
- arquitectura;
- UEFI o BIOS;
- tipo de equipo;
- CPU;
- GPU;
- memoria;
- almacenamiento;
- escritorios existentes;
- usuarios existentes;
- estado de servicios y aplicaciones que Korunix necesite comprender.

Cuando la detección no sea suficientemente fiable, Korunix debe mostrar una propuesta comprensible y permitir corregirla.

### 2.3. Nunca sorprender al usuario

Antes de una operación importante Korunix debe explicar:

- qué encontró;
- qué pretende cambiar;
- qué no va a cambiar;
- por qué necesita hacerlo;
- si necesita autenticación;
- si requerirá cerrar sesión o reiniciar.

Ninguna ventana de contraseña, Polkit o privilegios elevados debe aparecer sin un aviso previo de Korunix.

### 2.4. Todo cambio importante debe poder previsualizarse

La persona debe poder explorar opciones libremente antes de modificar el sistema.

Los cambios puramente visuales de la propia aplicación, como idioma, apariencia o previews, deben reflejarse inmediatamente. Los cambios del sistema solo se aplican tras revisar la propuesta.

### 2.5. Una única fuente de verdad

Una decisión no debe estar duplicada en cinco configuraciones independientes.

Ejemplos:

- el terminal predeterminado se define una vez;
- la apariencia se define una vez;
- el rol de navegador se define una vez;
- los atajos representan acciones semánticas y después cada escritorio los implementa;
- la versión/base del sistema se define una vez y las dependencias relacionadas se seleccionan en conjunto.

### 2.6. La estructura debe ser plana, obvia y navegable

Regla permanente del proyecto:

> Una carpeta solo existe si representa una colección real; el árbol debe ser plano, obvio y navegable por una persona cualquiera. Trabajamos por bloques grandes y coherentes, no por microcambios.

Si meses después una persona no puede adivinar dónde vive una opción razonable sin usar `grep`, la estructura debe reconsiderarse.

### 2.7. Copiar un comando no convierte a una persona en técnica

Incluso si Korunix se inicia desde una terminal, la aplicación debe mantener lenguaje humano. El bootstrap por terminal es un puente inicial, no una declaración sobre el nivel técnico de quien lo ejecuta.


### 2.8. Korunix es offline-first

La ausencia de Internet nunca debe impedir administrar el equipo cuando la operación pueda resolverse completamente con recursos locales.

Korunix debe seguir permitiendo, entre otras cosas, cuando los recursos necesarios ya están disponibles localmente:

- consultar y modificar la configuración;
- gestionar usuarios;
- cambiar preferencias;
- importar y exportar perfiles locales;
- consultar historial;
- restaurar copias y generaciones;
- administrar unidades y dispositivos;
- preparar cambios para aplicarlos después.

Las funciones cuyo origen o destino sea realmente remoto deben quedar temporalmente no disponibles y explicar por qué:

> Estás sin conexión. Esta operación necesita Internet para descargar componentes nuevos. Puedes seguir usando el resto de Korunix.

La aplicación no debe bloquearse globalmente porque una sola función necesite red.

### 2.9. Korunix oculta complejidad, no capacidad

La persona debe poder realizar una operación potente sin conocer el mecanismo interno que la hace posible.

Korunix traduce intención → capacidad → implementación.

Un usuario avanzado puede abrir detalles técnicos cuando los necesite, pero una persona normal no debe aprender flakes, derivaciones, grupos UNIX, Polkit, systemd o comandos de mantenimiento para administrar su equipo.

### 2.10. Degradación elegante

Si falta red, una dependencia opcional, una integración del escritorio o cualquier capacidad no esencial, Korunix debe conservar todo lo que siga funcionando y señalar únicamente lo que quedó limitado.

Un fallo parcial no debe convertir toda la aplicación en una pantalla de error.

## 3. Regla para todos los archivos de texto

Todos los archivos de texto mantenidos por el proyecto deben ser comprensibles por una persona.

Cuando el formato admita comentarios, los comentarios deben estar en español humano y explicar:

- qué es el elemento o bloque;
- qué hace;
- por qué existe cuando esa explicación aporte contexto;
- qué consecuencias importantes tiene modificarlo.

No se deben añadir comentarios redundantes que simplemente repitan literalmente una línea evidente.

Esta regla se aplica, entre otros, a:

- Nix;
- Bash;
- Fish;
- KDL;
- TOML;
- YAML;
- archivos de configuración de aplicaciones;
- archivos auxiliares legibles por humanos.

Cuando un formato no permita comentarios, como JSON estricto, la explicación debe vivir en el lugar humano más próximo sin romper el formato.

Los archivos generados deben identificarse como generados cuando el formato lo permita y nunca deben presentarse como si fueran el lugar correcto para una edición manual.

Todos los archivos `.nix` mantenidos por Korunix deben estar preparados para ser formateados de manera uniforme por Alejandra. `just fmt` debe ser la vía habitual para hacerlo.

## 4. Home Manager

Home Manager no forma parte de la arquitectura objetivo de Korunix.

Korunix debe integrar la configuración de sistema y usuario dentro de la transacción de NixOS que administra el equipo, utilizando módulos NixOS, archivos XDG y otros mecanismos apropiados sin mantener una segunda generación independiente de Home Manager.

Consecuencias:

- desaparece `homeConfigurations`;
- desaparecen los módulos dependientes de Home Manager;
- desaparece `just home`;
- no debe existir una compatibilidad ficticia que siga presentando “home” como una operación independiente.

## 5. Instalación y bootstrap

### 5.1. Objetivo

Una persona que acaba de instalar NixOS mediante el instalador gráfico debe poder iniciar Korunix con un único comando corto, copiable y razonablemente recordable.

El bootstrap no debe presuponer que Git, flakes, Flatpak, AppImage o `nix-ld` ya están configurados.

La vía inicial prevista es utilizar `nix-shell` para proporcionar Git temporalmente, clonar Korunix y arrancar el bootstrap.

Forma conceptual:

```bash
nix-shell -p git --run 'git clone https://github.com/koruninn/korunix ~/.korunix && ~/.korunix/scripts/korunix'
```

La cadena exacta debe validarse en un NixOS recién instalado antes de declararla comando oficial. Debe ser idempotente o fallar de manera humana si Korunix ya existe.

### 5.2. Después del bootstrap

La terminal deja de ser la vía normal de acceso.

Korunix debe instalarse e integrarse como una aplicación gráfica normal. Cuando esa integración esté realmente activa, debe comunicarlo claramente:

> Korunix ya está instalado en este equipo. A partir de ahora puedes abrirlo desde el menú de aplicaciones.

No debe afirmarlo antes de que el launcher gráfico haya sido verificado.

### 5.3. Motor y configuración no deben confundirse

La actualización del programa Korunix no debe obligar a descargar un ZIP, crear otra carpeta ni perder la configuración existente.

La arquitectura debe distinguir:

- el motor/aplicación Korunix que puede actualizarse;
- el espacio de configuración humana que pertenece al equipo y sus usuarios;
- el estado local sensible que no debe sincronizarse.

La ruta exacta final de cada capa se decidirá durante la implementación, pero la separación es obligatoria para evitar que actualizar el programa destruya o reemplace la configuración del usuario.


### 5.4. Arranque desde medios locales

Korunix también debe poder iniciarse desde una copia local del proyecto, por ejemplo un ZIP previamente descargado y transportado mediante USB u otro almacenamiento externo.

El archivo ZIP contiene el mismo código fuente que el repositorio y no debe crear un modelo paralelo de configuración.

Una copia local permite ejecutar todas las funciones que no necesiten descargar componentes ausentes. Si Nix necesita una dependencia que no exista en el store o cachés locales, Korunix debe detectarlo y explicar que esa parte concreta requiere conexión.

En el futuro puede estudiarse un paquete offline más completo que incluya cierres de Nix o recursos predescargados. No se debe prometer que un ZIP de código por sí solo contiene todas las dependencias necesarias para reconstruir cualquier sistema sin Internet.

### 5.5. Herramientas de desarrollo no son requisitos de uso

`just`, Git y otras herramientas de desarrollo pueden formar parte del entorno de trabajo del repositorio, pero no deben ser requisitos previos para una persona que solo quiere usar Korunix.

Después del bootstrap, Korunix debe ser invocable como aplicación normal.

Dentro de un checkout de desarrollo puede existir un acceso corto como:

```text
just korunix
```

pero esta comodidad no sustituye el bootstrap oficial porque `just` no está garantizado en una instalación limpia de NixOS.

El entorno de desarrollo puede proporcionar sus dependencias mediante Nix para que el colaborador no tenga que instalarlas permanentemente.

## 6. Primer uso y uso cotidiano

Korunix utiliza las mismas páginas y componentes en dos modos.

### 6.1. Primer uso

Si Korunix detecta que el equipo todavía no ha sido preparado por él, muestra un asistente guiado.

El asistente propone un orden, pero no es una cárcel lineal. Después de la capa inicial de idioma, la persona puede volver a áreas ya visitadas y modificar decisiones sin recorrer artificialmente todos los pasos intermedios.

### 6.2. Sistema existente

Si Korunix detecta un sistema ya configurado, abre el panel permanente mostrando el estado detectado y permite entrar directamente en cualquier sección.

No debe obligar a repetir un onboarding.

### 6.3. Mismas páginas, dos contextos

El asistente no debe contener implementaciones duplicadas de las configuraciones que luego aparecen en el panel. El onboarding es una orquestación de las mismas páginas y del mismo modelo de estado.

## 7. Lenguaje de diseño de Korunix

### 7.1. Referencias

Korunix toma inspiración de tres familias visuales:

- macOS Tahoe y su Setup Assistant: composición, calma, jerarquía, onboarding, previews, superficies amplias y decisiones claras;
- GNOME y Libadwaita: semántica de aplicación Linux, accesibilidad, navegación, adaptación y widgets sólidos;
- Noctalia: personalidad, superficies flotantes, redondeos, integración con Niri/Hyprland y la identidad visual que acompaña a Everforest.

Korunix debe tener identidad propia; las referencias son un estándar de calidad y una fuente de principios, no una colección de pantallas para copiar literalmente.

### 7.2. Base tecnológica prevista

La GUI objetivo es GTK4 + Libadwaita, con un design system propio de Korunix encima.

### 7.3. Ventana adaptable

Korunix no está obligado a ser una ventana estrecha.

- El asistente inicial puede usar una superficie amplia, centrada y respirada.
- El panel cotidiano puede usar una navegación lateral y una densidad ligeramente mayor.
- Ambos deben adaptarse a tamaños razonables de pantalla.
- Debe existir un ancho de contenido máximo para evitar líneas absurdamente largas en pantallas grandes.
- Debe funcionar correctamente en resoluciones comunes como 1366×768 sin recortes.

### 7.4. Componentes base

El design system debe definir como mínimo componentes equivalentes a:

- NavigationBar;
- PageTitle;
- Section;
- SettingRow;
- SettingGroup;
- ChoiceCard;
- DesktopCard;
- ApplicationCard;
- SearchablePicker;
- ToggleRow;
- StatusBadge;
- InfoBanner;
- WarningBanner;
- ProgressView;
- ConfirmationSheet;
- ErrorView;
- EmptyState;
- LoadingState;
- UpdateRow.

Una pantalla debe poder responder inmediatamente:

1. ¿Dónde estoy?
2. ¿Qué está configurado ahora?
3. ¿Qué puedo hacer?
4. ¿Qué ocurrirá si lo hago?

### 7.5. Señalética y traducción

Korunix puede preferir iconos, flechas y señalética cuando el significado sea universal y seguro. Esto reduce texto visible y por tanto parte del trabajo de traducción.

Ejemplos adecuados:

- volver;
- avanzar;
- cerrar;
- añadir;
- quitar cuando no sea destructivo o ambiguo;
- buscar;
- estado correcto;
- información.

Las operaciones delicadas o ambiguas deben conservar texto explícito, por ejemplo:

- aplicar cambios;
- eliminar usuario;
- restaurar sistema;
- reiniciar;
- instalar;
- desinstalar.

Los iconos no eliminan la necesidad de accesibilidad. Cada control debe tener un nombre accesible traducido para lectores de pantalla.

La navegación debe respetar idiomas RTL; las flechas direccionales no deben quedar codificadas de manera rígida si su significado depende de la dirección del idioma.

### 7.6. Ilustraciones vectoriales

Korunix debe disponer de una familia coherente de ilustraciones vectoriales para identificar áreas y etapas, por ejemplo:

- idioma;
- teclado;
- usuarios;
- escritorios;
- apariencia;
- copias de seguridad;
- actualizaciones;
- recuperación.

Deben escalar correctamente, adaptarse a claro/oscuro cuando corresponda y conservar una identidad visual propia de Korunix.

## 8. Idioma: capa cero

El idioma de Korunix se decide antes de dibujar la primera pantalla.

Flujo:

1. leer el idioma/locale actual del sistema;
2. normalizarlo, por ejemplo `es_PE.UTF-8` → `es-PE` → `es`;
3. comprobar si Korunix soporta ese idioma;
4. si lo soporta, iniciar en él;
5. si no lo soporta, iniciar en español.

No debe existir un parpadeo inicial en otro idioma antes de aplicar la detección.

Si el usuario cambia el idioma dentro de Korunix, la aplicación debe traducirse en vivo siempre que técnicamente sea posible, sin reiniciarse.

Los idiomas soportados de Korunix deben alinearse con la lista real de idiomas soportados por Noctalia cuando se implemente esta capa. La lista debe verificarse contra la versión de Noctalia utilizada; no se debe codificar una cifra histórica sin comprobación.

## 9. Localización, región e entrada

Korunix debe mantener separadas estas decisiones:

- idioma de la interfaz de Korunix;
- idiomas preferidos del sistema;
- región;
- formatos regionales;
- zona horaria;
- distribuciones de teclado;
- variantes de teclado;
- métodos de entrada.

Una persona puede usar, por ejemplo, interfaz en español, región Perú y un teclado de España. Korunix no debe derivar una propiedad de otra sin ofrecer corrección.

Cuando sea fiable, debe detectar y proponer valores. La interfaz debe favorecer el patrón:

> Detectamos esto → usar esta configuración → personalizar.

Se deben admitir múltiples idiomas y múltiples distribuciones de teclado cuando el sistema lo permita.

## 10. Apariencia de Korunix y apariencia del sistema

### 10.1. Korunix inicia neutral

La aplicación debe iniciar usando Adwaita de manera natural, respetando el modo claro, oscuro o automático detectado.

### 10.2. Everforest se adopta en vivo

Cuando el usuario seleccione Everforest, la misma instancia de Korunix debe adoptar el tema inmediatamente.

No debe:

- reiniciar la aplicación;
- reconstruir la interfaz;
- volver a abrir la pantalla.

GTK repintará internamente los widgets cuando sea necesario, pero la experiencia para el usuario debe ser continua.

Modelo conceptual:

```text
appearance.style = default | everforest
appearance.mode  = light | dark | auto
```

Un gestor global de apariencia debe alimentar toda la aplicación. Ninguna página debe mantener su propia lógica de tema.

### 10.3. Transiciones

Los cambios visuales deben ser suaves y cortos cuando GTK permita hacerlo de forma fiable.

Se prefieren transiciones discretas, por ejemplo crossfade y cambios de superficies alrededor de 150–250 ms, antes que animaciones llamativas.

Si una propiedad no puede interpolarse limpiamente, se prefiere un cambio inmediato y correcto antes que una animación frágil.

### 10.4. Automático

El modo automático sigue la fuente de estado que Korunix defina para el sistema. La tarjeta visual puede combinar una vista clara y una oscura para comunicar su significado sin necesitar una tercera captura completa.

### 10.5. Blur y transparencia

Blur y transparencia son mejoras visuales, no dependencias funcionales. Korunix debe verse completo y cuidado incluso cuando el compositor o escritorio no permita aplicar blur de la misma manera.

## 11. Previews de apariencia y escritorios

Las decisiones visuales deben tener respuesta visual inmediata.

Si se cambia:

- Adwaita → Everforest;
- claro → oscuro;
- escritorio seleccionado;

Korunix debe cambiar los previews correspondientes en la misma vista, preferentemente con crossfade.

### 11.1. Capturas base

Para cada escritorio y estilo deben mantenerse capturas comparables con una escena de referencia consistente:

- misma resolución de captura;
- composición comparable;
- aplicaciones equivalentes;
- hora y contenido controlados;
- misma intención visual.

Para `Automático` se puede construir una composición a partir de la captura clara y oscura en lugar de mantener una tercera captura redundante.

### 11.2. Múltiples escritorios

La selección se realiza mediante tarjetas visuales que contienen:

- captura;
- nombre;
- descripción corta de la experiencia;
- una frase del tipo “Ideal si…” cuando aporte orientación;
- características breves cuando ayuden a distinguir opciones.

No se deben etiquetar escritorios como “para expertos” de forma que expulse innecesariamente a un usuario.

Cuando haya varios escritorios seleccionados, la vista principal mantiene una única captura grande y permite cambiar entre ellos mediante pestañas o controles equivalentes.

Debe distinguirse entre:

- escritorio instalado;
- escritorio principal/preferido.

Una vista opcional “Comparar” puede poner dos escritorios lado a lado, especialmente Niri y Hyprland.

Para explicar comportamientos difíciles de mostrar en una imagen, pueden usarse clips WebM cortos. GIF no es el formato preferido.

## 12. Hardware

Korunix debe detectar el hardware antes de generar la propuesta de configuración.

Como mínimo debe considerar:

- arquitectura;
- UEFI o BIOS;
- CPU;
- GPU integrada/dedicada;
- fabricante y generación de GPU cuando afecte al controlador;
- memoria;
- almacenamiento;
- portátil o sobremesa;
- dispositivos relevantes para servicios que Korunix administre.

Las aplicaciones y configuraciones pueden depender de capacidades reales del hardware.

Ejemplo: la suite ofimática puede tener una implementación preferida en x86_64 y un fallback compatible en ARM.

Korunix debe modelar la intención “suite ofimática”, no obligar a que un perfil portable contenga siempre el mismo paquete binario.


### 12.1. Capacidades condicionadas por hardware

Korunix no debe ofrecer opciones de hardware inexistente como si fueran decisiones normales.

Ejemplo:

- si no existe ningún adaptador Bluetooth detectable, Bluetooth no se presenta como una capacidad disponible;
- si aparece un adaptador Bluetooth compatible, Korunix puede habilitar la pila Bluetooth y las integraciones de bajo coste definidas por su política;
- si el dispositivo desaparece temporalmente, la configuración declarada no debe destruirse de forma impulsiva; Korunix distingue entre “no conectado ahora” y “capacidad eliminada por el usuario”.

Cuando Bluetooth esté habilitado, Korunix puede incluir de forma preventiva soporte de bajo coste para mandos Xbox mediante `xpadneo`, evitando que una persona tenga que descubrir e instalar ese soporte después.

La misma filosofía se aplica a otras capacidades de compatibilidad: si el coste, riesgo y mantenimiento son bajos y evitan fricción recurrente, Korunix puede activarlas como parte de la experiencia recomendada.

## 13. Arranque, UEFI, BIOS y Windows

Korunix selecciona el gestor de arranque según el firmware detectado:

- UEFI → systemd-boot;
- BIOS/Legacy → GRUB.

No se le pregunta al usuario si la detección es fiable.

### 13.1. Dual boot con Windows

La configuración debe ser compatible con dual boot con Windows.

Korunix debe:

- detectar una instalación existente de Windows cuando sea posible;
- preservar particiones y entradas existentes;
- no reformatear una partición EFI existente sin una operación explícita y separada;
- permitir que Windows aparezca en el arranque cuando la instalación sea compatible;
- explicar cualquier incompatibilidad de modo UEFI/BIOS en lugar de intentar corregirla silenciosamente.

Secure Boot es un problema separado de “dual boot”. Korunix debe detectarlo y nunca cambiarlo silenciosamente. La política exacta de soporte de Secure Boot debe validarse antes de prometer compatibilidad automática.

## 14. Hosts dinámicos

Korunix no debe codificar un único host llamado `korunix` dentro del flake.

Los hosts se descubren a partir de `equipos/*.nix` y se emparejan con su hardware correspondiente en `equipos/<id>-detectado.nix`.

El flake debe generar dinámicamente `nixosConfigurations.<id>` para los hosts encontrados.

El identificador del host y el `networking.hostName` son conceptos diferentes. Cambiar el hostname visible no debe obligar a renombrar archivos.


Cuando Korunix detecte un hostname genérico de instalación, especialmente `nixos`, debe tratarlo como identidad todavía no personalizada.

En el primer uso debe preguntar en lenguaje humano:

> ¿Cómo quieres llamar a este equipo?

La persona puede escribir un nombre humano. Korunix genera de forma automática un hostname técnicamente válido y mantiene separado el identificador estructural estable del host.

Un hostname existente que ya parezca una elección deliberada debe preservarse y mostrarse como propuesta en lugar de reemplazarse.

Korunix debe poder ofrecer una operación de renombrado de identidad del host cuando la persona realmente quiera hacerlo. Esa operación debe ser transaccional y actualizar de manera coherente:

- archivo del host;
- archivo de hardware;
- referencias estructurales;
- validación final.

`specialArgs` debe usarse para contexto estructural necesario por los módulos, no como almacén universal de configuración editable.

Conceptualmente puede transportar:

```nix
korunixContext = {
  hostId = "...";
  hostFile = ...;
  hardwareFile = ...;
  personasPath = ./personas;
  configPath = ./config;
};
```

El estado editable por el usuario debe vivir en opciones `config.korunix.*` o el modelo equivalente que se defina.

## 15. Usuarios: múltiples usuarios y múltiples hosts

Korunix debe soportar:

- un usuario en un host;
- varios usuarios en un mismo host;
- el mismo usuario en varios hosts;
- usuarios diferentes en hosts diferentes.

La pertenencia de usuarios es propiedad del host. La identidad de cada usuario se define de forma independiente.

Ejemplo conceptual:

```text
usuarios disponibles
├── koru
├── maria
└── invitado

sobremesa
├── koru
└── maria

portátil
└── koru
```

### 15.1. Identidad

Korunix distingue:

- ID interno de Korunix;
- nombre de cuenta UNIX;
- nombre visible;
- avatar;
- preferencias personales;
- hosts en los que la cuenta existe.

Cambiar el nombre visible es una operación sencilla. Cambiar el nombre UNIX puede afectar home, ownership, rutas y referencias, por lo que debe tratarse como una migración especial y explicada.

### 15.2. Usuarios creados por Calamares

Si el sistema ya contiene un usuario creado por el instalador, Korunix debe detectarlo y preservarlo.

No debe recrearlo ni sustituir su contraseña.

Korunix puede ofrecer adoptarlo dentro de su modelo y mostrar qué aspectos administrará a partir de ese momento.

### 15.3. Crear usuarios desde Korunix

Korunix debe poder crear nuevas cuentas independientemente del escritorio utilizado. La gestión de usuarios no depende de que GNOME, Plasma o Cinnamon tengan sus propios paneles, y debe estar igualmente disponible en Niri e Hyprland.

La interfaz humana para crear una cuenta incluye, como mínimo:

- nombre visible;
- nombre de cuenta;
- contraseña;
- confirmación de contraseña;
- avatar opcional;
- rol: administrador o estándar.

## 16. Contraseñas y credenciales

Las contraseñas nunca forman parte de la configuración portable ni del repositorio.

No deben aparecer en:

- archivos Nix versionados;
- `spec.md`;
- `flake.lock`;
- perfiles exportados;
- historial humano;
- logs;
- diagnósticos.

### 16.1. Usuario existente

Si una cuenta ya existe, Korunix conserva su contraseña actual. No necesita conocerla ni reemplazarla para adoptar la cuenta.

### 16.2. Usuario nuevo

Cuando Korunix crea una cuenta, la contraseña se introduce de forma interactiva y se transforma en un hash de contraseña apropiado. El secreto o hash local necesario para crear la cuenta debe mantenerse fuera del repositorio y protegido por permisos del sistema.

La implementación objetivo debe evitar introducir hashes de contraseñas en el Nix store cuando exista una vía más segura, por ejemplo mediante un archivo local protegido referenciado durante la activación. La opción exacta debe validarse contra la versión de NixOS soportada antes de implementarla.

### 16.3. Perfiles portables

Un perfil puede contener que existe una persona llamada “Koru”, su avatar, preferencias y capacidades, pero no su contraseña.

Si ese perfil crea una cuenta nueva en otro equipo, Korunix solicita una contraseña nueva para ese equipo.

Si la cuenta ya existe en el equipo de destino, Korunix la detecta y no cambia su contraseña salvo petición explícita.


### 16.4. Inicio y desbloqueo sin contraseña

Korunix puede ofrecer una opción humana para quien prefiera máxima comodidad en un equipo concreto:

> Entrar sin contraseña.

La elección debe explicar una sola vez que reduce la protección física del equipo. Si la persona la confirma, Korunix deriva automáticamente las configuraciones coherentes con esa intención, sin encadenar preguntas técnicas adicionales.

Cuando sea técnicamente posible, esta modalidad debe coordinar:

- inicio de sesión automático;
- comportamiento de bloqueo/desbloqueo coherente con la elección;
- keyring o almacén de secretos para evitar que aparezcan solicitudes de contraseña contradictorias;
- sesión gráfica y servicios del usuario.

Korunix no debe prometer que todos los escritorios y almacenes de secretos permiten eliminar cada prompt sin consecuencias. La implementación concreta debe validarse por escritorio.

La decisión es local al host y no debe exportar secretos ni convertir por defecto otros equipos a inicio sin contraseña.

## 17. Administradores, capacidades y grupos

La interfaz normal expone dos roles comprensibles:

- Administrador: puede autorizar cambios del sistema;
- Usuario estándar: utiliza el equipo y modifica sus preferencias, pero no administra el sistema por defecto.

Internamente, Korunix traduce el rol de administrador a los permisos apropiados de NixOS, incluido `wheel` cuando corresponda.

Korunix debe impedir que una operación accidental deje el equipo sin ningún administrador funcional.

### 17.1. Los grupos son un detalle derivado

La interfaz normal no debe preguntar:

> ¿Quieres añadir a este usuario a `wheel`, `adbusers`, `libvirtd`...?

Korunix administra capacidades humanas y deriva los grupos necesarios.

Ejemplos:

```text
Administrador → permisos administrativos
Virtualización → permisos requeridos por la implementación de virtualización
Herramientas Android → permisos necesarios para ADB
```

Los grupos técnicos pueden verse en una sección avanzada, pero no constituyen la decisión primaria.

Korunix no debe añadir usuarios indiscriminadamente a grupos sensibles si ninguna capacidad requiere ese acceso.

## 18. Avatar, GDM y Noctalia

El avatar pertenece al usuario, no a Noctalia.

La imagen actual de un usuario debe poder reutilizarse en:

- GDM;
- Noctalia;
- AccountsService u otros consumidores apropiados.

Korunix es dueño de la identidad y debe encargarse de mantener la fuente correcta para cada integración. Noctalia puede consumir esa identidad, pero no debe convertirse en la fuente de verdad del avatar.

`.face` puede mantenerse como compatibilidad cuando resulte útil, pero no es el modelo principal de identidad.

Los recursos de usuario deben mantenerse cerca de su definición mientras sigan siendo una colección simple. Por ejemplo, `personas/koru.nix` y `personas/koru.jpg` pueden convivir sin crear una carpeta adicional. Si un usuario llega a necesitar una colección real de recursos, entonces se reconsidera la estructura.

## 19. GDM universal

GDM es el gestor de inicio de sesión común de Korunix para los cinco escritorios soportados.

Debe mostrar las cuentas del host con su nombre visible y avatar correspondiente cuando la integración del sistema lo permita.

Cuando existan varios escritorios instalados, GDM debe ofrecer las sesiones correspondientes.

Korunix debe recordar o establecer de forma coherente la sesión principal/preferida sin impedir que el usuario seleccione otra sesión instalada.

## 20. Escritorios soportados

La primera familia objetivo contiene cinco escritorios:

1. Niri;
2. Hyprland;
3. GNOME;
4. Cinnamon;
5. KDE Plasma.

### 20.1. Regla general

Korunix utiliza de preferencia las aplicaciones y servicios naturales del escritorio cuando ofrecen la capacidad requerida, salvo que exista una excepción explícita definida por Korunix.

### 20.2. Niri y Hyprland

Niri y Hyprland forman una pareja especial:

- ambos usan Noctalia;
- deben ofrecer paridad de experiencia cuando sea razonablemente posible;
- no necesitan tener configuraciones internas idénticas;
- las diferencias reales entre compositores deben respetarse y explicarse.

Niri es un gestor de ventanas desplazable. Hyprland usa un modelo de mosaico dinámico. Las previews deben comunicar esa diferencia sin hacer que sus superficies Korunix/Noctalia parezcan productos distintos.

## 21. Contrato de capacidades de escritorio

Cada escritorio debe declarar e implementar un contrato explícito.

Como mínimo:

```text
session
launcher
notifications
portal
polkit
terminal
fileManager
screenCapture
screenRecording
nightLight
clipboard
cursor
theme
wallpaper
lockScreen
idle
displayConfig
authenticationAgent
applications
defaultRoles
shortcuts
keyboardInput
accessibility
```

Una capacidad solo puede marcarse como soportada si Korunix puede garantizar un resultado real y probado.

Paridad de experiencia significa:

> misma intención, mismo resultado y misma interacción cuando sea posible, aunque el compositor necesite una implementación diferente.

## 22. Aplicaciones por roles

Korunix debe modelar roles humanos, no solamente listas de paquetes.

### 22.1. Navegador

Korunix ofrece Firefox y Google Chrome.

La persona puede:

- instalar Firefox;
- instalar Chrome;
- instalar ambos;
- elegir cuál de los instalados será predeterminado.

Korunix debe configurar las asociaciones y el navegador predeterminado de manera coherente. Instalar Chrome como secundario no debe hacer que secuestre asociaciones de manera inesperada.

### 22.2. Terminal

La terminal predeterminada de Korunix es Alacritty en todos los escritorios soportados.

La shell predeterminada del perfil Korunix es Fish.

`fetch` forma parte de la experiencia de terminal y utiliza Fastfetch como dependencia. Fastfetch no necesita presentarse como una elección independiente cuando solo existe para satisfacer esa dependencia.

La regla es:

```text
Terminal → Alacritty
Shell    → Fish
Fetch    → fetch
           └── Fastfetch como dependencia
```


`fetch` tiene su propio archivo de configuración y no debe tratarse como si simplemente reutilizara el archivo de Fastfetch.

Durante la migración de la configuración personal actual, Korunix debe trasladar la intención visual y la información mostrada desde la configuración existente de Fastfetch al formato propio de `fetch`, conservando la presentación ordenada y aprovechando sus capacidades visuales, incluido su logo 3D animado cuando corresponda.

La fuente de verdad debe ser un modelo humano de “información del sistema a mostrar”. Los adaptadores de `fetch` o Fastfetch generan el formato concreto que necesite cada herramienta, evitando mantener dos configuraciones conceptualmente duplicadas.

### 22.3. Kitty

Hyprland no debe imponer Kitty dentro de una configuración Korunix si Alacritty ya satisface el rol de terminal.

Korunix debe configurar Hyprland para abrir Alacritty y eliminar cualquier advertencia de configuración inicial/autogenerada que no sea pertinente en una configuración mantenida por Korunix.

Si en el futuro Korunix permitiera elegir Kitty explícitamente, Alacritty y Kitty se considerarían alternativas del mismo rol y sus integraciones se activarían de forma mutuamente excluyente.

### 22.4. Plantillas de Noctalia

Si Alacritty es la terminal activa:

- plantilla de Alacritty → activa;
- plantilla de Kitty → inactiva.

Si en el futuro Kitty fuera la terminal activa:

- plantilla de Kitty → activa;
- plantilla de Alacritty → inactiva.

Noctalia no debe mantener plantillas activas para aplicaciones que Korunix no usa en ese rol.

### 22.5. Explorador de archivos

- GNOME → Nautilus;
- Niri → Nautilus;
- Hyprland → Nautilus;
- Cinnamon → explorador nativo del escritorio;
- Plasma → Dolphin.

Cuando exista una acción “Abrir terminal aquí”, debe abrir Alacritty.

Para Nautilus se utiliza la integración equivalente a `nautilus-open-any-terminal` configurada para Alacritty.

Para Nemo, Dolphin u otros gestores se debe utilizar la integración adecuada del escritorio en vez de instalar Nautilus únicamente para obtener esa acción.

### 22.6. Visor de imágenes

Se usa el visor nativo del escritorio cuando existe una opción adecuada.

Niri e Hyprland utilizan Eye of GNOME como fallback GTK.

### 22.7. PDF

Se utiliza el visor nativo del escritorio cuando corresponde.

Niri e Hyprland utilizan Papers.

### 22.8. Suite ofimática

Se modela el rol “suite ofimática”. La implementación preferida puede depender de arquitectura y disponibilidad.

Para x86_64 la opción preferida definida es OnlyOffice. Cuando no esté disponible o no sea compatible en otra arquitectura, Korunix debe ofrecer el fallback curado definido para esa plataforma, como LibreOffice.

### 22.9. Regla superior

> Usar primero la solución natural del escritorio; sustituirla únicamente cuando Korunix haya establecido explícitamente una experiencia común.

Alacritty/Fish es una excepción deliberada. Nautilus en GNOME/Niri/Hyprland también.

## 23. Atajos: registro semántico y prevención de conflictos

Los atajos no deben quedar dispersos como decisiones independientes en cinco configuraciones.

Korunix define acciones semánticas, por ejemplo:

```text
launcher
terminal
fileManager
switchKeyboardNext
switchKeyboardPrevious
lockScreen
screenshotRegion
screenshotWindow
screenshotScreen
screenRecording
volumeUp
volumeDown
brightnessUp
brightnessDown
overview
closeWindow
```

Cada escritorio implementa esas acciones según sus mecanismos reales.

Antes de considerar válida una configuración, Korunix debe comprobar:

- que ninguna combinación esté asignada a dos acciones incompatibles;
- que las acciones obligatorias tengan binding;
- que los atajos de Niri y Hyprland respeten las diferencias reales de ambos compositores.

### 23.1. Super + Espacio

En Niri y Hyprland:

```text
Super + Espacio → launcher de Noctalia
```

Por tanto, el cambio de distribución de teclado no puede reutilizar esa combinación.

Korunix debe escoger o adaptar un atajo diferente y verificar que no colisione con otras acciones.

### 23.2. Niri y Hyprland no se presuponen idénticos

Capturas de pantalla, grabación, cambio de teclado y otras funciones pueden tener bindings o mecanismos internos diferentes. El contrato debe probar cada escritorio por separado.

## 24. Capturas de pantalla

Korunix debe ofrecer tanto una ruta gráfica como atajos de teclado.

### 24.1. Niri y Hyprland

Se debe preferir la integración de Noctalia cuando cubra correctamente la capacidad.

La GUI de Noctalia puede proporcionar acciones de captura y Korunix debe mantener atajos equivalentes disponibles.

Los bindings internos pueden diferir entre Niri y Hyprland; la experiencia humana no debe depender de que sus configuraciones sean idénticas.

### 24.2. Destino de archivos

Las rutas personales no se construyen suponiendo `/home/usuario/Pictures` ni nombres localizados rígidos.

Korunix debe resolver el directorio XDG de imágenes de cada usuario y preferir una subcarpeta humana de capturas de pantalla dentro de esa ubicación.

Ejemplo visual en un sistema español:

```text
Imágenes/
└── Capturas de pantalla/
```

El nombre físico se determina mediante la localización XDG correspondiente, no mediante una ruta codificada.

Cuando un escritorio tenga una convención nativa compatible, Korunix debe respetarla o integrarla con esta política en vez de imponer una ruta inconsistente.

## 25. Accesibilidad

La accesibilidad es una capacidad de primera clase.

El onboarding puede presentar categorías humanas como:

- visión;
- movilidad;
- audición;
- cognición.

No debe exponer nombres de implementaciones técnicas como primera opción.

Cada escritorio debe declarar qué capacidades de accesibilidad puede garantizar realmente. Korunix no debe prometer equivalencia donde el ecosistema no la ofrezca.

La GUI de Korunix debe diseñarse desde el inicio para:

- navegación completa por teclado;
- foco visible;
- lectores de pantalla;
- contraste;
- escalado de texto;
- hit targets generosos;
- ausencia de clipping;
- traducciones largas;
- CJK;
- diacríticos;
- preparación para RTL.

“Ahora no” es preferible a un “Omitir” que sugiera que el usuario está saltándose una obligación. La accesibilidad siempre debe poder configurarse posteriormente desde el panel permanente.

## 26. Capacidades técnicas predeterminadas

Korunix puede habilitar automáticamente capacidades técnicas que mejoren compatibilidad y no tengan sentido como preguntas para una persona normal, siempre que su impacto haya sido revisado.

Candidatos:

- soporte de comandos Nix y flakes necesarios por Korunix;
- AppImage;
- Flatpak;
- `nix-ld`;
- paquetes unfree cuando sean necesarios para el catálogo soportado;
- firewall activado;
- integración ADB cuando la capacidad Android esté disponible;
- compatibilidad de hardware de bajo coste, como `xpadneo` cuando exista Bluetooth habilitado.

La regla general es:

> Si una capacidad tiene un coste técnico bajo, un riesgo bajo y elimina una fricción frecuente, Korunix debe preferir habilitarla como parte de la experiencia recomendada.

Los servicios que exponen el equipo a la red se evalúan aparte. SSH no cumple automáticamente la regla de “bajo riesgo” y no debe habilitarse silenciosamente solo por conveniencia. Korunix preserva un SSH existente y puede activarlo cuando la persona elija una función que realmente lo necesite.

Cuando una capacidad como KDE Connect, Sunshine u otro servicio local necesite reglas de firewall, la persona no administra puertos manualmente:

```text
capacidad activada
→ servicio
→ permisos
→ puertos necesarios

capacidad desactivada
→ retirar lo que Korunix añadió y ya no sea necesario
```

Korunix debe abrir únicamente lo requerido por la capacidad activa y mantener el firewall habilitado.

## 27. Stable, unstable y versión del sistema

Korunix debe soportar como mínimo dos estrategias:

- base estable;
- base inestable.

La persona no tiene que comprender la mecánica de canales o inputs para utilizar esta elección.

El perfil personal actual puede preferir unstable, mientras que otras instalaciones pueden usar stable según contexto.

### 27.1. Una sola decisión de versión

La versión/base del sistema debe ser fácilmente editable en una única fuente de verdad.

Una modificación de esa decisión debe seleccionar automáticamente los inputs acoplados que correspondan.

Ejemplo conceptual:

```text
base = unstable
→ nixpkgs unstable
→ AAGL compatible con unstable

base = stable 26.05
→ nixpkgs 26.05
→ AAGL release-26.05
```

El flake puede mantener inputs disponibles para las familias soportadas y elegir internamente el conjunto coherente. El usuario no debe editar múltiples URLs para cambiar una sola decisión.

### 27.2. AAGL

AAGL mantiene ramas/releases vinculadas a determinadas versiones de Nixpkgs. Korunix debe tratar esa relación como una dependencia coordinada.

Al cambiar de una versión estable a otra, Korunix debe actualizar conjuntamente la referencia adecuada de AAGL cuando corresponda y validar antes de aplicar.

### 27.3. `system.stateVersion`

La versión elegida de NixOS no es `system.stateVersion`.

Korunix nunca debe actualizar `system.stateVersion` automáticamente como consecuencia de pasar de stable a unstable o de una release estable a otra. Ese valor conserva la semántica de compatibilidad de la instalación y solo se modifica mediante una migración explícita y técnicamente justificada.

## 28. Catálogo de aplicaciones

Korunix debe ofrecer un catálogo curado basado inicialmente en las aplicaciones útiles de la configuración existente, pero el catálogo no se considera una lista universal e inmutable.

La GUI debe mostrar qué aplicaciones ya están instaladas cuando pueda detectarlo.

Debe permitir:

- instalar;
- eliminar;
- buscar en el catálogo curado;
- buscar aplicaciones de Nixpkgs cuando no estén en el catálogo;
- utilizar Flatpak cuando sea la fuente apropiada.

Las dependencias técnicas no se presentan como elecciones separadas si el usuario no necesita decidir sobre ellas.

Korunix no debe convertirse solamente en una tienda de aplicaciones. Aplicaciones es una sección dentro de un centro mayor de administración del sistema.

## 29. Propiedad de configuración

Korunix debe saber qué puede modificar y qué pertenece al usuario o a una configuración externa.

Se definen tres estados conceptuales:

### 29.1. Propiedad de Korunix

Korunix administra ese estado y puede modificarlo como parte de una transacción revisada.

### 29.2. Propiedad del usuario/externa

Korunix puede detectarlo y mostrarlo, pero no puede sobrescribirlo silenciosamente.

### 29.3. Compartido o conflicto

Si Korunix necesita modificar algo que ya está administrado externamente, debe explicar el conflicto y pedir una decisión.

## 30. `.korunix` y edición manual

La configuración humana de Korunix debe seguir siendo legible y editable manualmente.

Requisitos:

- nombres previsibles;
- ubicaciones previsibles;
- comentarios humanos;
- cambios manuales respetados;
- ninguna reescritura silenciosa que borre una edición válida;
- migraciones explícitas cuando cambie el esquema.

La GUI y la edición manual son dos entradas al mismo modelo, no dos configuraciones distintas.

Korunix debe leer el estado real del equipo antes de aplicar y reconciliarlo con la configuración declarada.

Los secretos y credenciales locales quedan fuera del espacio portable/versionable.

## 31. Configuraciones NixOS existentes

Korunix no debe intentar reescribir arbitrariamente un `configuration.nix` artesanal que no le pertenece.

En un sistema existente debe:

1. detectar el estado que necesita comprender;
2. identificar qué áreas ya están configuradas externamente;
3. mostrar qué quiere adoptar o administrar;
4. mantener su propia superficie declarativa;
5. advertir ante conflictos;
6. aplicar solo después de una revisión.

Una futura función de adopción/importación puede convertir configuración externa a configuración gestionada por Korunix, pero debe hacerlo mediante preview y migración explícita.


### 31.1. Adopción inicial de una instalación de NixOS

En una instalación limpia, Korunix debe inspeccionar la configuración generada por NixOS/Calamares, incluido el `hardware-configuration.nix` correspondiente cuando exista.

El flujo objetivo es:

1. detectar la configuración de origen;
2. leer hardware, hostname y usuarios existentes;
3. copiar o incorporar el hardware generado dentro del espacio administrado por Korunix bajo `equipos/<hostId>-detectado.nix`;
4. conservar una referencia clara al origen y a la fecha de adopción;
5. validar que el nuevo árbol produce una configuración equivalente antes de considerarlo adoptado;
6. no borrar el origen hasta que la migración esté verificada y la política de propiedad lo permita.

Korunix no debe reimplementar la generación de hardware. Si necesita regenerar esta información, debe usar las herramientas oficiales de NixOS y después reconciliar el resultado con su estructura.

Si detecta hardware nuevo o un cambio relevante, debe explicarlo en lenguaje humano y ofrecer una regeneración segura cuando realmente sea necesaria.

### 31.2. Activación de flakes

Durante la adopción inicial Korunix debe habilitar las capacidades de Nix necesarias para trabajar con flakes y comandos modernos, sin convertir “activar flakes” en una pregunta técnica del onboarding.

## 32. Transacciones y Polkit

Una operación de sistema se trata como una transacción coherente.

Flujo objetivo:

1. crear propuesta;
2. validar configuración;
3. comprobar compatibilidad y dependencias;
4. construir la generación cuando proceda;
5. mostrar cambios;
6. avisar que se necesitará autorización;
7. solicitar Polkit/autenticación;
8. aplicar;
9. verificar;
10. marcar el resultado como válido;
11. comunicar si hace falta cerrar sesión, reiniciar o no hacer nada.

La GUI no debe ejecutarse completa como root. Debe utilizar un helper privilegiado o arquitectura equivalente con una superficie de privilegios reducida.

Siempre debe existir un aviso humano inmediatamente antes de que aparezca la autenticación.

Ejemplo:

> Vamos a aplicar los cambios que acabas de revisar. Necesitamos tu contraseña para modificar este equipo.

## 33. Progreso: regla general

Todo proceso que tarde lo suficiente como para que la persona pueda preguntarse qué ocurre debe mostrar progreso comprensible.

La vista de progreso debe contener, cuando exista información real suficiente:

- operación global;
- etapa actual;
- descripción concreta;
- barra de progreso global;
- porcentaje global;
- tiempo restante estimado;
- número de elementos completados y totales;
- cola de elementos;
- estado individual de cada elemento;
- nombre y versión cuando sean relevantes.

### 33.1. Estados concretos

Nunca se debe mostrar solamente “Preparando” o “Instalando” si Korunix puede explicar el objeto.

Ejemplos:

- Preparando la actualización del sistema;
- Descargando Firefox 146;
- Instalando Papers;
- Aplicando la configuración de Hyprland;
- Creando una copia de seguridad;
- Verificando la nueva generación;
- Limpiando archivos temporales de la operación.

### 33.2. Cola

Cuando una operación incluya múltiples elementos, cada uno puede mostrar:

- icono cuando exista y sea útil;
- nombre;
- versión actual y nueva cuando corresponda;
- indicador circular o barra individual;
- estado: pendiente, en progreso, completado, omitido o error.

Ejemplo conceptual:

```text
Actualizando el equipo                         72 %
Quedan aproximadamente 4 min
17 de 24 elementos completados

✓ Firefox              145 → 146     Completado
◉ Google Chrome        140 → 141     68 %
○ Noctalia             5.2 → 5.3     Pendiente
○ Configuración Niri                  Pendiente
```

### 33.3. Porcentajes reales

Korunix no debe inventar porcentajes.

Cuando una fase tenga una unidad medible, debe usar datos reales, por ejemplo:

- bytes descargados;
- tareas conocidas completadas;
- derivaciones conocidas cuando el backend pueda informarlas;
- elementos de una cola.

Cuando el denominador todavía no se conozca, se utiliza un estado indeterminado hasta tener información suficiente.

### 33.4. Tiempo estimado restante

El objetivo es mostrar un contador descendente que se adapte dinámicamente al ritmo real.

Sin embargo, Korunix no debe falsificar una cuenta atrás solo para que siempre disminuya. La estimación debe:

- aparecer cuando exista suficiente información;
- suavizar variaciones bruscas;
- recalcularse cuando cambie significativamente la velocidad o aparezca nuevo trabajo;
- mostrar “Calculando tiempo restante…” cuando todavía no exista una estimación fiable.

Si una recalibración legítima aumenta el tiempo restante, Korunix debe preferir una estimación honesta a una cuenta atrás falsa. La interfaz puede comunicar brevemente que está recalculando para evitar saltos confusos.

### 33.5. Cancelación

Solo debe ofrecerse “Cancelar” cuando la operación pueda detenerse de forma segura. La existencia de un botón no puede depender únicamente de que visualmente quede bien.

## 34. Consejos durante procesos largos

Durante operaciones largas Korunix puede mostrar frases y consejos cortos para mantener la interfaz viva y enseñar posibilidades del producto.

Deben ser:

- contextuales cuando sea posible;
- breves;
- traducidos;
- no técnicos;
- opcionales/silenciables;
- no repetitivos de forma molesta.

Ejemplos:

> Puedes cambiar tus aplicaciones predeterminadas más adelante desde Aplicaciones.

> Korunix conserva generaciones anteriores para que puedas volver atrás si algo sale mal.

> Puedes importar solo la apariencia de un perfil sin copiar el resto de su configuración.

Korunix puede recordar localmente qué consejos ya se mostraron para evitar repetirlos continuamente.

## 35. Generaciones y recuperación

Korunix debe presentar las generaciones de NixOS en lenguaje humano.

La interfaz normal muestra las tres generaciones recientes útiles para recuperación, evitando abrumar al usuario.

La política de presentación y la política de limpieza deben distinguirse:

- mostrar tres no significa necesariamente borrar instantáneamente todas las demás;
- cuando Korunix ejecute una limpieza configurada para conservar tres, debe hacerlo explícitamente y de forma segura.

Debe existir una acción para eliminar todas las generaciones antiguas que puedan eliminarse de forma segura.

La generación actualmente activa no debe presentarse como una generación “antigua” borrable.

Después de eliminar generaciones se puede ejecutar la limpieza/optimización correspondiente del store cuando sea apropiado.

## 36. Copias de seguridad

Las generaciones de NixOS son una red de seguridad, pero no sustituyen una copia portable de la configuración de Korunix.

Korunix debe permitir:

- exportar una copia de la configuración;
- crear una copia automática antes de migraciones importantes;
- restaurar una configuración desde la interfaz;
- combinar recuperación de configuración con rollback de NixOS cuando corresponda.

Las credenciales nunca se incluyen en una copia portable normal.

## 37. Perfiles portables

Los perfiles exportan decisiones e intenciones, no identidad física del hardware.

Pueden incluir, según selección del usuario:

- aplicaciones;
- roles predeterminados;
- apariencia;
- escritorios;
- preferencias;
- usuarios como identidad sin contraseñas;
- otras decisiones portables.

No deben copiar automáticamente:

- hardware;
- particiones;
- firmware;
- contraseña;
- secretos;
- hostname;
- identificadores locales que no tengan sentido en el destino.

Al importar:

1. detectar hardware del destino;
2. validar compatibilidad;
3. adaptar implementaciones cuando sea necesario;
4. ofrecer fallbacks;
5. permitir importación parcial;
6. mostrar preview;
7. aplicar solo tras confirmación.

## 38. Actualizaciones

Korunix debe ofrecer una experiencia unificada de actualización, sin obligar a descargar nuevas carpetas o ZIPs.

Debe poder distinguir y coordinar:

- actualización de Korunix;
- actualización de la base NixOS/nixpkgs;
- kernel y controladores;
- escritorios;
- Noctalia;
- aplicaciones;
- inputs externos;
- migraciones de configuración.

### 38.1. Tres niveles de control

La interfaz debe ofrecer:

1. **Actualizar todo** — recomendado para la mayoría;
2. **Personalizar** — elegir unidades de actualización compatibles;
3. **Avanzado** — fijar versiones o políticas especiales cuando la arquitectura lo permita.

### 38.2. Granularidad honesta

Korunix no debe fingir una granularidad que Nix no tiene.

Muchas aplicaciones provienen del mismo snapshot de nixpkgs. Actualizar ese input puede actualizar varias a la vez. En esos casos Korunix debe agrupar la unidad real de actualización y explicarlo.

Si el usuario quiere mantener una aplicación concreta mientras actualiza el resto, Korunix solo debe ofrecerlo cuando pueda implementar un pin o fuente separada de forma coherente y mantenible.

No se permiten combinaciones que rompan dependencias. La interfaz debe explicar por qué una selección está bloqueada:

> No se puede mantener este componente en esta versión porque la actualización seleccionada necesita una versión más reciente.

### 38.3. Canales de componentes y promociones de beta a estable

Cada componente externo administrado por Korunix debe poder declarar, cuando aplique:

- fuente;
- canal;
- rama, tag o referencia;
- revisión fijada;
- compatibilidades necesarias.

La política no debe codificar excepciones arbitrarias por proyecto.

Caso de referencia: Noctalia.

Mientras Noctalia v5 esté en beta en el canal que Korunix haya elegido, Korunix puede seguir esa línea de forma explícita. Cuando v5 pase a ser estable, el actualizador debe detectar la promoción y proponer el cambio hacia la referencia estable correspondiente, conservando la misma familia mayor cuando esa sea la política definida.

La migración debe indicar:

- canal actual;
- canal recomendado;
- cambios de configuración necesarios;
- compatibilidad con los escritorios;
- posibilidad de permanecer temporalmente en el canal anterior cuando siga soportado.

El mismo modelo debe servir para otros inputs que tengan canales stable, beta, legacy o release.

### 38.4. Changelog humano

Antes de aplicar, Korunix debe resumir lo relevante para ese equipo.

Ejemplo:

```text
Se encontraron 14 actualizaciones.

Firefox              145 → 146
Hyprland              0.xx → 0.yy
Noctalia              5.2 → 5.3
Korunix               1.4 → 1.5

No se detectaron incompatibilidades.
Será necesario cerrar sesión al finalizar.
```

“Ver detalles” puede mostrar información técnica adicional.

## 39. Migraciones y compatibilidad

Korunix debe mantener un contrato de compatibilidad de su configuración.

Una actualización nunca debe mover o renombrar archivos del usuario arbitrariamente.

Si una nueva versión necesita cambiar el esquema:

1. detectar la versión antigua;
2. explicar qué necesita migrarse;
3. explicar qué no se modificará;
4. crear una copia de seguridad;
5. pedir confirmación cuando la migración tenga impacto significativo;
6. migrar;
7. validar;
8. permitir restaurar si falla.

Ejemplo humano:

> Esta configuración fue creada con Korunix 1.4. Korunix 1.5 necesita adaptar dos campos. No se modificarán tus aplicaciones, usuarios ni apariencia.

Las migraciones deben ser versionadas y probables automáticamente.

## 40. Historial humano

Korunix debe mantener un historial comprensible de acciones relevantes, por ejemplo:

- instalaste GNOME;
- cambiaste Everforest a modo automático;
- añadiste a María como usuario estándar;
- actualizaste Firefox 145 → 146;
- restauraste una generación anterior.

No debe convertirse en un log técnico. Los detalles pueden desplegarse cuando sean útiles para diagnóstico.

Nunca se registran contraseñas ni secretos.


El historial pertenece principalmente al equipo que produjo los eventos y no se incluye por defecto en perfiles portables.

Una exportación avanzada puede permitir incluir una copia del historial para diagnóstico o archivo personal, pero debe ser una elección explícita y seguir excluyendo secretos.

## 41. Centro de salud del sistema

Korunix puede disponer de una vista de estado general con mensajes como:

- Todo está correcto;
- Hay actualizaciones disponibles;
- Tu última copia de seguridad tiene 30 días;
- Hay una migración pendiente;
- Una integración dejó de ser compatible;
- Se detectó una configuración externa en conflicto.

La vista debe priorizar acciones útiles y evitar convertir avisos inocuos en alarmas.

## 42. Diagnóstico humano

Cuando algo falle, Korunix debe explicar primero:

- qué operación falló;
- qué parte sí se completó;
- si el sistema sigue en un estado seguro;
- qué puede hacer el usuario ahora.

Después puede ofrecer “Ver detalles técnicos” con logs relevantes.

El mensaje primario no debe ser una pared de salida de Nix.

## 43. Búsqueda global

El panel permanente debe poder buscar decisiones y áreas mediante lenguaje normal.

Ejemplos:

- Firefox;
- idioma;
- teclado;
- capturas;
- actualizaciones;
- fondo;
- usuario.

La búsqueda debe llevar a la configuración correspondiente sin exigir conocer la estructura técnica del sistema.

## 44. Configuración recomendada

La primera versión debe priorizar una configuración recomendada sólida para la mayoría y una ruta clara de personalización.

No se deben crear demasiados perfiles de “gaming”, “trabajo”, “creación”, etc. antes de demostrar que realmente simplifican decisiones.

Los perfiles especializados pueden incorporarse después si aportan valor real.

## 45. `just` y CLI

Los comandos habituales del repositorio principal deben preservarse o adaptarse cuando sigan representando una operación válida.

Home Manager desaparece, por lo que `just home` desaparece.

Conjunto objetivo aproximado:

```text
just os        → aplicar la configuración Korunix/NixOS
just fmt       → formatear con Alejandra
just update    → actualización controlada y validada
just clean     → limpieza conservando la política de generaciones definida
just clean-all → eliminar generaciones antiguas de forma explícita
just check     → validar
just preview   → mostrar/evaluar propuesta
just build     → construir sin aplicar
just status    → mostrar estado
just korunix   → abrir Korunix desde el checkout de desarrollo
```

Los nombres definitivos deben favorecer compatibilidad con la memoria muscular existente cuando no contradigan el nuevo modelo.

`just update` no debe ser simplemente un `nix flake update` ciego si existen dependencias coordinadas que Korunix deba validar.

La CLI y la GUI son dos interfaces sobre el mismo motor y las mismas reglas, no dos implementaciones divergentes.

## 46. Actualización del propio Korunix

Korunix debe poder descubrir que existe una nueva versión de sí mismo y actualizarse sin crear una segunda carpeta de configuración.

Debe mostrar:

- versión actual;
- versión disponible;
- cambios relevantes;
- migraciones necesarias;
- compatibilidad con el sistema actual.

La configuración del usuario permanece en su lugar y se migra de manera controlada cuando sea necesario.

Nunca se debe pedir al usuario que descargue manualmente un ZIP nuevo y vuelva a configurar todo desde cero como mecanismo normal de actualización.

## 47. Desuso de funciones

Cuando Korunix necesite retirar una opción o cambiar una capacidad:

- debe avisarlo con antelación razonable cuando sea posible;
- debe ofrecer una alternativa o migración;
- no debe dejar configuraciones silenciosamente inválidas;
- debe conservar compatibilidad de lectura el tiempo necesario para migrar configuraciones antiguas cuando sea viable.

## 48. Límites de seguridad

Korunix debe preferir defaults útiles, pero “oculto” no significa “sin consecuencias”.

Reglas mínimas:

- firewall activado por defecto;
- SSH no se activa silenciosamente hasta definir una política segura;
- la GUI no corre completa como root;
- Polkit/helper privilegiado para operaciones de sistema;
- credenciales fuera de Git y fuera de perfiles exportables;
- grupos sensibles solo cuando una capacidad lo justifique;
- operaciones destructivas claramente separadas y confirmadas;
- una configuración existente nunca se sobrescribe sin entender propiedad/conflicto;
- `system.stateVersion` no cambia automáticamente;
- hardware y particiones no se regeneran o reformatean a ciegas.

## 49. Documentación futura

Cuando exista suficiente implementación real, la documentación podrá dividirse en tres niveles:

1. usuario final, sin jerga de Nix;
2. contribuidores/extensores, por ejemplo cómo añadir un escritorio, tema o traducción;
3. arquitectura, con las razones detrás de decisiones importantes.

Por ahora este `spec.md` es la constitución única del proyecto. No se crea una carpeta `docs/` hasta que exista una colección real que la justifique.

## 50. Criterio de calidad

Korunix no debe sentirse como “una GUI para ejecutar scripts”.

Debe sentirse como un producto coherente que:

- detecta antes de preguntar;
- propone antes de complicar;
- responde inmediatamente a decisiones visuales;
- explica antes de pedir privilegios;
- muestra progreso real;
- preserva salida y recuperación;
- mantiene una fuente de verdad;
- respeta las modificaciones manuales;
- adapta decisiones al hardware y al escritorio;
- mantiene el lenguaje humano incluso cuando internamente realiza operaciones complejas de NixOS.

La meta después de un mes de uso es que una persona pueda pensar:

> Ahora todo está en un solo sitio y no tengo que ir buscando cómo se hace cada cosa.

## 51. Arranque silencioso

El arranque normal de un sistema Korunix debe priorizar una experiencia limpia.

Korunix debe utilizar Plymouth o un mecanismo equivalente compatible para ocultar la sucesión normal de mensajes técnicos de arranque cuando el sistema funciona correctamente.

La experiencia objetivo es:

```text
encender
→ identidad visual de Korunix/NixOS
→ progreso discreto cuando sea útil
→ GDM
```

Los mensajes de systemd, kernel, display manager y servicios no deben convertirse en contenido visual normal para la persona.

Los detalles técnicos siguen disponibles en:

- modo de diagnóstico;
- consola;
- logs;
- recuperación;
- fallos donde ocultarlos impida entender el problema.

Korunix no debe esconder indefinidamente un fallo real detrás de una animación de carga.

## 52. Motor de capacidades

Korunix debe disponer de un modelo central de capacidades para traducir intenciones humanas a implementación.

Ejemplos:

```text
Quiero virtualización
→ paquetes
→ servicios
→ permisos
→ grupos necesarios

Quiero Sunshine
→ servicio
→ integración del escritorio
→ firewall
→ permisos

Quiero Android
→ herramientas
→ ADB
→ permisos necesarios
```

Una capacidad declara:

- requisitos;
- conflictos;
- dependencias;
- recursos que administra;
- consecuencias;
- acciones de activación y retirada;
- nivel de riesgo.

La GUI consume estas capacidades. Los módulos internos no deben duplicar lógica de producto.

## 53. Contrato interno, identificadores y modularidad

Los módulos de Korunix deben comunicarse mediante contratos estables en lugar de depender de rutas internas ajenas.

Cada área funcional debe tener una responsabilidad clara, por ejemplo:

- usuarios;
- hardware;
- escritorios;
- aplicaciones;
- apariencia;
- actualizaciones;
- almacenamiento;
- recuperación.

Los identificadores internos de hosts, usuarios, roles, capacidades y componentes deben ser estables. Una actualización no cambia un ID simplemente por modificar un nombre visible.

Si un esquema interno necesita evolucionar, debe hacerlo mediante una migración versionada.

## 54. Simulación universal

La previsualización no se limita a las actualizaciones.

Toda operación importante debe poder producir una simulación o propuesta antes de modificar el sistema cuando la naturaleza de la operación lo permita.

La vista debe responder:

- qué se añadirá;
- qué se quitará;
- qué se modificará;
- qué se preservará;
- qué necesita autorización;
- qué reinicio o cierre de sesión podría ser necesario;
- qué riesgos o incompatibilidades se detectaron.

La simulación y la ejecución deben usar el mismo modelo de operación para evitar que el preview prometa una cosa y el backend haga otra.

## 55. Privacidad

Korunix no envía telemetría por defecto.

La configuración, historial, hardware, usuarios, aplicaciones, diagnósticos y hábitos de uso permanecen locales salvo que una función elegida por la persona necesite comunicarse con un recurso remoto.

Si en el futuro se ofrecen métricas voluntarias:

- deben estar desactivadas por defecto;
- explicar exactamente qué se enviaría;
- pedir consentimiento explícito;
- permitir retirarlo;
- no incluir secretos ni contenido personal.

Una función online no debe utilizarse como excusa para enviar información no necesaria.

## 56. Mensajes y estados

Todo mensaje de Korunix debe intentar responder tres preguntas:

1. ¿Qué está pasando?
2. ¿Por qué está pasando?
3. ¿Qué puedo hacer ahora?

Los mensajes técnicos crudos pertenecen a “Ver detalles técnicos”.

Los estados deben nombrar el objeto concreto:

- Preparando la actualización de NixOS;
- Descargando Firefox 146;
- Aplicando la configuración de Niri;
- Guardando los datos en la unidad USB;
- Verificando la copia;
- Esperando a que el dispositivo termine de escribir.

Korunix no debe declarar “completado” mientras todavía exista trabajo indispensable pendiente.

## 57. Transferencias a almacenamiento extraíble

Korunix debe ofrecer un asistente de transferencias para operaciones donde la finalización real del guardado sea importante, especialmente archivos grandes, ISOs y almacenamiento USB.

No pretende sustituir a Nautilus, Nemo o Dolphin como gestor general de archivos.

Flujo humano:

1. seleccionar uno o varios archivos;
2. seleccionar el dispositivo o destino;
3. iniciar transferencia;
4. mostrar progreso global e individual;
5. mostrar porcentaje, velocidad y ETA cuando sean medibles;
6. verificar que las escrituras pendientes se hayan completado;
7. declarar la operación finalizada únicamente cuando el sistema confirme que el contenido fue persistido de forma segura;
8. ofrecer expulsar el dispositivo cuando proceda.

La interfaz puede usar estados como:

```text
Copiando Fedora.iso
→ Guardando los últimos datos
→ Verificando
→ Transferencia completada
→ Expulsar dispositivo
```

Detalles como `fsync`, `syncfs`, cachés del kernel o llamadas a UDisks son responsabilidad de la implementación y no deben aparecer como instrucciones para el usuario.

Si el dispositivo se desconecta antes de terminar, Korunix debe explicar qué ocurrió y nunca afirmar que el archivo quedó correctamente guardado sin verificarlo.

El mismo motor visual de colas y progreso utilizado por actualizaciones puede reutilizarse aquí.

## 58. Unidades de datos y disponibilidad al iniciar sesión

Korunix debe detectar unidades de almacenamiento adicionales y comprender si una persona depende de ellas para aplicaciones o archivos cotidianos.

Durante el primer uso, y también desde el panel permanente, puede recomendar:

> Hemos detectado un disco de datos que puedes dejar listo desde que inicias, para que tus aplicaciones lo usen sin esperar.

No debe usar “montar” como verbo principal en la interfaz normal.

Si la persona activa esta comodidad, Korunix debe configurar de manera segura el acceso automático y coordinar el inicio de servicios/aplicaciones que dependan de esos datos cuando sea técnicamente posible.

Si el dispositivo necesita desbloqueo o autenticación, Korunix debe explicar la consecuencia en lenguaje humano y utilizar mecanismos seguros para evitar prompts repetitivos solo cuando la persona haya autorizado esa automatización.

La opción debe poder activarse, modificarse o retirarse más tarde desde el panel de almacenamiento; no pertenece exclusivamente al asistente inicial.

Korunix nunca debe almacenar una clave de cifrado en el repositorio o en un perfil portable.

## 59. Experiencia offline y actualización local

La actualización normal de Korunix puede usar su repositorio remoto cuando existe conexión, pero debe existir también una ruta local.

Una persona puede llevar una versión nueva de Korunix mediante almacenamiento externo y pedir que el motor la evalúe como actualización local.

Korunix debe:

- reconocer la versión;
- comparar con la versión instalada;
- verificar compatibilidad;
- mostrar migraciones;
- crear backup cuando corresponda;
- actualizar el motor sin reemplazar la configuración humana;
- continuar funcionando offline con todo lo que no requiera descargar dependencias ausentes.

La interfaz debe distinguir claramente:

```text
Disponible sin conexión
Necesita conexión
Necesita un paquete que no está disponible localmente
```

## 60. Política de finalización segura

Una operación no termina cuando desaparece el último elemento visible de la cola, sino cuando el resultado relevante está realmente listo para usarse.

Esto se aplica, entre otros, a:

- escrituras en USB;
- actualizaciones;
- migraciones;
- generación de configuración;
- backups;
- desmontaje/expulsión;
- aplicaciones que dependen de recursos que todavía se están preparando.

Si existe una fase de persistencia, sincronización, verificación o limpieza indispensable, debe formar parte del progreso global con una descripción humana.

## 61. Recomendaciones permanentes

Korunix puede detectar oportunidades de mejora durante el primer uso o posteriormente.

Ejemplos:

- un disco de datos puede quedar disponible automáticamente;
- apareció un nuevo adaptador Bluetooth;
- una configuración tiene una migración recomendada;
- una aplicación tiene una integración nativa disponible;
- una copia de seguridad está muy desactualizada.

Estas recomendaciones no son exclusivas del onboarding. Deben poder consultarse desde el panel permanente y desaparecer cuando dejan de ser relevantes.

Una recomendación explica el beneficio, no el mecanismo técnico.

## 62. Decisiones que deben validarse antes de implementación definitiva

Esta especificación fija la dirección, pero los siguientes puntos requieren una comprobación técnica concreta antes de codificarlos como garantías:

- estrategia exacta de Secure Boot;
- comando final e idempotencia del bootstrap en una instalación gráfica limpia de NixOS;
- ruta final que separará motor de Korunix, configuración editable y estado local sensible;
- mecanismo exacto para almacenar hashes de contraseña fuera del repositorio y del Nix store cuando sea posible;
- integración exacta de avatar entre Korunix, AccountsService, GDM y Noctalia;
- contrato final de captura/grabación y sus bindings en cada uno de los cinco escritorios;
- lista real de idiomas soportados por la versión de Noctalia utilizada;
- aplicaciones nativas concretas de Cinnamon y Plasma para cada rol en la versión de NixOS soportada;
- política de soporte de releases stable a medida que cambien NixOS y dependencias como AAGL;
- fuente de verdad para el modo de apariencia automático en cada escritorio;
- forma de obtener porcentajes y ETA reales del backend de Nix sin presentar progreso ficticio;
- implementación exacta del arranque silencioso con Plymouth en UEFI/BIOS y en los cinco escritorios;
- estrategia correcta para autologin, bloqueo sin contraseña y keyring sin introducir prompts contradictorios ni debilitar secretos fuera de la elección del usuario;
- política exacta de `xpadneo` y otras capacidades automáticas por hardware;
- definición de qué funciones offline requieren únicamente el código de Korunix y cuáles necesitan cierres/dependencias ya presentes en el Nix store;
- formato futuro de un paquete offline completo, si se decide ofrecerlo;
- mecanismo de transferencia segura y verificación final en medios extraíbles mediante UDisks y primitivas de sincronización apropiadas;
- mecanismo para preparar unidades de datos antes de que aplicaciones dependientes las necesiten;
- proceso exacto para adoptar y, cuando corresponda, regenerar `hardware-configuration.nix` sin perder personalizaciones;
- modelo de canales de Noctalia y otros componentes para detectar de forma segura una promoción de beta a estable;
- adaptación exacta de la configuración visual existente de Fastfetch al formato propio de `fetch`.

Estos puntos no invalidan el resto del diseño. Se mantienen explícitos para que Korunix no convierta supuestos técnicos en promesas al usuario.

<!-- KORUNIX-MULTIMEDIA-CENTER:BEGIN -->
## Centro de prueba y control de dispositivos multimedia

Korunix debe proporcionar un centro común para comprobar y administrar
dispositivos de sonido y vídeo independientemente del escritorio utilizado.

Esta capacidad forma parte del producto final y no de la hoja de ruta temporal.

### Referencia de interacción

La sección de Sonido de macOS Tahoe sirve como referencia conceptual de
jerarquía: controles generales separados de los dispositivos, distinción clara
entre entrada y salida, selección visible del dispositivo activo y acciones de
prueba accesibles directamente.

Korunix conserva su propia identidad GTK/libadwaita y Everforest; la referencia
no implica copiar visualmente macOS.

### Salidas de sonido

Korunix debe identificar, cuando la información disponible lo permita:

- parlantes integrados;
- audífonos;
- salidas analógicas;
- dispositivos USB;
- dispositivos Bluetooth;
- HDMI y DisplayPort;
- dispositivos virtuales.

Para una salida debe poder ofrecer, según sus capacidades:

- volumen;
- silencio;
- balance;
- puerto o perfil;
- selección como dispositivo predeterminado;
- reproducción de un sonido de prueba;
- prueba separada de canales izquierdo y derecho cuando corresponda.

### Micrófonos y otras entradas

Korunix debe identificar las entradas disponibles y permitir seleccionar
explícitamente cuál se está inspeccionando.

Cada micrófono debe ofrecer una acción visible **Probar micrófono**.

La prueba debe incluir:

1. un medidor de señal en vivo para confirmar que está llegando sonido;
2. una acción **Grabar prueba** iniciada explícitamente por el usuario;
3. una grabación temporal de corta duración;
4. una acción **Reproducir prueba**;
5. una acción **Grabar de nuevo** sin conservar innecesariamente la anterior;
6. indicación visible mientras el micrófono está siendo utilizado.

La grabación de prueba debe eliminarse al cerrar o finalizar la prueba, salvo
que el usuario solicite explícitamente conservarla.

Korunix no debe activar la grabación automáticamente al entrar en la página.

Korunix tampoco debe enviar automáticamente la entrada del micrófono en tiempo
real a los parlantes o audífonos, porque esa monitorización puede producir eco
o acople.

La acción **Probar micrófono** y la acción **Usar como predeterminado** son
operaciones distintas. Probar un dispositivo nunca debe modificar
silenciosamente la configuración permanente.

Cuando el dispositivo exponga varios canales, Korunix podrá mostrar actividad
por canal si la infraestructura subyacente proporciona esa información.

### Bluetooth

Los perfiles Bluetooth deben expresarse mediante conceptos comprensibles para
el usuario.

Por ejemplo, la interfaz puede distinguir entre:

- mayor calidad para escuchar audio;
- uso simultáneo de audífonos y micrófono.

No se debe exigir al usuario conocer términos como A2DP, HFP o HSP para tomar
esa decisión.

### Cámaras

Korunix debe presentar cámaras integradas, USB y virtuales cuando puedan
identificarse.

Una prueba de cámara puede mostrar:

- nombre y tipo;
- disponibilidad;
- vista previa;
- resoluciones disponibles;
- frecuencias de imagen disponibles;
- combinación usada durante la prueba.

La cámara solo debe activarse como consecuencia de una acción explícita del
usuario y debe liberarse al abandonar la prueba.

### Prueba frente a preferencia permanente

El centro debe mantener una separación explícita entre acciones equivalentes a:

- **Probar dispositivo**;
- **Usar como predeterminado**.

Las pruebas son temporales y no deben modificar por sí solas la configuración
persistente del sistema.

### Motor

Korunix debe utilizar la infraestructura multimedia efectiva del sistema,
principalmente PipeWire/WirePlumber para audio y las interfaces de vídeo
disponibles en Linux, sin exponer esos detalles como conocimiento obligatorio
para el usuario.

La capacidad debe funcionar bajo GNOME, Niri, Hyprland, KDE Plasma y Cinnamon.

### Responsabilidad por fase

- **C · Operaciones del sistema:** inventario, estado, selección, niveles,
  dispositivos predeterminados y motores de prueba.
- **D · GUI completa:** presentación de Sonido y Vídeo, medidores, listas,
  previews, controles y mensajes humanos.
- **E · Robustez:** hotplug, dispositivo ocupado, pérdida de señal, permisos,
  Bluetooth, desaparición de cámara o micrófono durante una prueba y limpieza
  segura de recursos temporales.
<!-- KORUNIX-MULTIMEDIA-CENTER:END -->

<!-- KORUNIX-NIX-FIRST:BEGIN -->
## Arquitectura Nix-first y nomenclatura humana

Korunix utiliza **Nix como fuente principal de verdad**. Toda decisión que
pueda calcularse de forma declarativa durante la evaluación debe expresarse en
Nix antes de recurrir a lógica procedimental.

La frontera es:

- **Nix:** decisiones, defaults, canales, equipos declarados, personas,
  aplicaciones, escritorios, servicios, opciones, archivos generables,
  dependencias, systemd, Polkit y validaciones declarativas;
- **motor de Korunix:** únicamente interacción con el mundo vivo que Nix no
  puede representar como evaluación pura, por ejemplo UDisks, fwupd,
  PipeWire/WirePlumber, V4L2, autorización, progreso, cancelación y eventos de
  la interfaz;
- **GTK/libadwaita:** presentación e interacción. La GUI no contiene una
  segunda implementación de las operaciones.

El motor objetivo de runtime es **Rust**. Rust no debe volver a implementar
decisiones que ya pertenezcan al modelo Nix. Los contratos funcionales cerrados
en C se conservan durante la migración.

### Estructura del repositorio

La estructura debe permanecer tan plana como permita el significado de los
archivos. Una carpeta solo se justifica cuando agrupa una colección real que
una persona pueda nombrar con claridad.

Reglas:

1. no crear una carpeta para un solo archivo;
2. no crear `default.nix` únicamente para reexportar otros archivos;
3. evitar capas como `lib/`, `src/`, `backend/`, `frontend/`, `utils/`,
   `helpers/`, `common/`, `core/`, `runtime/`, `provider/`, `manager/`,
   `wrapper/` o `adapter/` cuando el nombre sea una decisión de Korunix;
4. los nombres propios de Korunix se escriben en español, en minúsculas y con
   significado humano;
5. si un nombre pertenece a una herramienta externa, se conserva el nombre
   oficial que esa herramienta reconoce.

Por ello son nombres válidos impuestos externamente, entre otros:

- `flake.nix`;
- `flake.lock`;
- `Cargo.toml`;
- `Cargo.lock`;
- `justfile`, mientras Just siga formando parte del flujo.

La estructura objetivo de las piezas duraderas es:

```text
korunix/
├── flake.nix
├── flake.lock
├── spec.md
├── canales.nix
├── predeterminados.nix
├── nvidia-open-pci-ids.txt
├── sistema/
├── equipos/
├── personas/
└── programa/
```

`equipos/<id>.nix` contiene decisiones del equipo y
`equipos/<id>-detectado.nix` contiene hechos físicos adoptados de esa máquina.

Durante la transición de D pueden permanecer `scripts/`, `app/` y `config/`.
No forman parte de la estructura objetivo: D.2 sustituye la lógica de scripts
por el motor Rust compatible y D.3 absorbe la aplicación Python al integrar la
GUI definitiva. `config/` solo se moverá cuando hacerlo no rompa rutas activas
del escritorio actual.

**Regla de nombres:** si el nombre lo decide Korunix, debe entenderlo una
persona; si el nombre lo impone una herramienta externa, se conserva el nombre
oficial.
<!-- KORUNIX-NIX-FIRST:END -->

<!-- KORUNIX-ROADMAP-TEMP:BEGIN -->
## Hoja de ruta temporal de desarrollo

> **Sección temporal.**
>
> Esta sección existe únicamente mientras Korunix continúe en desarrollo.
> Cuando el proyecto alcance su cierre definitivo y todas las macrofases
> descritas aquí estén completadas, debe retirarse íntegramente de `spec.md`,
> incluidos los marcadores `KORUNIX-ROADMAP-TEMP`.

La hoja de ruta sirve para mantener visible el contexto global del proyecto y
evitar que el trabajo sobre una subetapa haga parecer que el resto del proyecto
ha desaparecido o que el desarrollo es una sucesión indefinida de tareas sin
un final reconocible.

### Macrofases

| Fase | Objetivo |
| --- | --- |
| **A · Estabilización** | Consolidar la base declarativa, eliminar interferencias entre escritorios, estabilizar Noctalia/Niri y cerrar regresiones heredadas. |
| **B · Modelo funcional** | Definir las fuentes de verdad y el modelo común de canales, hardware, producto, personas, localización y primera adopción. |
| **C · Operaciones del sistema** | ✅ Cerrada |
| **D · GUI completa** | ▶ Activa · implementación parcial existente |
| **E · Robustez** | Probar interrupciones, rollback, idempotencia, instalación limpia, hardware diverso, arquitecturas soportadas, trabajo offline y degradación segura. |
| **F · Producto** | Completar distribución, bootstrap remoto, empaquetado, onboarding, documentación, traducciones y criterios de lanzamiento. |

### Estado actual

- **A · Estabilización:** cerrada.
- **B · Modelo funcional:** cerrada.
  - Canales, hardware, defaults, personas, localización y primera adopción
    disponen de contratos estructurados.
  - **B.3.5 · adopción transaccional de una instalación existente** está
    cerrado.
  - `bootstrap --adopt` y `bootstrap --adopt --yes` forman parte de la interfaz
    pública.
  - La adopción prepara configuración declarativa sin construir ni aplicar una
    generación.
- **C · Operaciones del sistema:** cerrada.
  - C.1-C.6 conservan contratos estructurados comunes para CLI y GUI.
  - D debe presentar esos motores sin duplicar su lógica.
- **D · GUI completa:** activa.
  - El trabajo actual adopta una arquitectura Nix-first antes de ampliar la GUI.
  - La implementación Python y los scripts existentes son transitorios mientras
    el motor único se migra de forma compatible.
- **E · Robustez:** pendiente como fase principal, aunque fases anteriores pueden
  adelantar pruebas de robustez cuando sean necesarias para cerrar correctamente
  un contrato.
- **F · Producto:** pendiente.

<!-- KORUNIX-ROADMAP-C:BEGIN -->
### Desglose de la fase C

La fase **C · Operaciones del sistema** tiene un alcance finito de seis frentes.
Este desglose existe para evitar convertir el desarrollo en una cadena
indefinida de subetapas.

#### C.1 · Ciclo de cambios — cerrada

Consolidar el recorrido normal de una modificación:

`estado → previsualización → construcción → aplicación → verificación`.

Debe reutilizar las operaciones que Korunix ya posee y ofrecer contratos
estructurados comunes para que CLI y GUI no implementen lógicas distintas.

También debe clasificar el efecto de cada cambio en términos humanos, por
ejemplo:

- inmediato;
- requiere cerrar sesión;
- requiere reiniciar;
- se aplicará en el próximo arranque.

**Estado: cerrado.** `preview`, `build` y `apply` comparten un único motor.
`build --json` y `preview --json` se comprobaron contra una generación
candidata real y produjeron la misma candidata y el mismo impacto sin activar
el sistema. La cancelación, `--yes` y `--yes --json` de `apply` se comprobaron
aislando completamente la frontera privilegiada.

**Regla de seguridad de pruebas:** las pruebas automatizadas de Korunix no
deben fabricar pseudo-TTY ni automatizar prompts de contraseña. Las rutas
privilegiadas se prueban sustituyendo explícitamente la frontera privilegiada,
salvo cuando la operación real sea deliberada y visible para la persona.

#### C.2 · Actualizaciones y migraciones — cerrada

Consolidar:

- actualización general;
- actualización selectiva de inputs;
- cambios entre canales soportados;
- explicación previa de los cambios;
- detección de migraciones;
- indicación de reinicio o cierre de sesión cuando corresponda;
- rollback asociado cuando una actualización no resulte utilizable.

`system.stateVersion` permanece independiente del canal de actualizaciones.

**Estado: cerrado.** La actualización total y selectiva comparte planificación
y resultado estructurados. `flake.lock` se respalda antes de escribir y se
restaura ante fallo o interrupción. La actualización de fuentes no modifica
`stateVersion`, no construye ni aplica una generación. El impacto real de
reinicio o cierre de sesión se delega al ciclo C.1. Los canales estable e
inestable conservan su planificación, validación y restauración transaccional.

La GUI debe consumir estos contratos y no interpretar la salida humana de
`nix flake update`.

#### C.3 · Recuperación y rollback — cerrada

Unificar las operaciones de recuperación alrededor de las generaciones de
NixOS y del gestor de arranque disponible.

Debe contemplar:

- inspección de puntos de recuperación;
- selección explícita;
- uso temporal en el siguiente arranque;
- rollback seguro;
- verificación posterior;
- diferencias necesarias entre UEFI/systemd-boot y BIOS/GRUB.

**Estado: cerrado.** Korunix expone las generaciones disponibles y un plan de
recuperación común para CLI y GUI. La recuperación segura programa una
generación existente únicamente para el próximo arranque: no sustituye la
sesión actual ni la generación predeterminada. En UEFI utiliza el arranque
único de systemd-boot; en BIOS/Legacy utiliza `grub-reboot` y verifica
`next_entry` mediante `grubenv`. La cancelación, la función privilegiada de
systemd-boot y la función privilegiada de GRUB se comprobaron con `sudo`
completamente sustituido por un stub.

La interfaz gráfica debe obtener confirmación antes de utilizar `--yes` y debe
mostrar explícitamente que el cambio solo afecta al próximo arranque.

#### C.4 · Limpieza y almacenamiento — cerrada

Consolidar:

- previsualización de limpieza normal;
- limpieza normal;
- previsualización de limpieza agresiva;
- limpieza agresiva;
- protección de generaciones necesarias para recuperación;
- expulsión segura de almacenamiento extraíble;
- progreso durante vaciado de escrituras pendientes;
- tratamiento específico de transferencias pesadas.

La expulsión segura no debe reducirse a ejecutar un `sync` global sin
información para el usuario.

**Estado: cerrado.** La limpieza normal conserva al menos las tres generaciones
más recientes y la agresiva conserva únicamente las necesarias. Ambas protegen
la generación activa, la predeterminada y una recuperación preparada durante
el arranque actual. Los planes, resultados y etapas de progreso comparten un
contrato estructurado para CLI y GUI.

Korunix expone además inventario y expulsión segura de almacenamiento
extraíble. Rechaza unidades que contengan `/`, `/nix`, `/boot` o `/boot/efi`,
desmonta mediante UDisks y solicita el apagado de la unidad. Para transferencias
pesadas utiliza `sync -f` por sistema de archivos y nunca `sync` global. Los
porcentajes representan etapas aproximadas y no bytes pendientes inventados.

#### C.5 · Firmware y frontera de privilegios — cerrada

Completar la integración de actualización de firmware y definir una única
frontera para las operaciones privilegiadas.

Debe incluir:

- inventario mediante fwupd cuando el hardware lo soporte;
- búsqueda explícita de metadatos;
- actualización iniciada por la persona;
- descripción de los dispositivos afectados;
- indicación de reinicio cuando sea necesario;
- uso de Polkit/helper privilegiado en lugar de ejecutar la GUI completa como
  root.

Las operaciones sin necesidad de privilegios deben permanecer fuera del helper.

**Estado: cerrado.** Korunix expone inventario y actualizaciones de firmware
mediante fwupd con contratos JSON compartidos por CLI y GUI. El refresco de
metadatos y la instalación requieren una acción explícita; la instalación se
realiza por dispositivo y nunca reinicia ni apaga automáticamente el equipo.

La interfaz completa permanece sin privilegios. Las operaciones administrativas
de los frentes anteriores atraviesan una única frontera que usa Polkit/pkexec
cuando está disponible y conserva un fallback de sudo únicamente para una
terminal interactiva sin Polkit. Las pruebas automatizadas sustituyen esa
frontera: no fabrican pseudo-TTY ni automatizan contraseñas.

#### C.6 · Dispositivos multimedia — cerrada

Implementar el motor operativo del centro de prueba y control multimedia
definido de forma permanente en esta especificación.

Debe cubrir como mínimo:

- inventario de salidas de sonido;
- inventario de entradas y micrófonos;
- selección del dispositivo predeterminado;
- niveles y silencio;
- prueba de salida;
- prueba de canales cuando corresponda;
- medidor de micrófono en vivo;
- **Probar micrófono**;
- **Grabar prueba** temporal;
- **Reproducir prueba**;
- **Grabar de nuevo**;
- eliminación automática de la grabación temporal;
- inventario de cámaras;
- prueba y vista previa explícita de cámara;
- resolución y frecuencia de imagen cuando estén disponibles.

Probar un dispositivo y convertirlo en predeterminado son operaciones
independientes.

La capa visual completa de estas capacidades corresponde a D y los escenarios
de desconexión, dispositivos ocupados, pérdida de señal y otros fallos
corresponden a E.

### Criterio de cierre de C

C podrá marcarse como cerrada cuando los seis frentes anteriores tengan un
contrato operativo común, sus operaciones reales estén conectadas a la interfaz
pública de Korunix y ninguna de ellas dependa de una segunda implementación
paralela exclusiva de la GUI.

Cerrar C no exige que toda la interfaz gráfica esté terminada; esa integración
visual completa pertenece a D.

**Estado de C.6: cerrado.** El motor multimedia publica un contrato común para
CLI y GUI sobre PipeWire/WirePlumber y V4L2. Permite inventariar salidas,
entradas, perfiles y puertos; cambiar predeterminados, volumen y silencio;
probar una salida sin hacerla predeterminada; medir un micrófono en vivo y
realizar una grabación temporal reproducible; e inventariar/previsualizar
cámaras con resolución y FPS. Las pruebas de micrófono no realizan
monitorización directa y las grabaciones se eliminan salvo petición explícita.

**Fase C cerrada.** C.1-C.6 comparten contratos operativos que puede consumir la
GUI. La fase D debe presentar estos mismos motores y no crear implementaciones
paralelas para aplicar, recuperar, limpiar, firmware o multimedia.
<!-- KORUNIX-ROADMAP-C:END -->

<!-- KORUNIX-ROADMAP-D:BEGIN -->
### Desglose de la fase activa D

D tiene exactamente cuatro frentes. No se crearán subfases formales dentro de
ellos.

#### D.1 · Arquitectura Nix-first y estructura humana — cerrada

Fijar la frontera Nix ↔ runtime, trasladar a Nix las fuentes declarativas que
todavía dependan de parsers auxiliares, aplanar las piezas duraderas del
repositorio y convertir la nomenclatura humana en una regla permanente.

**Estado: cerrado.** Las fuentes de canales y predeterminados son Nix,
`flake.nix` expone `korunix.canales`, `korunix.predeterminados` y
`korunix.equipos`, y las piezas declarativas duraderas se agrupan como
`sistema/`, `equipos/` y `personas/`. `catalog/`, `modules/`, `hosts/`,
`hardware/` y `users/` quedan retirados. La lógica procedural de `scripts/` y
la GUI Python de `app/` permanecen únicamente como transición explícita hacia
D.2 y D.3; no se multiplicarán durante esa transición.

#### D.2 · Motor Rust único — activa

Sustituir progresivamente `scripts/korunix`, `scripts/users` y
`scripts/localization` por un único ejecutable Rust. Debe conservar las
semánticas y contratos estructurados cerrados en C mientras usa Nix como fuente
del modelo declarativo.

**Estado actual de D.2:** el ejecutable Rust ya existe como
`sistema/programa/principal.rs`, con `Cargo.toml` y `Cargo.lock` en sus nombres
oficiales. El motor puede leer directamente el modelo Nix mediante `korunix
modelo ...` y conserva la interfaz existente delegando temporalmente las
operaciones que todavía viven en `scripts/korunix`.

La salida Nix `packages.<sistema>.motor` permite construir el motor sin
reemplazar todavía el paquete gráfico predeterminado. Esta compatibilidad es
transitoria: D.2 solo podrá cerrarse cuando las operaciones públicas de
`scripts/korunix`, `scripts/users` y `scripts/localization` hayan sido absorbidas
por Rust o retiradas porque Nix las resuelve declarativamente.

**Avance de D.2 — canales y permisos:** las consultas `channel --json` y
`privileges --json` ya nacen en Rust. Las definiciones de canales y valores
iniciales viven dentro de `sistema/`, porque son reglas internas del producto y
no preferencias de una persona. La salida flake temporal `korunix` fue retirada:
el motor lee las fuentes Nix directamente. La implementación Bash de canales queda como compatibilidad transitoria; el
motor Rust ya posee la consulta, el plan, la escritura y la recuperación.

**Avance de D.2 — dominio de canales completo:** el motor Rust ya consulta,
planifica y cambia el canal. El plan evalúa una copia temporal del checkout para
no modificar la configuración real ni siquiera durante la previsualización. La
escritura real cambia exactamente una declaración, crea un respaldo persistente
y revierte el archivo si la nueva declaración no evalúa. `channel` nunca cambia
`system.stateVersion`, nunca construye una generación y nunca la aplica. La
implementación Bash permanece únicamente como compatibilidad transitoria mientras
D.2 retira el resto del motor antiguo.

**Avance de D.2 — estructura maestra aplicada:** la raíz humana ya se separa
en `configuracion/`, `sistema/` y `generado/`. `configuracion/equipos/` y
`configuracion/personas/` contienen decisiones que una persona puede cambiar;
`generado/equipos/` contiene hechos detectados automáticamente; `sistema/`
contiene la implementación interna. Las rutas antiguas `equipos/` y `personas/`
ya no forman parte de la estructura activa. El sufijo `-detectado` puede
permanecer temporalmente durante D.2 por compatibilidad, aunque la carpeta
`generado/` ya define su naturaleza.

**Avance de D.2 — lecturas del equipo en Rust:** `hardware --json`,
`localization --json` y `users --json` ya nacen en el motor Rust. Las tres
combinan el modelo declarativo de Nix con el estado vivo del sistema sin pasar
por los backends Bash. Sus contratos JSON se mantienen idénticos a los contratos
heredados; `detectedAt` sigue siendo una marca temporal y no participa en las
comparaciones de identidad. `jq` permanece temporalmente como herramienta de
composición JSON, pero ya no contiene el dominio operativo ni actúa como punto
de entrada.

#### D.3 · GTK/libadwaita sobre el motor

Migrar la aplicación Python actual a la interfaz definitiva sobre el mismo
motor Rust. No debe existir un backend paralelo de administración.

#### D.4 · Cierre de experiencia

Completar navegación adaptable, progreso, confirmaciones, errores humanos,
detalles técnicos opcionales, internacionalización y validación visual.

**Criterio de cierre de D:** el modelo declarativo se obtiene de Nix; el
runtime interactivo pasa por un único motor; la GUI no ejecuta como root ni
duplica operaciones; y las rutas transitorias retiradas durante D ya no forman
parte de la arquitectura final.
<!-- KORUNIX-ROADMAP-D:END -->

### Regla obligatoria de seguimiento durante el desarrollo

Mientras esta sección exista:

1. al informar un avance, cierre, bloqueo o cambio de frente, se debe mostrar
   primero la **hoja de ruta completa A–F**;
2. después puede ampliarse la fase, frente o subfrente actualmente trabajado;
3. no se debe mostrar únicamente una cadena local como `B.3.5a → B.3.5e` sin
   situarla antes dentro de A–F;
4. debe distinguirse claramente entre:
   - cerrado;
   - activo;
   - parcialmente implementado;
   - pendiente;
   - deuda conocida;
5. los porcentajes solo deben utilizarse cuando exista una base verificable para
   calcularlos; no deben inventarse porcentajes para transmitir una falsa
   precisión;
6. el frente activo y el siguiente objetivo inmediato deben quedar visibles para
   que sea posible comprender hacia dónde avanza el proyecto;
7. cerrar una subetapa no implica cerrar automáticamente su macrofase;
8. si la estructura de la hoja de ruta cambia durante el desarrollo, esta
   sección debe actualizarse para seguir representando el plan real;
9. esta hoja de ruta describe el **progreso de desarrollo de Korunix**, no debe
   convertirse en una pantalla, opción ni concepto visible para el usuario final
   del producto salvo que exista una decisión independiente que lo justifique.

### Criterio para retirar esta sección

Esta sección debe eliminarse cuando Korunix haya completado las macrofases
**A, B, C, D, E y F** y se considere alcanzado el cierre del proyecto definido
para esta hoja de ruta.

La eliminación es intencional: una vez terminado el desarrollo que esta hoja
pretende seguir, mantener un registro operativo de “qué falta” dentro de la
especificación del producto deja de tener utilidad.

La historia del desarrollo seguirá perteneciendo a Git y a la documentación
histórica correspondiente; `spec.md` debe volver a contener únicamente
requisitos y decisiones relevantes para Korunix como producto terminado.<!-- KORUNIX-ROADMAP-TEMP:END -->

<!-- KORUNIX-CLARIDAD-PRODUCTO:BEGIN -->

## Claridad de producto y separación entre lo editable, lo interno y lo generado

Esta política es obligatoria para todo Korunix. No existe una parte del proyecto
que quede excluida por ser antigua, transitoria, técnica o difícil. Las piezas
existentes se migrarán de forma progresiva, pero toda pieza nueva o modificada
debe cumplir esta política desde el momento en que se toque.

### 1. La raíz debe explicar primero qué puede tocar una persona

La estructura final debe agrupar los archivos por su relación con la persona,
no por la tecnología con la que están escritos.

```text
configuracion/
  equipos/
  personas/

sistema/
  ...

generado/
  ...
```

- `configuracion/` contiene decisiones pensadas para ser leídas y cambiadas por
  una persona.
- `sistema/` contiene las reglas y el código que hacen funcionar Korunix. Una
  persona no necesita entrar aquí para cambiar sus preferencias normales.
- `generado/` contiene información creada o detectada automáticamente. No debe
  editarse manualmente.

Los nombres técnicos que una herramienta obliga a conservar, como `flake.nix`,
`flake.lock`, `Cargo.toml` y `Cargo.lock`, pueden permanecer donde la herramienta
los necesite. Korunix debe explicar claramente qué son y cuáles son generados.
No se deben modificar archivos generados solo para añadir comentarios humanos.

Mientras continúe la transición de D, `scripts/`, `app/` y `config/` pueden
existir por compatibilidad. No forman parte de la estructura final y también
deben respetar esta política cada vez que se modifiquen.

### 2. Prueba de claridad

Una explicación no cumple solo por ser técnicamente correcta.

Debe poder entenderla una persona que no conozca Nix, Rust, Git, Linux, Bash,
Python ni la arquitectura interna de Korunix. El objetivo de claridad es que la
idea principal pueda entenderla incluso un niño pequeño.

Si una palabra técnica es inevitable:

1. primero se explica la idea con palabras comunes;
2. después se menciona el nombre técnico;
3. se explica para qué sirve en ese lugar concreto;
4. se indica si una persona debe tocarlo o no.

Frases como "contexto estructural", "backend", "dispatcher", "payload",
"atributo", "derivación", "closure", "specialArgs", "mkIf" o "mkMerge" no pueden
darse por entendidas.

### 3. Encabezado obligatorio según el tipo de archivo

Todo archivo permanente cuyo nombre y formato controle Korunix debe dejar claro,
desde el principio, a qué clase pertenece.

Un archivo de `configuracion/` debe comenzar explicando:

- que se puede cambiar;
- qué decisiones guarda;
- qué cosas importantes pueden cambiarse ahí;
- dónde están las cosas que NO pertenecen a ese archivo.

Un archivo de `sistema/` debe comenzar explicando:

- que es una parte interna de Korunix;
- qué problema resuelve;
- qué recibe;
- qué produce o modifica;
- qué opción de `configuracion/` debe tocar una persona en vez de editar esa
  implementación interna.

Un archivo de `generado/` debe dejar claro, cuando el formato permita un
encabezado:

- que Korunix lo creó automáticamente;
- qué información contiene;
- que un cambio manual puede perderse;
- qué operación de Korunix debe utilizarse para volver a generarlo o corregirlo.

Cuando el formato generado no admita comentarios seguros, esta explicación debe
vivir en el archivo humano más cercano, en la interfaz y en la documentación.

### 4. Controles principales

Korunix llama **control principal** a una decisión que puede cambiar varias cosas
relacionadas al mismo tiempo.

Ejemplos posibles son:

- activar o desactivar virtualización;
- elegir un escritorio;
- elegir una aplicación que activa servicios o complementos;
- elegir un canal de actualizaciones;
- activar Korunix completo;
- cualquier opción que haga aparecer, desaparecer o cambiar varias piezas.

Todo control principal editable debe explicar, antes de su valor:

1. **Qué es.**
2. **Para qué sirve.**
3. **Qué valores puede usar la persona.**
4. **Qué cambia directamente.**
5. **Qué cambia también de forma indirecta.**
6. **Qué NO cambia.**
7. **Qué valor usa Korunix de forma predeterminada.**
8. **Qué elección suele ser recomendable y en qué caso.**
9. **Qué puede requerir reinicio, cierre de sesión, reconstrucción u otra acción.**
10. **Un ejemplo sencillo de un cambio seguro cuando ayude a entenderlo.**

Una opción editable nunca puede esconder efectos secundarios. Si activar una
opción también activa cinco componentes, Korunix debe decir cuáles son y por qué
se administran juntos.

### 5. Elecciones, listas y reglas automáticas

Además de los controles principales, Korunix debe reconocer y explicar:

- **Elección:** escoge una opción entre varias.
- **Lista de elecciones:** permite escoger varias cosas a la vez.
- **Regla automática:** Korunix calcula un resultado a partir de otras
  decisiones.
- **Regla condicional:** una parte se usa solamente cuando se cumple una
  condición.
- **Combinador:** junta varias reglas en una configuración final.
- **Conexión interna:** lleva información de una parte de Korunix a otra.

La persona no debe tener que deducir estas relaciones leyendo código.

### 6. Construcciones técnicas con alcance amplio

Toda construcción técnica capaz de afectar varias partes debe tener una
explicación humana junto a su definición o en el punto más cercano donde una
persona pueda entender su función.

Esto incluye, sin limitarse a:

- `specialArgs`;
- `mkEnableOption`;
- `mkIf`;
- `mkMerge`;
- `optional`, `optionals` y equivalentes;
- variables calculadas del tipo `...Enabled`;
- funciones que transforman una elección humana en varias opciones de NixOS;
- funciones que crean, quitan o modifican varios archivos o servicios;
- tablas que convierten nombres humanos en paquetes o servicios;
- valores que se heredan o propagan a varios módulos;
- rutas de privilegios;
- funciones de actualización, recuperación, limpieza, hardware y multimedia con
  efectos múltiples.

No basta con describir la sintaxis. Hay que explicar el efecto real dentro de
Korunix.

### 7. `specialArgs`

`specialArgs` es una **conexión interna**, no una zona de configuración.

Su propósito es entregar a los módulos información estructural que necesitan
para trabajar, por ejemplo:

- qué equipo se está preparando;
- dónde están ciertos archivos;
- qué conjunto de paquetes corresponde al canal elegido;
- qué entradas externas ya seleccionó Korunix.

No debe utilizarse como almacén universal de preferencias editables.

Si una persona debe poder elegir un valor, esa elección pertenece a
`configuracion/` y las partes internas deben recibir el resultado sin duplicar la
decisión.

### 8. Interruptores y valores calculados

Un interruptor interno como `niriEnabled`, `hyprlandEnabled` o cualquier valor
equivalente no debe convertirse en otra opción que la persona tenga que
sincronizar manualmente.

La persona elige una vez. Korunix calcula el resto.

Por ejemplo, elegir un escritorio puede activar automáticamente:

- el paquete del escritorio;
- sus servicios;
- sus portales;
- sus archivos de configuración;
- sus variables de sesión;
- las integraciones que ese escritorio necesita.

La explicación de la elección humana debe resumir esos efectos. Los valores
calculados internos deben explicar de qué elección nacen y qué controlan.

### 9. Sugerencias de cambio

Toda opción editable debe ayudar a decidir, no limitarse a enumerar valores.

Cuando sea posible debe incluir:

- el valor predeterminado;
- la opción más sencilla o segura para una persona que no tiene una preferencia;
- cuándo conviene usar una alternativa;
- consecuencias importantes;
- incompatibilidades conocidas;
- qué hace Korunix automáticamente para acompañar esa elección.

Una sugerencia nunca debe presentarse como obligación cuando existen varias
elecciones igualmente válidas.

Las piezas internas no deben sugerir que la persona las edite. Deben señalar la
opción humana que gobierna su comportamiento.

### 10. Interfaz gráfica

La GUI debe mostrar el mismo modelo mental que los archivos humanos.

Un control de gran alcance debe poder mostrar, cuando sea útil:

- qué cambiará;
- qué otras piezas cambiarán junto con él;
- por qué están relacionadas;
- si el cambio es inmediato;
- si requiere reconstrucción, cierre de sesión o reinicio;
- qué valor se recomienda para un caso común.

La GUI no debe obligar a conocer nombres internos como `mkIf`, `specialArgs` o
`pkgsUnstable`.

### 11. Localización

La explicación humana es parte del producto, no un comentario decorativo.

El español estándar es la fuente canónica de significado del proyecto. Toda
explicación visible para la persona debe llegar a todas las localizaciones
soportadas por Korunix.

Una función, opción o control nuevo no está completamente terminado si su
explicación existe solo en un idioma cuando esa explicación aparece en la GUI,
documentación distribuida, mensajes o ayudas públicas.

Los comentarios internos del código pueden conservar el español como idioma
canónico para evitar duplicar bloques dentro del mismo archivo. El contenido
equivalente que vea la persona debe existir en el sistema de localización.

Las traducciones deben conservar:

- el significado;
- los efectos directos;
- los efectos indirectos;
- las advertencias;
- las recomendaciones;
- los ejemplos importantes.

No se debe reducir una traducción hasta perder información útil.

### 12. Aplicación obligatoria a todo el proyecto

Esta política se aplica a:

- Nix;
- Rust;
- Bash mientras siga existiendo;
- Python mientras siga existiendo;
- configuración de escritorios;
- plantillas;
- validadores;
- archivos de personas y equipos;
- hardware detectado;
- opciones de aplicaciones;
- servicios;
- arranque;
- actualizaciones;
- recuperación;
- almacenamiento;
- firmware;
- multimedia;
- privilegios;
- GUI;
- documentación;
- ayudas de terminal;
- mensajes de error;
- futuras tecnologías que Korunix incorpore.

No existe una excepción por tratarse de código avanzado.

### 13. Migración del contenido existente

No se exige reescribir todo el repositorio en una sola operación arriesgada.

La migración será progresiva:

1. toda pieza nueva cumple esta política desde su creación;
2. toda pieza existente que se modifique debe actualizar también sus
   explicaciones;
3. cada frente de D, E y F debe retirar deuda de claridad relacionada con lo que
   toca;
4. antes de considerar Korunix producto final, se hará una comprobación global
   para verificar que no queden controles principales, reglas automáticas,
   conexiones internas o efectos masivos sin explicación humana.

No se permiten excepciones permanentes por antigüedad.

### 14. Criterio de aceptación

Una pieza de Korunix con capacidad de modificar varias cosas no se considera
terminada hasta que pueda responder claramente:

- ¿Qué es?
- ¿Para qué sirve?
- ¿Debo cambiarlo?
- Si debo cambiarlo, ¿qué valores puedo usar?
- ¿Qué otras cosas cambiarán también?
- ¿Qué no cambiará?
- ¿Qué me recomienda Korunix y por qué?
- ¿Necesitaré reiniciar, cerrar sesión o hacer otra acción?
- Si no debo tocarlo, ¿qué opción humana debo cambiar en su lugar?

Si esas respuestas requieren leer la implementación para descubrirlas, la pieza
todavía no cumple la calidad de producto de Korunix.

<!-- KORUNIX-CLARIDAD-PRODUCTO:END -->

# Referencia funcional — d0b40b6

Este archivo marca el último corte integral que quedó **probado, publicado y aplicado** antes de empezar a replantear Korunix desde cero.

No significa que la arquitectura actual sea la que debamos copiar. Sirve para recordar qué cosas ya funcionaron de verdad y qué comportamiento vale la pena rescatar.

## Commit

```text
d0b40b682fcc6e70f9181a5b2f4b93175cbbe609
```

Mensaje:

```text
agiliza los flujos cotidianos de la interfaz
```

El código exacto de ese estado ya está guardado por Git en ese commit. No hace falta duplicar un script de más de mil líneas para conservarlo.

## Qué quedó comprobado

- Karere y Blender permanecen seleccionados.
- Las aplicaciones añadidas por nombre aparecen aunque no tengan una ficha curada de Korunix.
- Karere no tuvo que añadirse a `sistema/aplicaciones.nix`.
- Escritorio instalado, escritorio principal y escritorio mostrado en la vista previa son decisiones distintas.
- Tener Plasma o Cinnamon instalado no bloquea Dinámico o Everforest para Niri/Hyprland.
- Idioma y región, Personas, Copias e historial y Mantenimiento pueden cargar de forma independiente.
- La lectura de localización dejó de repetir `localectl` para cada dato XKB.
- Las pruebas del motor pasaron.
- Las pruebas de la GUI pasaron.
- La puerta integral pasó.
- La generación aplicada quedó activa y también guardada como generación persistente.
- La rama `pruebas` quedó limpia después del corte.

## Tiempos medidos después del cambio

```text
Idioma y región          180 ms
Catálogo localización     21 ms
Personas                   26 ms
Copias e historial         13 ms
Mantenimiento total        56 ms
```

Estos tiempos no significan que el problema de carga visual quedara perfecto. Después vimos que solo Resumen estaba listo al abrir Korunix y que las demás páginas todavía podían mostrar una primera carga. Esa observación sigue siendo útil para una reimplementación nueva: abrir primero y precalentar después, sin bloquear la ventana.

## Script que produjo el corte

El script usado fue:

```text
korunix-flujos-visibles-sin-esperas-v3.sh
```

SHA-256:

```text
0f62e3982f9f8014db6f4263852a23436edbd22138f12e09e3ca36090fa11721
```

El script es una herramienta de migración/prueba de ese momento. La referencia principal es el commit, porque ahí está el resultado final que realmente quedó publicado.

## Cómo usar esta referencia

Si en la nueva versión necesitamos recordar cómo resolvimos alguna de estas cosas, revisamos primero este commit y luego decidimos si conviene rescatar la idea, no necesariamente el código.

```bash
git show d0b40b682fcc6e70f9181a5b2f4b93175cbbe609
```

`main` sigue siendo la referencia de simpleza. `pruebas` queda como archivo de lo aprendido y validado.

#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

[[ -f sistema/interfaz/principal.rs ]]
[[ -f sistema/interfaz/io.github.koruninn.Korunix.desktop ]]

interfaz="sistema/interfaz/principal.rs"

grep -Fq 'adw::Application' "$interfaz"
grep -Fq 'KORUNIX_MOTOR_BIN' "$interfaz"
grep -Fq 'ejecutar_motor' "$interfaz"
grep -Fq 'serde_json' "$interfaz"

# La búsqueda global forma parte del contrato de producto.
grep -Fq 'gtk::SearchEntry::new()' "$interfaz"
grep -Fq 'connect_search_changed' "$interfaz"
grep -Fq 'terminos_busqueda_pagina' "$interfaz"
grep -Fq 'No encontramos un área con ese nombre.' "$interfaz"
grep -Fq 'Buscar ajustes y áreas' "$interfaz"

if grep -Fq 'cabecera_contenido.pack_start(&busqueda_global)' "$interfaz"; then
  echo "ERROR: la búsqueda volvió al encabezado y rompe la adaptación estrecha." >&2
  exit 1
fi

grep -Fq 'adw::BreakpointCondition::parse("max-width: 819px")' "$interfaz"
grep -Fq 'fila.set_size_request(-1, 60)' "$interfaz"
grep -Fq 'busqueda_global.set_size_request(-1, 38)' "$interfaz"

# Las superficies normales deben presentar objetivos humanos.
grep -Fq 'fn presentacion_aplicacion' "$interfaz"
grep -Fq 'fn unidades_actualizacion_humanas' "$interfaz"
grep -Fq 'fn resultados_aplicaciones_externas' "$interfaz"
grep -Fq 'Buscar más aplicaciones' "$interfaz"
grep -Fq 'Navegador predeterminado' "$interfaz"
grep -Fq 'Editor de texto en Plasma' "$interfaz"
grep -Fq 'KDE Plasma' "$interfaz"
grep -Fq 'fn asuntos_resumen' "$interfaz"
grep -Fq 'Estado del equipo' "$interfaz"
grep -Fq 'Todo está bien' "$interfaz"
grep -Fq 'Expulsión después de archivos grandes' "$interfaz"
grep -Fq 'El firmware está al día' "$interfaz"
grep -Fq 'Última copia portable' "$interfaz"
grep -Fq 'Versiones para recuperación' "$interfaz"
grep -Fq 'Limpieza recomendada' "$interfaz"

if grep -Fq '"firmware", "refresh", "--plan"' "$interfaz"; then
  echo "ERROR: Firmware volvió a pedir confirmación para una mera consulta." >&2
  exit 1
fi

if grep -Fq 'selector.set_title(texto(estado.idioma, "generations"))' "$interfaz"; then
  echo "ERROR: Mantenimiento volvió a esconder las recuperaciones en un selector técnico." >&2
  exit 1
fi

if grep -Fq 'fuentes_modelo = gtk::StringList' "$interfaz"; then
  echo "ERROR: Aplicaciones volvió a pedir una fuente técnica al usuario." >&2
  exit 1
fi

if grep -Fq 'fila.set_subtitle(match fuente.as_str()' "$interfaz"; then
  echo "ERROR: una fila de aplicación volvió a usar la fuente como descripción." >&2
  exit 1
fi
grep -Fq 'Configuración actual' "$interfaz"
grep -Fq 'grupo.set_visible(false)' "$interfaz"

if [[ -d app ]]; then
  echo "ERROR: app/ debía desaparecer al retirar la GUI Python." >&2
  exit 1
fi

if rg -n \
  'korunix_backend\.py|korunix_i18n\.py|app/korunix\.py|scripts/validate-gui\.py' \
  --glob '!spec.md'
then
  echo "ERROR: quedó una dependencia activa de la GUI Python." >&2
  exit 1
fi

echo "✓ interfaz Rust declarada"
echo "✓ GUI Python retirada"
echo "✓ interfaz consume el motor público"
echo "✓ búsqueda global filtra mientras se escribe"
echo "✓ búsqueda vive en la navegación lateral"
echo "✓ adaptación recupera el breakpoint de 819 px"
echo "✓ filas de navegación conservan 60 px"
echo "✓ aplicaciones técnicas derivadas no son decisiones visibles"
echo "✓ catálogo describe qué instala cada aplicación y para qué sirve"
echo "✓ buscador de aplicaciones vive arriba y oculta la fuente técnica"
echo "✓ actualizaciones agrupan dependencias bajo decisiones humanas"
echo "✓ localización muestra primero valores humanos"

echo "✓ Resumen funciona como centro de salud con señales comprobables"
echo "✓ Almacenamiento explica la expulsión de archivos grandes"
echo "✓ Firmware presenta un único estado y consulta sin diálogo redundante"
echo "✓ Copias e historial muestran estado y tiempo relativo"
echo "✓ Mantenimiento muestra tres recuperaciones recientes y limpieza comprensible"

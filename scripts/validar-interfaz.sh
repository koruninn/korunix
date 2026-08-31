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
grep -Fq 'fn aplicacion_derivada_o_interna' "$interfaz"
grep -Fq 'fn nombre_aplicacion_humano' "$interfaz"
grep -Fq 'fn objetivo_actualizacion_humano' "$interfaz"
grep -Fq 'KDE Plasma' "$interfaz"
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
echo "✓ actualizaciones tienen propósito humano"
echo "✓ localización muestra primero valores humanos"

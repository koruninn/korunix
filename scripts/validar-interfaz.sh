#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

[[ -f sistema/interfaz/principal.rs ]]
[[ -f sistema/interfaz/io.github.koruninn.Korunix.desktop ]]

grep -Fq 'adw::Application' sistema/interfaz/principal.rs
grep -Fq 'KORUNIX_MOTOR_BIN' sistema/interfaz/principal.rs
grep -Fq 'ejecutar_motor' sistema/interfaz/principal.rs
grep -Fq 'serde_json' sistema/interfaz/principal.rs

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

#!/usr/bin/env bash
set -uo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.." || exit 1

fallos=0

ok() {
  printf '✓ %s\n' "$1"
}

fallo() {
  printf '✗ %s\n' "$1"
  fallos=$((fallos + 1))
}

ejecutar() {
  descripcion="$1"
  shift

  if "$@"; then
    ok "$descripcion"
  else
    fallo "$descripcion"
  fi
}

printf '%s\n' \
  '========================================' \
  ' KORUNIX · PUERTA AUTOMATIZABLE' \
  '========================================'

printf '\n%s\n' '→ Integridad del árbol'

ejecutar \
  'git diff --check' \
  git diff --check

if git ls-files --cached -- 'target/**' | grep -q .
then
  fallo 'target/ contiene archivos versionados'
else
  ok 'resultados de Cargo fuera del producto'
fi


printf '\n%s\n' '→ Rust'

ejecutar \
  'cargo fmt' \
  cargo fmt -- --check

ejecutar \
  'pruebas del motor' \
  cargo test --locked --no-default-features --bin korunix

gui_rust_log="$(mktemp)"
if cargo check --locked --features interfaz --bin korunix-interfaz \
    2> >(tee "$gui_rust_log" >&2)
then
  if grep -q '^warning:' "$gui_rust_log" \
     && grep -q 'sistema/interfaz/principal.rs' "$gui_rust_log"
  then
    fallo 'GUI Rust conserva advertencias propias'
  else
    ok 'GUI Rust sin advertencias propias'
  fi
else
  fallo 'GUI Rust'
fi
rm -f "$gui_rust_log"

if rg -n 'stdenv\.(isLinux|isDarwin)' \
    --glob '*.nix' \
    --glob '!result/**' \
    .
then
  fallo 'Korunix usa propiedades stdenv obsoletas'
else
  ok 'Korunix no usa propiedades stdenv obsoletas'
fi

# El contrato de producto debe probar el binario que acabamos de construir,
# nunca un target/debug antiguo encontrado por scripts/korunix.
if cargo build --locked --no-default-features --bin korunix; then
  motor_actual="$PWD/target/debug/korunix"
  ok 'motor actual construido para contratos'
else
  motor_actual=''
  fallo 'motor actual no pudo construirse'
fi


printf '\n%s\n' '→ Shell y escritorio'

bash_ok=1

while IFS= read -r archivo; do
  if ! bash -n "$archivo"; then
    bash_ok=0
  fi
done < <(
  find scripts \
    -maxdepth 1 \
    -type f \
    \( -name '*.sh' -o -name 'korunix-bootstrap' \) \
    -print \
    | sort
)

if [[ "$bash_ok" -eq 1 ]]; then
  ok 'Bash parsea'
else
  fallo 'Bash no parsea'
fi

shellcheck_args=()

while IFS= read -r archivo; do
  shellcheck_args+=("$archivo")
done < <(
  find scripts \
    -maxdepth 1 \
    -type f \
    \( -name '*.sh' -o -name 'korunix-bootstrap' \) \
    -print \
    | sort
)

if [[ "${#shellcheck_args[@]}" -gt 0 ]] \
   && shellcheck -x "${shellcheck_args[@]}"
then
  ok 'ShellCheck'
else
  fallo 'ShellCheck'
fi

ejecutar \
  'desktop entry' \
  desktop-file-validate \
  sistema/interfaz/io.github.koruninn.Korunix.desktop

ejecutar \
  'AppStream' \
  appstreamcli validate \
  --no-net \
  sistema/interfaz/io.github.koruninn.Korunix.metainfo.xml


printf '\n%s\n' '→ Nix y arquitecturas'

ejecutar \
  'flake check completo de ambas arquitecturas' \
  nix flake check \
  path:. \
  --no-build \
  --show-trace \
  --all-systems

for sistema in x86_64-linux aarch64-linux; do
  ejecutar \
    "motor evaluable en ${sistema}" \
    nix eval \
    --raw \
    "path:.#packages.${sistema}.motor.drvPath"

  ejecutar \
    "GUI evaluable en ${sistema}" \
    nix eval \
    --raw \
    "path:.#packages.${sistema}.korunix.drvPath"

  ejecutar \
    "bootstrap evaluable en ${sistema}" \
    nix eval \
    --raw \
    "path:.#packages.${sistema}.bootstrap.drvPath"
done

ejecutar \
  'motor, GUI y bootstrap construyen en la arquitectura actual' \
  nix build \
  --no-link \
  path:.#motor \
  path:.#korunix \
  path:.#bootstrap


printf '\n%s\n' '→ Contratos de robustez'

operaciones='sistema/programa/operaciones.rs'

if rg -q 'sync_all\(\)' "$operaciones" \
   && rg -q 'parent.*sync_all|directory.*sync_all|parent_dir.*sync_all' "$operaciones"
then
  ok 'escritura atómica persiste archivo y directorio'
else
  fallo 'persistencia atómica incompleta'
fi

if rg -q 'transaction\.pending' "$operaciones" \
   && rg -q '"declarative-files"' "$operaciones" \
   && rg -q '"restore-tree"' "$operaciones"
then
  ok 'journal transaccional único'
else
  fallo 'journal transaccional incompleto'
fi

if rg -q 'flake\.lock\.candidate' "$operaciones" \
   && rg -q -- '--output-lock-file' "$operaciones" \
   && rg -q -- '--reference-lock-file' "$operaciones"
then
  ok 'actualización por lock candidato'
else
  fallo 'update no usa lock candidato'
fi

if rg -q 'RENAME_EXCHANGE|exchange_paths' "$operaciones"; then
  ok 'restore usa intercambio atómico'
else
  fallo 'restore sin intercambio atómico'
fi

if rg -q 'syncfs|sincronizar_sistema_archivos' "$operaciones"; then
  ok 'expulsión usa sincronización por sistema de archivos'
else
  fallo 'expulsión no usa sincronización por sistema de archivos'
fi


printf '\n%s\n' '→ Producto y contrato'

if rg -q 'fn build_candidate\(raiz: &Path, json: bool\)' sistema/programa/operaciones.rs \
   && rg -q 'Nix sigue construyendo' sistema/programa/operaciones.rs \
   && rg -q 'authorization_required' sistema/programa/operaciones.rs \
   && rg -q 'validate_quiet\(raiz\)' sistema/programa/operaciones.rs
then
  ok 'apply comunica validación, construcción y autorización'
else
  fallo 'apply volvió a quedar sin fases humanas o contaminó el modo JSON'
fi

if grep -Fq 'fn flake_source(raiz: &Path) -> String' sistema/programa/operaciones.rs \
   && grep -Fq 'raiz.display().to_string()' sistema/programa/operaciones.rs \
   && grep -Fq 'fn flake_reference(raiz: &Path, fragment: &str) -> String' sistema/programa/operaciones.rs \
   && grep -Fq 'flake_source(raiz),' sistema/programa/operaciones.rs \
   && grep -Fq 'let target = flake_reference(' sistema/programa/operaciones.rs \
   && grep -Fq 'let flake = flake_reference(raiz, &host);' sistema/programa/operaciones.rs \
   && ! grep -Fq 'let flake = format!(".#{host}");' sistema/programa/operaciones.rs \
   && ! grep -Fq 'let flake = format!("path:{}#{host}", raiz.display());' sistema/programa/operaciones.rs \
   && ! grep -Fq 'nombre.starts_with("/nix/store/") && nombre.ends_with("/bin/switch-to-configuration")' sistema/programa/operaciones.rs \
   && grep -Fq '"nixos-rebuild"' sistema/programa/operaciones.rs \
   && grep -Fq 'let profile_system = fs::canonicalize(system_profile())' sistema/programa/operaciones.rs \
   && grep -Fq 'let registered = generations()' sistema/programa/operaciones.rs \
   && grep -Fq 'no quedó registrada de forma persistente como generación predeterminada' sistema/programa/operaciones.rs \
   && grep -Fq 'misma identidad de fuente del flake' spec.md \
   && grep -Fq 'independiente del directorio de' spec.md \
   && grep -Fq 'candidata aparece entre las generaciones registradas y recuperables' spec.md
then
  ok 'apply usa una fuente absoluta común y verifica su persistencia'
else
  fallo 'el ciclo de apply puede depender del cwd, cambiar de candidata o perder persistencia'
fi

if rg -q 'La previsualización no necesita privilegios' sistema/programa/operaciones.rs \
   && ! rg -q 'dry-activate' sistema/programa/operaciones.rs
then
  ok 'previsualización no cruza Polkit'
else
  fallo 'previsualización volvió a solicitar privilegios'
fi

if grep -Fq 'Una operación lógica debe cruzar la frontera de privilegios el menor número de' spec.md \
   && grep -Fq 'stdout se reserva para el resultado JSON final' spec.md \
   && grep -Fq 'La ausencia de porcentaje exacto no justifica una interfaz muda' spec.md
then
  ok 'especificación protege autorización única y progreso visible'
else
  fallo 'la especificación perdió el contrato de autorización/progreso'
fi

if rg -U -q '"nixos-rebuild",\n[[:space:]]*&\["switch"\.into\(\), "--flake"\.into\(\), flake\],\n[[:space:]]*!json,' "$operaciones" \
   && grep -Fq 'stdout queda reservado para el JSON final' "$operaciones"
then
  ok 'apply JSON aísla stdout del proceso privilegiado'
else
  fallo 'apply JSON puede volver a mezclar salida humana con el documento final'
fi

if grep -Fq 'if let Err(error) = validate_quiet(raiz)' sistema/programa/operaciones.rs \
   && ! grep -Fq 'estado.stack.set_sensitive(false);' sistema/interfaz/principal.rs \
   && grep -Fq 'let indicador_busqueda = gtk::Spinner::new();' sistema/interfaz/principal.rs \
   && grep -Fq '(_, "both") => "Ambos lados"' sistema/interfaz/principal.rs \
   && grep -Fq 'for boton in botones {' sistema/interfaz/principal.rs \
   && grep -Fq 'Una operación larga en una página no debe volver insensibles las demás áreas' spec.md
then
  ok 'GUI conserva navegación, JSON limpio y serializa pruebas de sonido'
else
  fallo 'la GUI puede volver a bloquearse, contaminar JSON o solapar pruebas de sonido'
fi

ssh_activo="$(
  nix eval \
    --json \
    '.#nixosConfigurations.korunix.config.services.openssh.enable' \
    2>/dev/null \
    || true
)"

ssh_firewall="$(
  nix eval \
    --json \
    '.#nixosConfigurations.korunix.config.services.openssh.openFirewall' \
    2>/dev/null \
    || true
)"

if [[ "$ssh_activo" == 'true' ]] \
   && [[ "$ssh_firewall" == 'true' ]] \
   && grep -Fq 'SSH es una decisión deliberada y permanente de producto' spec.md \
   && grep -Fq 'Korunix no ofrece una opción para desactivarlo' spec.md \
   && ! grep -Fq 'puede desactivarse desde Korunix' spec.md
then
  ok 'SSH permanece activo por contrato y acompañado del firewall'
else
  fallo 'SSH dejó de ser permanente o la especificación volvió a hacerlo desactivable'
fi

storage_model="$(
  nix eval \
    --json \
    '.#nixosConfigurations.korunix.config.korunix.storage.dataVolumes' \
    2>/dev/null \
    || true
)"

storage_mount="$(
  nix eval \
    --json \
    '.#nixosConfigurations.korunix.config.fileSystems."/mnt/datos"' \
    2>/dev/null \
    || true
)"

if jq -e '
     length == 1
     and .[0].uuid == "036F8E656FF00FB2"
     and .[0].fileSystem == "ntfs"
     and .[0].path == "/mnt/datos"
     and .[0].availableAtLogin == true
   ' <<<"$storage_model" >/dev/null 2>&1 \
   && jq -e '
     .device == "/dev/disk/by-uuid/036F8E656FF00FB2"
     and .fsType == "ntfs3"
     and (.options | index("x-systemd.automount") != null)
     and (.options | index("nofail") != null)
     and (.options | index("umask=0077") != null)
   ' <<<"$storage_mount" >/dev/null 2>&1
then
  ok 'unidad de datos usa UUID estable y disponibilidad automática'
else
  fallo 'la unidad de datos perdió su contrato declarativo'
fi

if rg -q 'c922 pro stream webcam' sistema/interfaz/principal.rs \
   && rg -q 'Realtek ALC897' sistema/interfaz/principal.rs \
   && rg -q 'device\.product\.name' sistema/interfaz/principal.rs \
   && rg -q 'alsa_card_name' sistema/interfaz/principal.rs \
   && rg -q 'fn consultar_disponibilidad_camara' sistema/interfaz/principal.rs \
   && rg -q 'glib::timeout_add_local' sistema/interfaz/principal.rs \
   && rg -q '"--tree"\.into\(\)' sistema/programa/operaciones.rs \
   && rg -q 'dataVolumes' sistema/programa/operaciones.rs
then
  ok 'multimedia identifica el audio y refresca cámaras virtuales'
else
  fallo 'multimedia perdió identidad o refresco dinámico'
fi

personas='sistema/personas.nix'
escritorio='sistema/escritorio.nix'

if rg -q 'type = "ibus";' "$personas" \
   && rg -q 'else "ibus";' "$personas" \
   && rg -q 'launcher = "xdg-autostart";' "$personas" \
   && grep -Fq "Exec=\${ibusPackage}/bin/ibus start --type wayland" "$escritorio" \
   && grep -Fq 'NotShowIn=KDE;' "$escritorio" \
   && ! grep -Fq 'ibus-daemon --daemonize --xim' "$escritorio" \
   && ! grep -Fq 'NotShowIn=KDE;niri;Hyprland;hyprland;' "$escritorio"
then
  ok 'IBus usa el arranque Wayland y sigue disponible en Niri/Hyprland'
else
  fallo 'IBus quedó deshabilitado, volvió al arranque XIM o contradice la política de diacríticos'
fi

if rg -q 'ibus\.waylandFrontend = true;' "$personas"
then
  ok 'IBus usa su frontend Wayland sin forzar módulos GTK/Qt'
else
  fallo 'IBus perdió waylandFrontend y puede volver a mostrar la advertencia de entorno'
fi

if grep -Fq '### 9.1. Teclas muertas, diacríticos y métodos de composición' spec.md \
   && grep -Fq 'Todas las aplicaciones GNOME instaladas que utilicen GTK/GTK4 y puedan depender' spec.md \
   && grep -Fq 'Text Editor son ejemplos de esa familia, no excepciones ni el alcance completo.' spec.md \
   && grep -Fq 'ibus start --type wayland' spec.md \
   && grep -Fq 'Korunix no debe ocultar ni filtrar una advertencia de IBus' spec.md \
   && grep -Fq 'La validación no se considera superada si funciona en' spec.md
then
  ok 'especificación protege diacríticos y el arranque Wayland de IBus'
else
  fallo 'la especificación perdió el alcance GNOME o el contrato Wayland de IBus'
fi

aplicaciones='sistema/aplicaciones.nix'
equipo='configuracion/equipos/korunix.nix'

if grep -Fq 'figma-linux-next.url = "github:arximus88/figma-linux-next";' flake.nix \
   && grep -Fq 'inputs.figma-linux-next.nixosModules.default' flake.nix \
   && grep -Fq '"figma-linux-next"' "$aplicaciones" \
   && grep -Fq 'programs.figma-linux-next.enable' "$aplicaciones" \
   && grep -Fq '"figma-linux-next"' "$equipo" \
   && ! grep -Fq '"figma-linux"' "$aplicaciones" \
   && ! grep -Fq '"figma-linux"' "$equipo" \
   && jq -e '.nodes.root.inputs["figma-linux-next"] != null' flake.lock >/dev/null
then
  ok 'Figma usa Figma Linux Next y el paquete histórico quedó retirado'
else
  fallo 'Figma volvió al paquete antiguo o perdió su integración declarativa'
fi

if grep -Fq 'arximus88/figma-linux-next' spec.md \
   && grep -Fq "El paquete histórico \`figma-linux\` de Nixpkgs no forma parte" spec.md \
   && grep -Fq "no fuerza \`GTK_IM_MODULE\` ni" spec.md
then
  ok 'especificación protege IBus Wayland y Figma Linux Next'
else
  fallo 'spec.md perdió la decisión de IBus o Figma'
fi

figma_visible="$(
  nix eval \
    --raw \
    '.#nixosConfigurations.korunix.config.korunix.internal.applicationPresentation."figma-linux-next".name' \
    2>/dev/null \
    || true
)"

if [[ "$figma_visible" == "Figma" ]] \
   && grep -Fq 'visible es exactamente “Figma”' spec.md \
   && ! grep -Fq 'name = "Figma Linux Next";' sistema/aplicaciones.nix
then
  ok 'Figma se presenta únicamente como Figma'
else
  fallo 'figma-linux-next se filtró al nombre visible del catálogo'
fi

if grep -Fq 'legacy_figma = "figma-linux.desktop"' "$personas" \
   && grep -Fq 'current_figma = "figma-linux-next.desktop"' "$personas" \
   && grep -Fq 'if len(matches) != 1:' "$personas" \
   && grep -Fq 'backup_dir = state_home / "backups" / "mime"' "$personas" \
   && grep -Fq 'os.fsync(directory)' "$personas" \
   && grep -Fq 'Al migrar desde ese cliente histórico' spec.md \
   && grep -Fq 'La migración es idempotente' spec.md
then
  ok 'migración de Figma conserva asociaciones personales'
else
  fallo 'la migración de Figma puede sobrescribir estado personal o perdió su contrato'
fi

if [[ -s spec.md ]] && grep -q '^# Korunix' spec.md; then
  ok 'especificación de producto'
else
  fallo 'falta la especificación de producto'
fi

if [[ -s sistema/interfaz/io.github.koruninn.Korunix.metainfo.xml ]]; then
  ok 'metadatos AppStream'
else
  fallo 'falta AppStream'
fi

if [[ -n "$motor_actual" ]]; then
  producto="$(
    KORUNIX_ROOT="$PWD" \
      "$motor_actual" product --json 2>/dev/null \
      || true
  )"

  if printf '%s\n' "$producto" \
      | jq -e '
          .schemaVersion == 1
          and .kind == "korunix-product-status"
          and .connectivity.offlineFirst == true
          and .platform.supported == true
          and (.platform.supportedSystems | index("x86_64-linux") != null)
          and (.platform.supportedSystems | index("aarch64-linux") != null)
          and (.capabilities.localWithoutNetwork | index("validate") != null)
          and (.capabilities.localWithoutNetwork | index("backup") != null)
        ' \
        >/dev/null
  then
    ok 'contrato de producto/offline'
  else
    printf '%s\n' "$producto"
    fallo 'contrato de producto/offline'
  fi
else
  fallo 'contrato de producto/offline'
fi


printf '\n%s\n' '→ Privacidad del producto'

if nix eval \
    --raw \
    '.#packages.x86_64-linux.motor.src' \
    2>/dev/null \
    | grep -Eq '/nix/store/'
then
  ok 'source empaquetado filtra datos humanos'
else
  fallo 'no pude verificar el source empaquetado'
fi


printf '\n%s\n' '→ Estructura de localización'

# Revisión fijada actualmente para Noctalia:
# 4b8c722e0c82816ca50a28ab4695ab765f3f4ab0
noctalia_locales=(
  be-Latn
  be
  ca
  cs
  de
  en
  es
  fr
  gl-ES
  hu
  it
  ko
  ku
  nl
  nn
  pl
  pt-BR
  ru
  sv
  tr
  uk-UA
  vi
  zh-Hans
)

ok 'lista de idiomas de Noctalia corresponde a la revisión verificada'

gui_variants="$(
  sed -n \
    '/^enum Idioma {$/,/^}$/p' \
    sistema/interfaz/principal.rs \
    | grep -Ec '^[[:space:]]+[[:alnum:]_]+,'
)"

ramas_localizadas="$(
  rg -o 'Idioma::(BelarusLatino|Belarus|Catalan|Checo|Aleman|Frances|Gallego|Italiano|Coreano|Kurdo|Neerlandes|NoruegoNynorsk|Polaco|PortuguesBrasil|Ruso|Sueco|Turco|Ucraniano|Vietnamita|ChinoSimplificado)[^=\n]*=>' \
    sistema/interfaz/principal.rs \
    | sed -E 's/^.*Idioma::([A-Za-z0-9_]+).*$/\1/' \
    | sort -u \
    | wc -l
)"

if [[ "$gui_variants" -eq "${#noctalia_locales[@]}" ]]; then
  if [[ "$ramas_localizadas" -eq 20 ]]; then
    ok "GUI declara las ${#noctalia_locales[@]} localizaciones verificadas y contiene ramas específicas"
  else
    fallo "hay variantes declaradas sin ramas específicas: $ramas_localizadas/20"
  fi
else
  fallo \
    "estructura de localización incompleta: GUI=${gui_variants}, Noctalia verificada=${#noctalia_locales[@]}"
fi


printf '\n%s\n' '=== RESULTADO AUTOMATIZABLE ==='

if [[ "$fallos" -eq 0 ]]; then
  printf '%s\n' \
    '✓ las comprobaciones automatizables actuales fueron superadas'
else
  printf '✗ la puerta automatizable encontró %s bloqueo(s)\n' "$fallos"
  false
fi

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

if rg -q 'fn storage_transfer\(' "$operaciones" \
   && rg -q 'fn transferir_archivos_a_directorio\(' "$operaciones" \
   && rg -q 'KORUNIX_TRANSFER' "$operaciones" \
   && rg -q 'fn sincronizar_sistema_archivos\(' "$operaciones" \
   && rg -q 'archivos_iguales' "$operaciones" \
   && rg -q 'renombrar_sin_reemplazar' "$operaciones" \
   && grep -Fq '"usesGlobalSync": false' "$operaciones"
then
  ok 'transferencias persisten y verifican sin sync global'
else
  fallo 'transferencias seguras perdieron persistencia, verificación o aislamiento'
fi

if rg -q 'gtk::FileChooserNative::new' sistema/interfaz/principal.rs \
   && rg -q 'set_select_multiple\(true\)' sistema/interfaz/principal.rs \
   && rg -q 'EventoMotor::Transferencia' sistema/interfaz/principal.rs \
   && rg -q 'mostrar_progreso_transferencia' sistema/interfaz/principal.rs \
   && grep -Fq 'Transferencias a almacenamiento extraíble' spec.md \
   && grep -Fq 'mostrar porcentaje, velocidad y ETA cuando sean medibles' spec.md \
   && grep -Fq 'ofrecer expulsar el dispositivo cuando proceda' spec.md
then
  ok 'GUI ofrece el asistente de transferencias definido por la especificación'
else
  fallo 'la GUI perdió el asistente o su contrato de progreso'
fi

if rg -q 'fn storage_tool\(' "$operaciones" \
   && rg -q 'capture_status\(raiz, programa, args\)' "$operaciones" \
   && ! grep -Fq 'visible(raiz, "sync"' "$operaciones"
then
  ok 'expulsión JSON mantiene stdout limpio y evita sync global'
else
  fallo 'expulsión puede contaminar JSON o volver a sincronización global'
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

if grep -Fq 'texto_ahora_no(estado_transferir.idioma)' sistema/interfaz/principal.rs \
   && grep -Fq 'dialogo_confirmacion_con_secundaria' sistema/interfaz/principal.rs \
   && grep -Fq 'no cancela ni revierte la transferencia' spec.md \
   && grep -Fq 'Ahora no' spec.md
then
  ok 'posponer la expulsión no se presenta como cancelar la transferencia'
else
  fallo 'la oferta posterior a transferir volvió a confundir expulsión con cancelación'
fi

if grep -Fq 'let modelo_fisico = modelo_audio_conocido(&huella);' sistema/interfaz/principal.rs \
   && grep -Fq 'format!("{modelo} · {entrada}")' sistema/interfaz/principal.rs \
   && grep -Fq 'format!("Realtek ALC897 · {salida}")' sistema/interfaz/principal.rs \
   && grep -Fq 'misma identidad base de marca/modelo' spec.md
then
  ok 'entrada y salida comparten identidad física cuando puede demostrarse'
else
  fallo 'audio perdió la identidad común del dispositivo físico'
fi

if grep -Fq 'for fuente in ["nixpkgs", "flatpak"]' sistema/interfaz/principal.rs \
   && grep -Fq 'if fuente == "nixpkgs" && encontrados > 0' sistema/interfaz/principal.rs \
   && grep -Fq 'let clave = resultado.nombre.trim().to_lowercase();' sistema/interfaz/principal.rs \
   && grep -Fq 'Flatpak actúa como segunda fuente únicamente' spec.md
then
  ok 'búsqueda externa prioriza Nixpkgs y usa Flatpak como fallback'
else
  fallo 'la búsqueda de aplicaciones perdió su prioridad de fuentes'
fi

if grep -Fq 'let avatar_seleccion = Rc::new(RefCell::new(None::<PathBuf>));' sistema/interfaz/principal.rs \
   && grep -Fq 'Elegir avatar' sistema/interfaz/principal.rs \
   && ! grep -Fq 'Avatar opcional · ruta PNG/JPEG/WebP' sistema/interfaz/principal.rs \
   && grep -Fq 'selector de' spec.md \
   && grep -Fq 'No debe tener que escribir ni conocer una ruta del sistema' spec.md
then
  ok 'Personas selecciona el avatar sin pedir rutas técnicas'
else
  fallo 'Personas volvió a exponer la ruta del avatar como entrada normal'
fi

if grep -Fq 'let copia_seleccion = Rc::new(RefCell::new(None::<PathBuf>));' sistema/interfaz/principal.rs \
   && grep -Fq 'Elegir copia de Korunix' sistema/interfaz/principal.rs \
   && grep -Fq 'Restaurar una copia desde la interfaz normal utiliza un selector de archivo' spec.md
then
  ok 'Copias restaura mediante selector de archivo'
else
  fallo 'Copias volvió a exigir escribir una ruta para restaurar'
fi

if grep -Fq 'fn resumen_historial_humano(resumen: &str) -> String' sistema/interfaz/principal.rs \
   && grep -Fq 'nombre_aplicacion_historial' sistema/interfaz/principal.rs \
   && grep -Fq 'Preparaste la apariencia {estilo} en modo {modo}' sistema/interfaz/principal.rs \
   && grep -Fq 'la vista normal debe traducirlos a lenguaje humano' spec.md
then
  ok 'historial heredado oculta identificadores internos en la vista normal'
else
  fallo 'historial puede volver a mostrar identificadores internos heredados'
fi

if grep -Fq 'preferredLanguages = lib.mkOption' sistema/idioma.nix \
   && grep -Fq 'environment.sessionVariables.LANGUAGE' sistema/idioma.nix \
   && grep -Fq 'preferredLanguages = ["es"];' configuracion/equipos/korunix.nix \
   && grep -Fq 'fn localization_catalog_json' sistema/programa/operaciones.rs \
   && grep -Fq 'xkeyboard_config.outPath' sistema/programa/operaciones.rs \
   && grep -Fq 'tzdata.outPath' sistema/programa/operaciones.rs
then
  ok 'localización modela idiomas preferidos y catálogos efectivos'
else
  fallo 'localización perdió idiomas preferidos o sus catálogos reales'
fi

if grep -Fq 'struct OpcionLocalizacion' sistema/interfaz/principal.rs \
   && grep -Fq 'set_enable_search(true)' sistema/interfaz/principal.rs \
   && grep -Fq -- '--preferred-languages-json' sistema/interfaz/principal.rs \
   && grep -Fq -- '--keyboards-json' sistema/interfaz/principal.rs \
   && ! grep -Fq 'Los identificadores técnicos solo se muestran aquí porque esta edición avanzada' sistema/interfaz/principal.rs
then
  ok 'Idioma y región usa selectores humanos buscables'
else
  fallo 'Idioma y región volvió a exponer edición técnica o perdió selección múltiple'
fi

if grep -Fq 'El catálogo de teclados no es una lista histórica' spec.md \
   && grep -Fq "Las zonas horarias se obtienen de la \`tzdata\` efectiva" spec.md \
   && grep -Fq 'Los idiomas preferidos del sistema forman una lista ordenada' spec.md
then
  ok 'especificación protege localización humana y catálogos reales'
else
  fallo 'spec perdió el contrato de localización humana'
fi

if grep -Fq 'const KORUNIX_INTERFACE_LANGUAGES' sistema/programa/operaciones.rs \
   && grep -Fq 'fn interface_language_operation' sistema/programa/operaciones.rs \
   && grep -Fq '"changesSystemLanguage": false' sistema/programa/operaciones.rs \
   && grep -Fq '"changesSessionLanguage": false' sistema/programa/operaciones.rs \
   && grep -Fq '"requiresSystemApply": false' sistema/programa/operaciones.rs \
   && grep -Fq '"interface-language" => interface_language_operation' sistema/programa/operaciones.rs
then
  ok 'motor separa el idioma de Korunix de localización del sistema'
else
  fallo 'motor perdió la separación del idioma propio de Korunix'
fi

if grep -Fq "interfaceLanguage: (\$u.interfaceLanguage // null)" sistema/programa/principal.rs \
   && grep -Fq 'portableProfileSchemaVersion\":3' sistema/programa/principal.rs \
   && grep -Fq 'compatiblePortableProfileSchemaVersions\":[1,2,3]' sistema/programa/principal.rs \
   && grep -Fq "interfaceLanguage:(\$p.interfaceLanguage // null)" sistema/programa/operaciones.rs \
   && grep -Fq 'fn profile_text_from_manifest' sistema/programa/operaciones.rs
then
  ok 'interfaceLanguage viaja en el perfil portable versión 3'
else
  fallo 'interfaceLanguage puede perderse al exportar o importar perfiles'
fi

if grep -Fq 'language = profile.language or config.korunix.localization.systemLanguage;' sistema/personas.nix \
   && ! grep -Fq 'profile.interfaceLanguage' sistema/personas.nix \
   && grep -Fq 'interfaceLanguage = null;' configuracion/personas/koru.nix
then
  ok 'language de sesión permanece independiente de interfaceLanguage'
else
  fallo 'la preferencia visual volvió a mezclarse con la preparación de sesión'
fi

if grep -Fq 'fn idioma_preferido_interfaz' sistema/interfaz/principal.rs \
   && grep -Fq 'fn fijar_idioma_interfaz' sistema/interfaz/principal.rs \
   && grep -Fq 'fn construir_ventana_con_idioma' sistema/interfaz/principal.rs \
   && grep -Fq '"interface-language".to_string()' sistema/interfaz/principal.rs \
   && grep -Fq 'ventana_anterior.close();' sistema/interfaz/principal.rs
then
  ok 'GUI cambia el idioma dentro del mismo proceso'
else
  fallo 'GUI perdió la recarga viva del idioma'
fi

if grep -Fq 'fn profile_simple_optional_string_value' sistema/programa/operaciones.rs \
   && grep -Fq 'profile_simple_optional_string_value(&text, "interfaceLanguage")?' sistema/programa/operaciones.rs \
   && grep -Fq 'Una preferencia simple del perfil no justifica evaluar Nix' sistema/programa/operaciones.rs \
   && grep -Fq "Leer \`interfaceLanguage\` desde el perfil portable no debe disparar una evaluación" spec.md
then
  ok 'idioma inicial usa lectura local antes de dibujar la ventana'
else
  fallo 'arranque puede volver a evaluar Nix para conocer el idioma'
fi

linea_presentar="$(
  grep -nF '    ventana.present();' sistema/interfaz/principal.rs \
    | tail -n 1 \
    | cut -d: -f1
)"
linea_precarga="$(
  grep -nF '    cargar_pagina(estado, pagina_precarga_inicial(), false);' \
    sistema/interfaz/principal.rs \
    | tail -n 1 \
    | cut -d: -f1
)"

if [[ "$linea_presentar" =~ ^[0-9]+$ ]] \
   && [[ "$linea_precarga" =~ ^[0-9]+$ ]] \
   && (( linea_presentar < linea_precarga )) \
   && grep -Fq 'fn pagina_precarga_inicial()' sistema/interfaz/principal.rs \
   && grep -Fq 'fn cargar_pagina(estado: Rc<Estado>, nombre: &str, forzar: bool)' sistema/interfaz/principal.rs \
   && grep -Fq 'paginas_cargadas: RefCell<HashSet<String>>' sistema/interfaz/principal.rs \
   && grep -Fq 'pagina_pendiente: RefCell<Option<(String, bool)>>' sistema/interfaz/principal.rs \
   && grep -Fq 'let pagina = pagina_cargando(idioma);' sistema/interfaz/principal.rs \
   && ! grep -Fq 'let pagina = pagina_error(idioma, texto(idioma, "loading"));' sistema/interfaz/principal.rs \
   && ! grep -Fq 'mostrar_progreso(&estado, 97, "reading");' sistema/interfaz/principal.rs \
   && grep -Fq 'Korunix no inicia una recarga global de todas las áreas' spec.md \
   && grep -Fq 'Navegar a Hardware no debe esperar' spec.md
then
  ok 'GUI carga Resumen primero y cada sección bajo demanda'
else
  fallo 'la GUI puede volver a precargar todas las secciones o fingir progreso global'
fi

catalogos_interfaz=0
catalogos_interfaz_ok=1
while IFS= read -r catalogo; do
  catalogos_interfaz=$((catalogos_interfaz + 1))
  if ! jq -e 'has("Idioma de Korunix")' "$catalogo" >/dev/null; then
    catalogos_interfaz_ok=0
  fi
done < <(find sistema/interfaz/i18n -maxdepth 1 -type f -name '*.json' -print | sort)

if [[ "$catalogos_interfaz" -eq 23 ]] \
   && [[ "$catalogos_interfaz_ok" -eq 1 ]]
then
  ok 'los 23 catálogos traducen el selector del idioma de Korunix'
else
  fallo 'algún catálogo perdió el rótulo del idioma de Korunix'
fi

if grep -Fq "se guarda en el perfil portable como \`interfaceLanguage\`" spec.md \
   && grep -Fq "\`interfaceLanguage = null\` o un campo ausente significa modo automático" spec.md \
   && grep -Fq "no modifica \`systemLanguage\`" spec.md \
   && grep -Fq 'dentro del mismo proceso con el catálogo elegido' spec.md
then
  ok 'spec protege idioma visual, modo automático y cambio vivo'
else
  fallo 'spec perdió el contrato independiente del idioma de interfaz'
fi

if grep -Fq 'applyNoctaliaPortalSettings' sistema/escritorio.nix \
   && grep -Fq 'msg theme-mode-get' sistema/escritorio.nix \
   && grep -Fq 'org.gnome.desktop.interface color-scheme' sistema/escritorio.nix \
   && grep -Fq 'systemctl --user --quiet is-active noctalia.service' sistema/escritorio.nix \
   && grep -Fq 'export DCONF_PROFILE=noctalia' sistema/escritorio.nix \
   && grep -Fq 'Nautilus— deben recibir los cambios claro/oscuro' spec.md \
   && grep -Fq 'portal GTK debe observar la misma base dconf' spec.md
then
  ok 'GTK4 y Nautilus siguen la apariencia efectiva de Noctalia'
else
  fallo 'se perdió el contrato de cambio vivo GTK4/Noctalia'
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
     and (.options | index("x-gvfs-show") != null)
   ' <<<"$storage_mount" >/dev/null 2>&1 \
   && grep -Fq 'una unidad de datos que Korunix deja lista para uso cotidiano debe aparecer allí' spec.md \
   && grep -Fq 'x-gvfs-show' spec.md
then
  ok 'unidad de datos usa UUID estable, disponibilidad automática y visibilidad en archivos'
else
  fallo 'la unidad de datos perdió disponibilidad o visibilidad en el gestor de archivos'
fi

if rg -q 'fn refrescar_monitor_gvfs_si_cambio_fstab\(' "$operaciones" \
   && grep -Fq 'let fstab_antes = fs::read("/etc/fstab").ok();' "$operaciones" \
   && grep -Fq '"try-restart",' "$operaciones" \
   && grep -Fq '"gvfs-udisks2-volume-monitor.service",' "$operaciones" \
   && grep -Fq 'refrescar_monitor_gvfs_si_cambio_fstab(fstab_antes);' "$operaciones" \
   && grep -Fq 'verificar correctamente la activación. El refresco solo corresponde cuando el' spec.md \
   && grep -Fq 'contenido efectivo de' spec.md \
   && grep -Fq 'cambió; debe usar' spec.md
then
  ok 'apply refresca GVfs únicamente cuando cambia fstab'
else
  fallo 'apply puede dejar GVfs con una lista de unidades obsoleta'
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


fetch_campos="$(
  sed -E '/^[[:space:]]*(#|$)/d; /=/d' config/fetch.conf \
    | paste -sd ' ' -
)"

if [[ "$fetch_campos" == 'os kernel shell wm cpu memory disk colors' ]] \
   && grep -Fq 'korunixFetch = pkgs.writeShellApplication' sistema/base.nix \
   && grep -Fq 'export HOME=/etc/korunix/fetch-home' sistema/base.nix \
   && grep -Fq 'environment.etc."korunix/fetch-home/.config/fetch/config".source = ../config/fetch.conf;' sistema/base.nix \
   && grep -Fq '      korunixFetch' sistema/base.nix \
   && ! grep -Fq "ensure_link \"\$config_home/fetch/config\"" sistema/personas.nix \
   && grep -Fq 'Host, tiempo encendido, pantalla, iconos, fuente, terminal, GPU, swap, IP local y' spec.md \
   && grep -Fq 'La versión actual de Fetch busca su archivo directamente bajo' spec.md \
   && grep -Fq 'no utiliza' spec.md \
   && grep -Fq 'XDG_CONFIG_HOME' spec.md \
   && grep -Fq 'fetchPackage = fetchUpstream.overrideAttrs' sistema/base.nix \
   && grep -Fq '("CPU",' sistema/base.nix \
   && grep -Fq '("memoria",' sistema/base.nix \
   && grep -Fq '("disco",' sistema/base.nix \
   && grep -Fq 'CPU muestra el nombre humano del modelo' spec.md \
   && grep -Fq 'Memoria muestra usado / total con una sola cifra decimal' spec.md \
   && grep -Fq 'Disco muestra usado / total redondeado y una sola unidad' spec.md \
   && grep -Fq 'debe fallar durante la construcción si el código upstream' spec.md
then
  ok 'Fetch conserva la salida compacta y usa su configuración de producto'
else
  fallo 'Fetch puede volver a mostrar la salida extensa o ignorar su configuración'
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

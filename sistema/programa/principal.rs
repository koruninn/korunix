//! Motor operativo de Korunix.
//!
//! Este archivo es parte interna del sistema.
//!
//! Una persona no necesita editarlo para cambiar su computadora. Las decisiones
//! humanas viven en los archivos de configuración. Este programa toma esas
//! decisiones y realiza las consultas u operaciones necesarias.
//!
//! Rust es el único motor operativo público de Korunix.
//! Los archivos que permanecen en scripts/ son accesos de compatibilidad y no
//! contienen dominio operativo.

mod operaciones;

use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::{self, IsTerminal};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

const AYUDA: &str = r#"Korunix

Uso:
  korunix modelo [canales|predeterminados|equipos [id]]
  korunix bootstrap --plan [--json]
  korunix bootstrap --adopt [--yes]
  korunix product [--json]
  korunix host [--json | rename <nombre> [--plan] [--yes] [--json]]
  korunix users [operación]
  korunix applications [operación]
  korunix desktop [operación]
  korunix appearance [operación]
  korunix defaults [--json | set --person <id> [--browser firefox|google-chrome] [--plasma-text-editor kwrite|kate] [--plan] [--yes] [--json]]
  korunix hardware [--json]
  korunix localization [operación]
  korunix interface-language [--json | set <idioma|auto> [--plan] [--yes] [--json]]
  korunix backup [operación]
  korunix history [--json]
  korunix channel [opciones]
  korunix status
  korunix structure
  korunix validate
  korunix format
  korunix preview [--json]
  korunix build [--json]
  korunix apply [--yes] [--json]
  korunix update [entradas...] [--plan] [--json]
  korunix rollback --list [--json]
  korunix rollback <id> [--plan] [--yes] [--json]
  korunix clean-preview [--json]
  korunix clean [--yes] [--json]
  korunix clean-all-preview [--json]
  korunix clean-all [--yes] [--json]
  korunix storage ...
  korunix firmware ...
  korunix media ...
  korunix privileges [--json]

Nix contiene las decisiones declarativas. Rust realiza las operaciones del
sistema vivo. Korunix no automatiza contraseñas ni fabrica pseudo-TTY.
"#;
fn es_raiz_korunix(ruta: &Path) -> bool {
    ruta.join("flake.nix").is_file()
        && ruta.join("sistema/canales.nix").is_file()
        && ruta.join("sistema/predeterminados.nix").is_file()
        && ruta.join("sistema/programa/principal.rs").is_file()
}

fn raiz_repositorio() -> Result<PathBuf, String> {
    if let Some(valor) = env::var_os("KORUNIX_ROOT") {
        let ruta = PathBuf::from(valor);

        if es_raiz_korunix(&ruta) {
            return Ok(ruta);
        }

        return Err(format!(
            "KORUNIX_ROOT no apunta a una carpeta válida de Korunix: {}",
            ruta.display()
        ));
    }

    let actual =
        env::current_dir().map_err(|error| format!("No pude leer la carpeta actual: {error}"))?;

    for candidato in actual.ancestors() {
        if es_raiz_korunix(candidato) {
            return Ok(candidato.to_path_buf());
        }
    }

    if let Some(home) = env::var_os("HOME") {
        let candidato = PathBuf::from(home).join(".korunix");

        if es_raiz_korunix(&candidato) {
            return Ok(candidato);
        }
    }

    Err("No pude encontrar la carpeta de Korunix. Define KORUNIX_ROOT.".to_string())
}

fn id_valido(valor: &str) -> bool {
    !valor.is_empty()
        && valor
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

fn json_texto(valor: &str) -> String {
    let mut salida = String::with_capacity(valor.len() + 2);
    salida.push('"');

    for caracter in valor.chars() {
        match caracter {
            '"' => salida.push_str("\\\""),
            '\\' => salida.push_str("\\\\"),
            '\n' => salida.push_str("\\n"),
            '\r' => salida.push_str("\\r"),
            '\t' => salida.push_str("\\t"),
            c if c.is_control() => salida.push_str(&format!("\\u{:04x}", c as u32)),
            c => salida.push(c),
        }
    }

    salida.push('"');
    salida
}

fn ejecutar_capturando(
    programa: &str,
    argumentos: &[String],
    raiz: &Path,
) -> Result<String, String> {
    let salida = Command::new(programa)
        .args(argumentos)
        .current_dir(raiz)
        .output()
        .map_err(|error| format!("No pude ejecutar {programa}: {error}"))?;

    if !salida.status.success() {
        let error = String::from_utf8_lossy(&salida.stderr).trim().to_string();
        return Err(if error.is_empty() {
            format!("{programa} terminó con un error.")
        } else {
            error
        });
    }

    Ok(String::from_utf8_lossy(&salida.stdout).trim().to_string())
}

fn nix_archivo_json(raiz: &Path, archivo: &Path) -> Result<String, String> {
    ejecutar_capturando(
        "nix",
        &[
            "eval".to_string(),
            "--json".to_string(),
            "--file".to_string(),
            archivo.display().to_string(),
        ],
        raiz,
    )
}

fn valor_canal(raiz: &Path, canal: &str, clave: &str) -> Result<String, String> {
    if !matches!(canal, "stable" | "unstable") {
        return Err(format!("Canal desconocido: {canal}"));
    }

    let claves = [
        "label",
        "description",
        "nixpkgs_ref",
        "aagl_ref",
        "label_en",
        "description_en",
        "label_hu",
        "description_hu",
    ];

    if !claves.contains(&clave) {
        return Err(format!("Dato desconocido del canal: {clave}"));
    }

    // El archivo se entrega a Nix mediante `--file`. Eso permite leer una
    // fuente local concreta sin incrustar su ruta dentro de una expresión pura.
    //
    // `--apply` recibe el resultado del archivo y escoge solamente el dato que
    // necesitamos. Es parecido a decir: "abre esta caja y dame esta pieza".
    // `canal` y `clave` ya fueron limitados a valores conocidos unas líneas
    // arriba, así que no pueden convertirse en código Nix arbitrario.
    let seleccion = format!("datos: datos.channels.{}.{}", canal, clave);

    ejecutar_capturando(
        "nix",
        &[
            "eval".to_string(),
            "--raw".to_string(),
            "--file".to_string(),
            "sistema/canales.nix".to_string(),
            "--apply".to_string(),
            seleccion,
        ],
        raiz,
    )
}

fn equipos_disponibles(raiz: &Path) -> Result<Vec<String>, String> {
    let carpeta = raiz.join("configuracion").join("equipos");
    let mut ids = Vec::new();

    for entrada in fs::read_dir(&carpeta)
        .map_err(|error| format!("No pude leer {}: {error}", carpeta.display()))?
    {
        let entrada = entrada.map_err(|error| format!("No pude leer un equipo: {error}"))?;
        let ruta = entrada.path();

        if !ruta.is_file() {
            continue;
        }

        let Some(nombre) = ruta.file_name().and_then(|valor| valor.to_str()) else {
            continue;
        };

        if !nombre.ends_with(".nix") || nombre.ends_with("-detectado.nix") {
            continue;
        }

        let id = nombre.trim_end_matches(".nix");
        if id_valido(id) {
            ids.push(id.to_string());
        }
    }

    ids.sort();
    Ok(ids)
}

fn host_id_marker_value(texto: &str) -> Result<String, String> {
    let valor = texto.trim();

    if !id_valido(valor) {
        return Err(
            "El identificador estructural persistido por Korunix no es válido.".to_string(),
        );
    }

    Ok(valor.to_string())
}

fn host_id_persistido() -> Result<Option<String>, String> {
    let ruta = env::var_os("KORUNIX_HOST_ID_FILE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/etc/korunix/host-id"));

    if !ruta.exists() {
        return Ok(None);
    }

    if !ruta.is_file() {
        return Err(format!(
            "El marcador de identidad {} no es un archivo.",
            ruta.display()
        ));
    }

    let texto = fs::read_to_string(&ruta)
        .map_err(|error| format!("No pude leer {}: {error}", ruta.display()))?;

    Ok(Some(host_id_marker_value(&texto)?))
}

fn resolver_equipo(raiz: &Path) -> Result<String, String> {
    if let Ok(valor) = env::var("KORUNIX_HOST") {
        if !id_valido(&valor) {
            return Err("KORUNIX_HOST contiene un identificador inválido.".to_string());
        }

        if raiz
            .join("configuracion")
            .join("equipos")
            .join(format!("{valor}.nix"))
            .is_file()
        {
            return Ok(valor);
        }

        return Err(format!("No existe configuracion/equipos/{valor}.nix."));
    }

    if let Some(valor) = host_id_persistido()? {
        if raiz
            .join("configuracion")
            .join("equipos")
            .join(format!("{valor}.nix"))
            .is_file()
        {
            return Ok(valor);
        }

        return Err(format!(
            "Este sistema se identifica como {valor}, pero falta configuracion/equipos/{valor}.nix."
        ));
    }

    let nombre = ejecutar_capturando("hostname", &[], raiz).unwrap_or_default();
    if id_valido(&nombre)
        && raiz
            .join("configuracion")
            .join("equipos")
            .join(format!("{nombre}.nix"))
            .is_file()
    {
        return Ok(nombre);
    }

    let disponibles = equipos_disponibles(raiz)?;
    match disponibles.as_slice() {
        [unico] => Ok(unico.clone()),
        [] => Err("Korunix no encontró ninguna computadora configurada.".to_string()),
        _ => Err(
            "Hay varias computadoras configuradas y este sistema todavía no tiene un hostId persistido. Define KORUNIX_HOST para elegir una."
                .to_string(),
        ),
    }
}

fn canal_declarado(raiz: &Path, equipo: &str) -> Result<String, String> {
    let archivo = raiz
        .join("configuracion")
        .join("equipos")
        .join(format!("{equipo}.nix"));
    let texto = fs::read_to_string(&archivo)
        .map_err(|error| format!("No pude leer {}: {error}", archivo.display()))?;

    let mut encontrados = Vec::new();
    for linea in texto.lines() {
        match linea.trim() {
            "channel = \"stable\";" => encontrados.push("stable"),
            "channel = \"unstable\";" => encontrados.push("unstable"),
            _ => {}
        }
    }

    match encontrados.as_slice() {
        [canal] => Ok((*canal).to_string()),
        _ => Err(format!(
            "configuracion/equipos/{equipo}.nix debe declarar exactamente un canal."
        )),
    }
}

fn flake_raw(raiz: &Path, atributo: &str) -> Result<String, String> {
    let instalable = format!("path:{}#{atributo}", raiz.display());
    ejecutar_capturando(
        "nix",
        &["eval".to_string(), "--raw".to_string(), instalable],
        raiz,
    )
}

fn runtime_state_path() -> PathBuf {
    env::var_os("KORUNIX_RUNTIME_STATE_FILE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/etc/korunix/runtime-state.json"))
}

fn sha256_archivo(raiz: &Path, ruta: &Path) -> Option<String> {
    let salida = ejecutar_capturando("sha256sum", &[ruta.display().to_string()], raiz).ok()?;

    salida.split_whitespace().next().map(ToString::to_string)
}

fn runtime_hash_entry_matches(raiz: &Path, entry: &serde_json::Value) -> bool {
    let Some(relative) = entry.get("path").and_then(serde_json::Value::as_str) else {
        return false;
    };
    let Some(expected) = entry.get("sha256").and_then(serde_json::Value::as_str) else {
        return false;
    };

    let path = raiz.join(relative);
    path.is_file()
        && sha256_archivo(raiz, &path)
            .map(|actual| actual == expected)
            .unwrap_or(false)
}

fn runtime_state_source_matches(raiz: &Path, state: &serde_json::Value) -> bool {
    let Some(source) = state.get("sourceHashes") else {
        return false;
    };

    for key in ["host", "hardware", "channels"] {
        let Some(entry) = source.get(key) else {
            return false;
        };
        if !runtime_hash_entry_matches(raiz, entry) {
            return false;
        }
    }

    let Some(expected_personas) = source.get("personas").and_then(serde_json::Value::as_array)
    else {
        return false;
    };

    let mut expected_paths = expected_personas
        .iter()
        .filter_map(|entry| entry.get("path"))
        .filter_map(serde_json::Value::as_str)
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    expected_paths.sort();

    let personas_dir = raiz.join("configuracion/personas");
    let Ok(entries) = fs::read_dir(&personas_dir) else {
        return false;
    };

    let mut current_paths = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if !path.is_file() || path.extension().and_then(|value| value.to_str()) != Some("nix") {
                return None;
            }

            path.strip_prefix(raiz)
                .ok()
                .and_then(|relative| relative.to_str())
                .map(ToString::to_string)
        })
        .collect::<Vec<_>>();
    current_paths.sort();

    if current_paths != expected_paths {
        return false;
    }

    expected_personas
        .iter()
        .all(|entry| runtime_hash_entry_matches(raiz, entry))
}

fn runtime_state_current(raiz: &Path) -> Result<Option<serde_json::Value>, String> {
    let path = runtime_state_path();
    if !path.is_file() {
        return Ok(None);
    }

    let raw = match fs::read_to_string(&path) {
        Ok(value) => value,
        Err(_) => return Ok(None),
    };

    let state = match serde_json::from_str::<serde_json::Value>(&raw) {
        Ok(value) => value,
        Err(_) => return Ok(None),
    };

    if state
        .get("schemaVersion")
        .and_then(serde_json::Value::as_u64)
        != Some(1)
    {
        return Ok(None);
    }

    let host = resolver_equipo(raiz)?;
    if state.get("hostId").and_then(serde_json::Value::as_str) != Some(host.as_str()) {
        return Ok(None);
    }

    if !runtime_state_source_matches(raiz, &state) {
        return Ok(None);
    }

    Ok(Some(state))
}

fn canal_json_runtime(raiz: &Path, state: &serde_json::Value) -> Result<Option<String>, String> {
    let declarado = canal_declarado(raiz, &resolver_equipo(raiz)?)?;
    let Some(channel) = state.get("channel") else {
        return Ok(None);
    };
    let Some(model) = channel.get("model") else {
        return Ok(None);
    };
    let Some(selected) = model.get(&declarado) else {
        return Ok(None);
    };

    let option = |id: &str| -> Option<serde_json::Value> {
        let value = model.get(id)?;
        Some(serde_json::json!({
            "id": id,
            "labels": {
                "es": value.get("label")?,
                "en": value.get("label_en")?,
                "hu": value.get("label_hu")?
            },
            "descriptions": {
                "es": value.get("description")?,
                "en": value.get("description_en")?,
                "hu": value.get("description_hu")?
            },
            "sources": {
                "nixpkgs": value.get("nixpkgs_ref")?,
                "aagl": value.get("aagl_ref")?
            }
        }))
    };

    let Some(stable) = option("stable") else {
        return Ok(None);
    };
    let Some(unstable) = option("unstable") else {
        return Ok(None);
    };

    let output = serde_json::json!({
        "schemaVersion": 1,
        "hostId": state.get("hostId").cloned().unwrap_or(serde_json::Value::Null),
        "declared": declarado,
        "effective": channel.get("effective").cloned().unwrap_or(serde_json::Value::Null),
        "label": selected.get("label").cloned().unwrap_or(serde_json::Value::Null),
        "description": selected.get("description").cloned().unwrap_or(serde_json::Value::Null),
        "nixosVersion": channel.get("nixosVersion").cloned().unwrap_or(serde_json::Value::Null),
        "stateVersion": channel.get("stateVersion").cloned().unwrap_or(serde_json::Value::Null),
        "stateVersionIndependent": true,
        "sources": {
            "nixpkgs": selected.get("nixpkgs_ref").cloned().unwrap_or(serde_json::Value::Null),
            "aagl": selected.get("aagl_ref").cloned().unwrap_or(serde_json::Value::Null)
        },
        "options": [stable, unstable]
    });

    Ok(Some(output.to_string()))
}

fn canal_json(raiz: &Path) -> Result<String, String> {
    if let Some(runtime) = runtime_state_current(raiz)? {
        if let Some(output) = canal_json_runtime(raiz, &runtime)? {
            return Ok(output);
        }
    }

    let equipo = resolver_equipo(raiz)?;
    let declarado = canal_declarado(raiz, &equipo)?;

    let efectivo = flake_raw(
        raiz,
        &format!("nixosConfigurations.{equipo}.config.korunix.channel"),
    )?;
    let version = flake_raw(
        raiz,
        &format!("nixosConfigurations.{equipo}.config.system.nixos.version"),
    )?;
    let state_version = flake_raw(
        raiz,
        &format!("nixosConfigurations.{equipo}.config.system.stateVersion"),
    )?;

    let label = valor_canal(raiz, &declarado, "label")?;
    let description = valor_canal(raiz, &declarado, "description")?;
    let nixpkgs = valor_canal(raiz, &declarado, "nixpkgs_ref")?;
    let aagl = valor_canal(raiz, &declarado, "aagl_ref")?;

    let stable_label = valor_canal(raiz, "stable", "label")?;
    let stable_description = valor_canal(raiz, "stable", "description")?;
    let stable_label_en = valor_canal(raiz, "stable", "label_en")?;
    let stable_description_en = valor_canal(raiz, "stable", "description_en")?;
    let stable_label_hu = valor_canal(raiz, "stable", "label_hu")?;
    let stable_description_hu = valor_canal(raiz, "stable", "description_hu")?;
    let stable_nixpkgs = valor_canal(raiz, "stable", "nixpkgs_ref")?;
    let stable_aagl = valor_canal(raiz, "stable", "aagl_ref")?;

    let unstable_label = valor_canal(raiz, "unstable", "label")?;
    let unstable_description = valor_canal(raiz, "unstable", "description")?;
    let unstable_label_en = valor_canal(raiz, "unstable", "label_en")?;
    let unstable_description_en = valor_canal(raiz, "unstable", "description_en")?;
    let unstable_label_hu = valor_canal(raiz, "unstable", "label_hu")?;
    let unstable_description_hu = valor_canal(raiz, "unstable", "description_hu")?;
    let unstable_nixpkgs = valor_canal(raiz, "unstable", "nixpkgs_ref")?;
    let unstable_aagl = valor_canal(raiz, "unstable", "aagl_ref")?;

    Ok(format!(
        concat!(
            "{{",
            "\"schemaVersion\":1,",
            "\"hostId\":{},",
            "\"declared\":{},",
            "\"effective\":{},",
            "\"label\":{},",
            "\"description\":{},",
            "\"nixosVersion\":{},",
            "\"stateVersion\":{},",
            "\"stateVersionIndependent\":true,",
            "\"sources\":{{\"nixpkgs\":{},\"aagl\":{}}},",
            "\"options\":[",
            "{{\"id\":\"stable\",\"labels\":{{\"es\":{},\"en\":{},\"hu\":{}}},",
            "\"descriptions\":{{\"es\":{},\"en\":{},\"hu\":{}}},",
            "\"sources\":{{\"nixpkgs\":{},\"aagl\":{}}}}},",
            "{{\"id\":\"unstable\",\"labels\":{{\"es\":{},\"en\":{},\"hu\":{}}},",
            "\"descriptions\":{{\"es\":{},\"en\":{},\"hu\":{}}},",
            "\"sources\":{{\"nixpkgs\":{},\"aagl\":{}}}}}",
            "]}}"
        ),
        json_texto(&equipo),
        json_texto(&declarado),
        json_texto(&efectivo),
        json_texto(&label),
        json_texto(&description),
        json_texto(&version),
        json_texto(&state_version),
        json_texto(&nixpkgs),
        json_texto(&aagl),
        json_texto(&stable_label),
        json_texto(&stable_label_en),
        json_texto(&stable_label_hu),
        json_texto(&stable_description),
        json_texto(&stable_description_en),
        json_texto(&stable_description_hu),
        json_texto(&stable_nixpkgs),
        json_texto(&stable_aagl),
        json_texto(&unstable_label),
        json_texto(&unstable_label_en),
        json_texto(&unstable_label_hu),
        json_texto(&unstable_description),
        json_texto(&unstable_description_en),
        json_texto(&unstable_description_hu),
        json_texto(&unstable_nixpkgs),
        json_texto(&unstable_aagl),
    ))
}

// CONTROL PRINCIPAL: canal de actualizaciones.
//
// ¿Qué controla?
// Esta elección decide qué familia de Nixpkgs y AAGL usa una computadora.
//
// ¿Qué NO controla?
// No cambia system.stateVersion y no aplica una generación por sí sola.
//
// ¿Por qué hay varias funciones debajo?
// Consultar, planificar y cambiar son pasos distintos. Separarlos permite que
// Korunix enseñe primero lo que ocurrirá y que pueda revertir una escritura si
// la nueva declaración no evalúa correctamente.
fn canal_humano(raiz: &Path, canal: &str) -> String {
    valor_canal(raiz, canal, "label").unwrap_or_else(|_| match canal {
        "stable" => "Estable".to_string(),
        "unstable" => "Inestable".to_string(),
        otro => otro.to_string(),
    })
}

fn marca_unica() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duracion| duracion.as_nanos())
        .unwrap_or(0);

    format!("{}-{nanos}", std::process::id())
}

fn archivo_equipo(raiz: &Path, equipo: &str) -> PathBuf {
    raiz.join("configuracion")
        .join("equipos")
        .join(format!("{equipo}.nix"))
}

// Cambia exactamente UNA línea `channel = "...";`.
//
// No busca y reemplaza texto a ciegas: si falta la línea o aparecen dos,
// Korunix se detiene. Así una edición inesperada no puede provocar un cambio
// ambiguo en la computadora.
fn texto_con_canal(texto: &str, objetivo: &str) -> Result<String, String> {
    if !matches!(objetivo, "stable" | "unstable") {
        return Err(format!("Canal desconocido: {objetivo}"));
    }

    let mut salida = String::with_capacity(texto.len());
    let mut encontrados = 0usize;

    for fragmento in texto.split_inclusive('\n') {
        let tiene_salto = fragmento.ends_with('\n');
        let linea = fragmento.strip_suffix('\n').unwrap_or(fragmento);
        let sin_espacios = linea.trim();

        if matches!(
            sin_espacios,
            "channel = \"stable\";" | "channel = \"unstable\";"
        ) {
            let posicion = linea
                .find("channel =")
                .ok_or_else(|| "No pude ubicar la declaración del canal.".to_string())?;
            let prefijo = &linea[..posicion];

            salida.push_str(prefijo);
            salida.push_str("channel = \"");
            salida.push_str(objetivo);
            salida.push_str("\";");
            encontrados += 1;
        } else {
            salida.push_str(linea);
        }

        if tiene_salto {
            salida.push('\n');
        }
    }

    if encontrados != 1 {
        return Err(
            "El archivo del equipo debe declarar exactamente un canal antes de cambiarlo."
                .to_string(),
        );
    }

    Ok(salida)
}

// Escritura atómica.
//
// Primero se prepara un archivo nuevo al lado del original. Solo cuando está
// completo se renombra encima del anterior. Una interrupción no debería dejar
// medio archivo escrito.
fn escribir_canal_atomico(raiz: &Path, equipo: &str, objetivo: &str) -> Result<(), String> {
    let archivo = archivo_equipo(raiz, equipo);
    let original = fs::read_to_string(&archivo)
        .map_err(|error| format!("No pude leer {}: {error}", archivo.display()))?;
    let nuevo = texto_con_canal(&original, objetivo)?;

    let metadata = fs::metadata(&archivo).map_err(|error| {
        format!(
            "No pude leer los permisos de {}: {error}",
            archivo.display()
        )
    })?;
    let temporal = archivo.with_file_name(format!(
        ".{}.canal.{}",
        archivo
            .file_name()
            .and_then(|nombre| nombre.to_str())
            .unwrap_or("equipo.nix"),
        marca_unica()
    ));

    fs::write(&temporal, nuevo)
        .map_err(|error| format!("No pude preparar {}: {error}", temporal.display()))?;

    if let Err(error) = fs::set_permissions(&temporal, metadata.permissions()) {
        let _ = fs::remove_file(&temporal);
        return Err(format!(
            "No pude conservar los permisos de {}: {error}",
            archivo.display()
        ));
    }

    if let Err(error) = fs::rename(&temporal, &archivo) {
        let _ = fs::remove_file(&temporal);
        return Err(format!(
            "No pude sustituir de forma atómica {}: {error}",
            archivo.display()
        ));
    }

    Ok(())
}

// El plan nunca necesita tocar la configuración real.
//
// Cuando se quiere probar otro canal, Korunix copia el checkout a una carpeta
// temporal, cambia únicamente la copia y pide a Nix que evalúe esa copia.
// Al terminar, la carpeta temporal se elimina.
fn copiar_repo_para_plan(raiz: &Path) -> Result<PathBuf, String> {
    let base = env::var_os("TMPDIR")
        .map(PathBuf::from)
        .unwrap_or_else(env::temp_dir);
    let destino = base.join(format!("korunix-canal-plan-{}", marca_unica()));

    fs::create_dir_all(&destino)
        .map_err(|error| format!("No pude crear {}: {error}", destino.display()))?;

    let estado = Command::new("cp")
        .arg("-a")
        .arg("--reflink=auto")
        .arg(raiz.join("."))
        .arg(&destino)
        .status()
        .map_err(|error| format!("No pude copiar Korunix para el plan: {error}"))?;

    if !estado.success() {
        let _ = fs::remove_dir_all(&destino);
        return Err("No pude preparar la copia temporal del plan de canal.".to_string());
    }

    // La copia contiene los cambios de D.2 que todavía no tienen commit. Los
    // hacemos visibles a la evaluación Git solamente DENTRO de la copia.
    let git = Command::new("git")
        .arg("-C")
        .arg(&destino)
        .arg("add")
        .arg("-A")
        .status()
        .map_err(|error| format!("No pude preparar el índice de la copia: {error}"))?;

    if !git.success() {
        let _ = fs::remove_dir_all(&destino);
        return Err("No pude preparar el índice Git temporal del plan.".to_string());
    }

    Ok(destino)
}

fn evaluar_canal_en_raiz(raiz: &Path, equipo: &str) -> Result<(String, String), String> {
    let version = flake_raw(
        raiz,
        &format!("nixosConfigurations.{equipo}.config.system.nixos.version"),
    )?;
    let drv = flake_raw(
        raiz,
        &format!("nixosConfigurations.{equipo}.config.system.build.toplevel.drvPath"),
    )?;

    Ok((version, drv))
}

fn evaluar_canal_git_temporal(raiz: &Path, equipo: &str) -> Result<(String, String), String> {
    let version = ejecutar_capturando(
        "nix",
        &[
            "eval".to_string(),
            format!(".#nixosConfigurations.{equipo}.config.system.nixos.version"),
            "--raw".to_string(),
        ],
        raiz,
    )?;

    let drv = ejecutar_capturando(
        "nix",
        &[
            "eval".to_string(),
            format!(".#nixosConfigurations.{equipo}.config.system.build.toplevel.drvPath"),
            "--raw".to_string(),
        ],
        raiz,
    )?;

    Ok((version, drv))
}

fn evaluar_canal_objetivo_rust(
    raiz: &Path,
    equipo: &str,
    objetivo: &str,
) -> Result<(String, String), String> {
    let actual = canal_declarado(raiz, equipo)?;
    let copia = copiar_repo_para_plan(raiz)?;

    let resultado = (|| {
        if actual != objetivo {
            escribir_canal_atomico(&copia, equipo, objetivo)?;
        }

        // La evaluación ocurre como flake Git dentro de la copia. Eso conserva
        // la misma procedencia que tenía el contrato heredado y evita que una
        // ruta temporal cambie artificialmente la derivación resultante.
        evaluar_canal_git_temporal(&copia, equipo)
    })();

    let limpieza = fs::remove_dir_all(&copia);

    if let Err(error) = limpieza {
        if resultado.is_ok() {
            return Err(format!(
                "El plan terminó, pero no pude retirar la copia temporal {}: {error}",
                copia.display()
            ));
        }
    }

    resultado
}

fn plan_canal_json(raiz: &Path, equipo: &str, objetivo: &str) -> Result<String, String> {
    if !matches!(objetivo, "stable" | "unstable") {
        return Err(format!("Canal desconocido: {objetivo}"));
    }

    let actual = canal_declarado(raiz, equipo)?;
    let etiqueta = canal_humano(raiz, objetivo);
    let descripcion = valor_canal(raiz, objetivo, "description")?;
    let nixpkgs = valor_canal(raiz, objetivo, "nixpkgs_ref")?;
    let aagl = valor_canal(raiz, objetivo, "aagl_ref")?;
    let (version, drv) = evaluar_canal_objetivo_rust(raiz, equipo, objetivo)?;
    let cambia = actual != objetivo;

    Ok(format!(
        concat!(
            "{{",
            "\"schemaVersion\":1,",
            "\"hostId\":{},",
            "\"current\":{},",
            "\"target\":{},",
            "\"targetLabel\":{},",
            "\"description\":{},",
            "\"changed\":{},",
            "\"valid\":true,",
            "\"nixosVersion\":{},",
            "\"drvPath\":{},",
            "\"sources\":{{\"nixpkgs\":{},\"aagl\":{}}},",
            "\"modifiesHost\":false,",
            "\"buildsGeneration\":false,",
            "\"appliesGeneration\":false",
            "}}"
        ),
        json_texto(equipo),
        json_texto(&actual),
        json_texto(objetivo),
        json_texto(&etiqueta),
        json_texto(&descripcion),
        cambia,
        json_texto(&version),
        json_texto(&drv),
        json_texto(&nixpkgs),
        json_texto(&aagl),
    ))
}

fn mostrar_canal_actual_humano(raiz: &Path, equipo: &str) -> Result<(), String> {
    let canal = canal_declarado(raiz, equipo)?;
    let etiqueta = canal_humano(raiz, &canal);
    let descripcion = valor_canal(raiz, &canal, "description")?;
    let nixpkgs = valor_canal(raiz, &canal, "nixpkgs_ref")?;
    let aagl = valor_canal(raiz, &canal, "aagl_ref")?;
    let version = flake_raw(
        raiz,
        &format!("nixosConfigurations.{equipo}.config.system.nixos.version"),
    )
    .unwrap_or_default();

    println!("=== Canal de actualizaciones ===");
    println!("Equipo: {equipo}");
    println!("Canal actual: {etiqueta}");

    if !descripcion.is_empty() {
        println!("{descripcion}");
    }

    if !version.is_empty() {
        println!("NixOS efectivo: {version}");
    }

    if !nixpkgs.is_empty() {
        println!("Nixpkgs: {nixpkgs}");
    }

    if !aagl.is_empty() {
        println!("AAGL: {aagl}");
    }

    println!();
    println!("system.stateVersion no cambia al cambiar de canal.");

    Ok(())
}

fn mostrar_plan_canal_humano(raiz: &Path, equipo: &str, objetivo: &str) -> Result<(), String> {
    let actual = canal_declarado(raiz, equipo)?;
    let etiqueta_actual = canal_humano(raiz, &actual);
    let etiqueta_objetivo = canal_humano(raiz, objetivo);
    let descripcion = valor_canal(raiz, objetivo, "description")?;
    let nixpkgs = valor_canal(raiz, objetivo, "nixpkgs_ref")?;
    let aagl = valor_canal(raiz, objetivo, "aagl_ref")?;

    println!("=== Plan de cambio de canal ===");
    println!("Equipo: {equipo}");
    println!("Actual: {etiqueta_actual}");
    println!("Objetivo: {etiqueta_objetivo}");

    if !descripcion.is_empty() {
        println!("{descripcion}");
    }

    println!();
    println!("Fuentes que utilizaría:");

    if !nixpkgs.is_empty() {
        println!("  Nixpkgs: {nixpkgs}");
    }

    if !aagl.is_empty() {
        println!("  AAGL: {aagl}");
    }

    println!();
    println!("Evaluando una copia temporal sin aplicar ninguna generación...");

    let (version, drv) = evaluar_canal_objetivo_rust(raiz, equipo, objetivo)?;

    println!("✓ Evaluación correcta");
    println!("NixOS resultante: {version}");
    println!("Derivación: {drv}");

    if objetivo == actual {
        println!();
        println!("No hay cambio: ese canal ya está declarado.");
    } else {
        println!();
        println!("Este plan no modificó configuracion/equipos/{equipo}.nix.");
        println!("No se construyó ni aplicó ninguna generación.");
    }

    Ok(())
}

fn confirmar_cambio_canal(
    equipo: &str,
    actual: &str,
    objetivo: &str,
    raiz: &Path,
) -> Result<bool, String> {
    if !io::stdin().is_terminal() {
        return Err("El cambio de canal necesita confirmación interactiva o --yes.".to_string());
    }

    println!();
    print!(
        "¿Cambiar {equipo} de {} a {}? [s/N] ",
        canal_humano(raiz, actual),
        canal_humano(raiz, objetivo)
    );

    use std::io::Write;
    io::stdout()
        .flush()
        .map_err(|error| format!("No pude mostrar la confirmación: {error}"))?;

    let mut respuesta = String::new();
    io::stdin()
        .read_line(&mut respuesta)
        .map_err(|error| format!("No pude leer la confirmación: {error}"))?;

    let normalizada = respuesta.trim().to_lowercase();
    let acepta = matches!(normalizada.as_str(), "s" | "si" | "sí");

    if !acepta {
        println!("Operación cancelada.");
    }

    Ok(acepta)
}

fn directorio_estado() -> Result<PathBuf, String> {
    if let Some(valor) = env::var_os("XDG_STATE_HOME") {
        return Ok(PathBuf::from(valor));
    }

    let home = env::var_os("HOME")
        .ok_or_else(|| "No pude encontrar HOME para guardar el respaldo.".to_string())?;

    Ok(PathBuf::from(home).join(".local/state"))
}

fn crear_respaldo_canal(
    raiz: &Path,
    equipo: &str,
    actual: &str,
    objetivo: &str,
) -> Result<PathBuf, String> {
    let fecha = ejecutar_capturando("date", &["+%Y%m%d-%H%M%S".to_string()], raiz)
        .unwrap_or_else(|_| marca_unica());

    let respaldo = directorio_estado()?.join("korunix/backups").join(format!(
        "canal-{equipo}-{actual}-a-{objetivo}-{fecha}-{}",
        std::process::id()
    ));

    let carpeta_equipos = respaldo.join("equipos");
    fs::create_dir_all(&carpeta_equipos).map_err(|error| {
        format!(
            "No pude crear el respaldo {}: {error}",
            carpeta_equipos.display()
        )
    })?;

    let origen = archivo_equipo(raiz, equipo);
    let destino = carpeta_equipos.join(format!("{equipo}.nix"));

    fs::copy(&origen, &destino).map_err(|error| {
        format!(
            "No pude respaldar {} en {}: {error}",
            origen.display(),
            destino.display()
        )
    })?;

    Ok(respaldo)
}

fn restaurar_desde_respaldo(raiz: &Path, equipo: &str, respaldo: &Path) -> Result<(), String> {
    let origen = respaldo.join("equipos").join(format!("{equipo}.nix"));
    let destino = archivo_equipo(raiz, equipo);

    let contenido = fs::read_to_string(&origen)
        .map_err(|error| format!("No pude leer el respaldo {}: {error}", origen.display()))?;
    let metadata = fs::metadata(&destino)
        .map_err(|error| format!("No pude leer {}: {error}", destino.display()))?;

    let temporal = destino.with_file_name(format!(
        ".{}.restauracion.{}",
        destino
            .file_name()
            .and_then(|nombre| nombre.to_str())
            .unwrap_or("equipo.nix"),
        marca_unica()
    ));

    fs::write(&temporal, contenido)
        .map_err(|error| format!("No pude preparar la restauración: {error}"))?;
    fs::set_permissions(&temporal, metadata.permissions())
        .map_err(|error| format!("No pude conservar los permisos al restaurar: {error}"))?;
    fs::rename(&temporal, &destino)
        .map_err(|error| format!("No pude restaurar {}: {error}", destino.display()))?;

    Ok(())
}

fn validar_canal_escrito(raiz: &Path, equipo: &str, objetivo: &str) -> Result<(), String> {
    let declarado = canal_declarado(raiz, equipo)?;
    if declarado != objetivo {
        return Err(format!(
            "El archivo declara {declarado}, pero se esperaba {objetivo}."
        ));
    }

    let efectivo = flake_raw(
        raiz,
        &format!("nixosConfigurations.{equipo}.config.korunix.channel"),
    )?;
    if efectivo != objetivo {
        return Err(format!(
            "Nix evalúa el canal como {efectivo}, pero se esperaba {objetivo}."
        ));
    }

    let _ = evaluar_canal_en_raiz(raiz, equipo)?;

    let estado = Command::new("git")
        .arg("-C")
        .arg(raiz)
        .arg("diff")
        .arg("--check")
        .arg("--")
        .arg(format!("configuracion/equipos/{equipo}.nix"))
        .status()
        .map_err(|error| format!("No pude comprobar el diff del canal: {error}"))?;

    if !estado.success() {
        return Err("El cambio de canal produjo un diff inválido.".to_string());
    }

    Ok(())
}

fn cambiar_canal_rust(
    raiz: &Path,
    equipo: &str,
    objetivo: &str,
    confirmado: bool,
) -> Result<(), String> {
    if !matches!(objetivo, "stable" | "unstable") {
        return Err(format!("Canal desconocido: {objetivo}"));
    }

    let actual = canal_declarado(raiz, equipo)?;

    mostrar_plan_canal_humano(raiz, equipo, objetivo)?;

    if objetivo == actual {
        return Ok(());
    }

    if !confirmado && !confirmar_cambio_canal(equipo, &actual, objetivo, raiz)? {
        return Ok(());
    }

    let respaldo = crear_respaldo_canal(raiz, equipo, &actual, objetivo)?;

    escribir_canal_atomico(raiz, equipo, objetivo)?;

    println!();
    println!("Validando la nueva declaración...");

    if let Err(error_validacion) = validar_canal_escrito(raiz, equipo, objetivo) {
        eprintln!();
        eprintln!("ERROR: la nueva declaración no superó la validación.");
        eprintln!("Restaurando el canal anterior...");

        restaurar_desde_respaldo(raiz, equipo, &respaldo)?;
        eprintln!("✓ Canal anterior restaurado");

        return Err(format!(
            "El cambio de canal fue revertido antes de aplicar. {error_validacion}"
        ));
    }

    println!();
    println!("========================================");
    println!(" CANAL PREPARADO");
    println!("========================================");
    println!("Equipo: {equipo}");
    println!("Canal: {}", canal_humano(raiz, objetivo));
    println!("✓ La declaración cambió");
    println!("✓ La configuración evalúa correctamente");
    println!("✓ flake.lock no necesita reescribirse");
    println!("✓ No se aplicó ninguna generación");
    println!();
    println!("Para revisar el cambio antes de aplicarlo:");
    println!("  korunix preview");
    println!();
    println!("Para aplicarlo posteriormente:");
    println!("  korunix apply");
    println!();
    println!("Respaldo: {}", respaldo.display());

    Ok(())
}

fn ayuda_canal() {
    println!(
        r#"Uso:
  korunix channel
      Mostrar el canal actual y sus fuentes.

  korunix channel stable --plan
  korunix channel unstable --plan
      Evaluar otro canal usando una copia temporal.
      No modifica la configuración real.

  korunix channel stable --plan --json
  korunix channel unstable --plan --json
      Entregar el mismo plan como datos estructurados.

  korunix channel stable
  korunix channel unstable
      Mostrar el plan, pedir confirmación y cambiar solo la declaración.

  korunix channel stable --yes
  korunix channel unstable --yes
      Igual que el cambio anterior, pero una interfaz superior ya confirmó.

Cambiar el canal nunca modifica system.stateVersion y nunca aplica
automáticamente una generación."#
    );
}

fn ejecutar_canal(raiz: &Path, argumentos: &[OsString]) -> Result<ExitCode, String> {
    let equipo = resolver_equipo(raiz)?;
    let texto: Vec<String> = argumentos
        .iter()
        .map(|valor| valor.to_string_lossy().into_owned())
        .collect();

    match texto.as_slice() {
        [] => {
            mostrar_canal_actual_humano(raiz, &equipo)?;
        }
        [opcion] if opcion == "--json" => {
            println!("{}", canal_json(raiz)?);
        }
        [opcion] if opcion == "-h" || opcion == "--help" => {
            ayuda_canal();
        }
        [objetivo] if matches!(objetivo.as_str(), "stable" | "unstable") => {
            cambiar_canal_rust(raiz, &equipo, objetivo, false)?;
        }
        [objetivo, modo]
            if matches!(objetivo.as_str(), "stable" | "unstable") && modo == "--yes" =>
        {
            cambiar_canal_rust(raiz, &equipo, objetivo, true)?;
        }
        [objetivo, modo]
            if matches!(objetivo.as_str(), "stable" | "unstable") && modo == "--plan" =>
        {
            mostrar_plan_canal_humano(raiz, &equipo, objetivo)?;
        }
        [objetivo, plan, formato]
            if matches!(objetivo.as_str(), "stable" | "unstable")
                && plan == "--plan"
                && formato == "--json" =>
        {
            println!("{}", plan_canal_json(raiz, &equipo, objetivo)?);
        }
        _ => {
            return Err(
                "Uso: korunix channel [--json | stable|unstable [--plan [--json]|--yes]]."
                    .to_string(),
            );
        }
    }

    Ok(ExitCode::SUCCESS)
}

// ---------------------------------------------------------------------------
// D.2 · Lecturas del equipo
// ---------------------------------------------------------------------------
//
// Estas tres lecturas mezclan dos clases de información:
//
//   1. decisiones declaradas en Nix;
//   2. hechos del sistema que está funcionando ahora.
//
// Nix sigue siendo la fuente de verdad de lo declarativo. Rust consulta el
// mundo vivo mediante interfaces normales del sistema y entrega el contrato
// JSON. No hace falta pasar por Bash para leer hardware, localización o
// personas.
//
// `jq` permanece temporalmente como herramienta de composición JSON en esta
// etapa. No decide políticas ni ejecuta operaciones privilegiadas.

fn ejecutar_con_entrada(
    programa: &str,
    argumentos: &[String],
    raiz: &Path,
    entrada: &str,
) -> Result<String, String> {
    let mut hijo = Command::new(programa)
        .args(argumentos)
        .current_dir(raiz)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("No pude ejecutar {programa}: {error}"))?;

    if let Some(mut stdin) = hijo.stdin.take() {
        use std::io::Write;
        stdin
            .write_all(entrada.as_bytes())
            .map_err(|error| format!("No pude entregar datos a {programa}: {error}"))?;
    }

    let salida = hijo
        .wait_with_output()
        .map_err(|error| format!("No pude esperar a {programa}: {error}"))?;

    if !salida.status.success() {
        let error = String::from_utf8_lossy(&salida.stderr).trim().to_string();
        return Err(if error.is_empty() {
            format!("{programa} terminó con un error.")
        } else {
            error
        });
    }

    Ok(String::from_utf8_lossy(&salida.stdout).trim().to_string())
}

fn jq_con_entrada(raiz: &Path, argumentos: &[String], entrada: &str) -> Result<String, String> {
    ejecutar_con_entrada("jq", argumentos, raiz, entrada)
}

fn jq_texto(raiz: &Path, datos: &str, filtro: &str) -> Result<String, String> {
    jq_con_entrada(raiz, &["-r".to_string(), filtro.to_string()], datos)
}

fn jq_compacto(raiz: &Path, datos: &str, filtro: &str) -> Result<String, String> {
    jq_con_entrada(raiz, &["-c".to_string(), filtro.to_string()], datos)
}

fn flake_json(raiz: &Path, atributo: &str) -> Result<String, String> {
    let instalable = format!("path:{}#{atributo}", raiz.display());
    ejecutar_capturando(
        "nix",
        &["eval".to_string(), "--json".to_string(), instalable],
        raiz,
    )
}

fn capturar_opcional(programa: &str, argumentos: &[&str], raiz: &Path) -> String {
    match Command::new(programa)
        .args(argumentos)
        .current_dir(raiz)
        .output()
    {
        Ok(salida) => String::from_utf8_lossy(&salida.stdout).trim().to_string(),
        Err(_) => String::new(),
    }
}

fn comando_exitoso(programa: &str, argumentos: &[&str], raiz: &Path) -> bool {
    Command::new(programa)
        .args(argumentos)
        .current_dir(raiz)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|estado| estado.success())
        .unwrap_or(false)
}

fn fecha_iso(raiz: &Path) -> Result<String, String> {
    ejecutar_capturando("date", &["--iso-8601=seconds".to_string()], raiz)
}

fn json_lista_textos(valores: &[String]) -> String {
    format!(
        "[{}]",
        valores
            .iter()
            .map(|valor| json_texto(valor))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn leer_texto(ruta: &Path) -> String {
    fs::read(ruta)
        .ok()
        .map(|bytes| {
            String::from_utf8_lossy(&bytes)
                .replace('\0', "")
                .trim_end()
                .to_string()
        })
        .unwrap_or_default()
}

fn valor_cpu(campo: &str) -> String {
    let texto = fs::read_to_string("/proc/cpuinfo").unwrap_or_default();

    for linea in texto.lines() {
        let Some((nombre, valor)) = linea.split_once(':') else {
            continue;
        };

        let coincide = match campo {
            "modelo" => matches!(nombre.trim(), "model name" | "Processor" | "Hardware"),
            "fabricante" => nombre.trim() == "vendor_id",
            "microcode" => nombre.trim() == "microcode",
            _ => false,
        };

        if coincide {
            return valor.trim_start().to_string();
        }
    }

    String::new()
}

fn procesadores_logicos() -> usize {
    fs::read_to_string("/proc/cpuinfo")
        .unwrap_or_default()
        .lines()
        .filter(|linea| {
            linea
                .split_once(':')
                .map(|(nombre, _)| nombre.trim() == "processor")
                .unwrap_or(false)
        })
        .count()
}

fn memoria_bytes() -> u64 {
    fs::read_to_string("/proc/meminfo")
        .unwrap_or_default()
        .lines()
        .find_map(|linea| {
            linea
                .strip_prefix("MemTotal:")
                .and_then(|resto| resto.split_whitespace().next())
                .and_then(|valor| valor.parse::<u64>().ok())
        })
        .unwrap_or(0)
        .saturating_mul(1024)
}

fn bateria_de_sistema() -> bool {
    let Ok(entradas) = fs::read_dir("/sys/class/power_supply") else {
        return false;
    };

    for entrada in entradas.flatten() {
        let ruta = entrada.path();
        if leer_texto(&ruta.join("type")) == "Battery"
            && leer_texto(&ruta.join("scope")) == "System"
        {
            return true;
        }
    }

    false
}

fn tipo_equipo(chasis: &str) -> &'static str {
    match chasis {
        "3" | "4" | "5" | "6" | "7" | "15" | "16" | "24" => "desktop",
        "8" | "9" | "10" | "14" | "30" | "31" | "32" => "laptop",
        "13" => "all-in-one",
        "17" | "23" => "server",
        _ => "unknown",
    }
}

fn lineas_pci(raiz: &Path, clase: &str) -> Vec<String> {
    let salida = capturar_opcional("lspci", &["-nn"], raiz);
    let mut valores = Vec::new();

    for linea in salida.lines() {
        let coincide = match clase {
            "graphics" => {
                linea.contains("[0300]") || linea.contains("[0302]") || linea.contains("[0380]")
            }
            "network" => linea.contains("[0200]") || linea.contains("[0280]"),
            _ => false,
        };

        if !coincide {
            continue;
        }

        let sin_direccion = linea
            .find(char::is_whitespace)
            .map(|posicion| linea[posicion..].trim_start().to_string())
            .unwrap_or_else(|| linea.to_string());

        if !sin_direccion.is_empty() {
            valores.push(sin_direccion);
        }
    }

    valores
}

fn drivers_graficos(raiz: &Path) -> Vec<String> {
    let salida = capturar_opcional("lsmod", &[], raiz);
    let mut drivers = Vec::new();

    for linea in salida.lines() {
        let Some(nombre) = linea.split_whitespace().next() else {
            continue;
        };

        if matches!(nombre, "amdgpu" | "nvidia" | "nouveau" | "i915" | "xe") {
            drivers.push(nombre.to_string());
        }
    }

    drivers.sort();
    drivers.dedup();
    drivers
}

fn id_pci(ruta: &Path, nombre: &str) -> String {
    leer_texto(&ruta.join(nombre))
        .to_ascii_uppercase()
        .trim_start_matches("0X")
        .to_string()
}

fn nvidia_open_soportado(
    raiz: &Path,
    device_id: &str,
    subsystem_vendor_id: &str,
    subsystem_device_id: &str,
) -> bool {
    let catalogo = raiz.join("sistema/nvidia-open-pci-ids.txt");
    let Ok(texto) = fs::read_to_string(catalogo) else {
        return false;
    };

    for linea in texto.lines() {
        if linea.trim().is_empty() || linea.starts_with('#') {
            continue;
        }

        let columnas: Vec<&str> = linea.split('\t').collect();
        if columnas.len() < 3 || columnas[0] != device_id {
            continue;
        }

        if columnas[1] == "*" && columnas[2] == "*" {
            return true;
        }

        if columnas[1] == subsystem_vendor_id && columnas[2] == subsystem_device_id {
            return true;
        }
    }

    false
}

fn dispositivos_graficos_json(raiz: &Path) -> String {
    let Ok(entradas) = fs::read_dir("/sys/bus/pci/devices") else {
        return "[]".to_string();
    };

    let mut rutas: Vec<PathBuf> = entradas.flatten().map(|entrada| entrada.path()).collect();
    rutas.sort();

    let mut objetos = Vec::new();

    for dispositivo in rutas {
        let clase = id_pci(&dispositivo, "class");
        if !matches!(clase.as_str(), "030000" | "030200" | "038000") {
            continue;
        }

        let direccion = dispositivo
            .file_name()
            .and_then(|nombre| nombre.to_str())
            .unwrap_or("")
            .to_string();

        let vendor_id = id_pci(&dispositivo, "vendor");
        let device_id = id_pci(&dispositivo, "device");
        let subsystem_vendor_id = {
            let valor = id_pci(&dispositivo, "subsystem_vendor");
            if valor.is_empty() {
                "0000".to_string()
            } else {
                valor
            }
        };
        let subsystem_device_id = {
            let valor = id_pci(&dispositivo, "subsystem_device");
            if valor.is_empty() {
                "0000".to_string()
            } else {
                valor
            }
        };

        let vendor = match vendor_id.as_str() {
            "1002" => "amd",
            "8086" => "intel",
            "10DE" => "nvidia",
            _ => "unknown",
        };

        let driver = fs::read_link(dispositivo.join("driver"))
            .ok()
            .and_then(|ruta| fs::canonicalize(dispositivo.join(ruta)).ok())
            .and_then(|ruta| {
                ruta.file_name()
                    .map(|nombre| nombre.to_string_lossy().to_string())
            })
            .unwrap_or_else(|| "none".to_string());

        let primary = leer_texto(&dispositivo.join("boot_vga")) == "1";

        let lspci = capturar_opcional("lspci", &["-Dnn", "-s", &direccion], raiz);
        let nombre = if lspci.is_empty() {
            format!("GPU PCI {vendor_id}:{device_id}")
        } else {
            lspci
                .find(' ')
                .map(|posicion| lspci[posicion + 1..].to_string())
                .unwrap_or(lspci)
        };

        let nvidia_open = vendor == "nvidia"
            && nvidia_open_soportado(raiz, &device_id, &subsystem_vendor_id, &subsystem_device_id);

        objetos.push(format!(
            concat!(
                "{{",
                "\"pciAddress\":{},",
                "\"name\":{},",
                "\"vendor\":{},",
                "\"vendorId\":{},",
                "\"deviceId\":{},",
                "\"subsystemVendorId\":{},",
                "\"subsystemDeviceId\":{},",
                "\"class\":{},",
                "\"driver\":{},",
                "\"primary\":{},",
                "\"kind\":\"unknown\",",
                "\"nvidiaOpen\":{}",
                "}}"
            ),
            json_texto(&direccion),
            json_texto(&nombre),
            json_texto(vendor),
            json_texto(&vendor_id),
            json_texto(&device_id),
            json_texto(&subsystem_vendor_id),
            json_texto(&subsystem_device_id),
            json_texto(&clase),
            json_texto(&driver),
            primary,
            nvidia_open,
        ));
    }

    format!("[{}]", objetos.join(","))
}

fn hardware_json(raiz: &Path) -> Result<String, String> {
    let equipo = resolver_equipo(raiz)?;

    let arch_kernel = ejecutar_capturando("uname", &["-m".to_string()], raiz)?;
    let arch_detectada = match arch_kernel.as_str() {
        "x86_64" => "x86_64-linux".to_string(),
        "aarch64" | "arm64" => "aarch64-linux".to_string(),
        otro => format!("{otro}-linux"),
    };

    let firmware_detectado = if Path::new("/sys/firmware/efi").is_dir() {
        "uefi"
    } else {
        "bios"
    };

    let runtime = runtime_state_current(raiz)?;

    let arch_declarada = if let Some(value) = runtime
        .as_ref()
        .and_then(|state| state.pointer("/hardware/platform"))
        .and_then(serde_json::Value::as_str)
    {
        value.to_string()
    } else {
        flake_raw(
            raiz,
            &format!("nixosConfigurations.{equipo}.config.nixpkgs.hostPlatform.system"),
        )
        .unwrap_or_default()
    };

    let firmware_declarado = if let Some(value) = runtime
        .as_ref()
        .and_then(|state| state.pointer("/hardware/firmware"))
        .and_then(serde_json::Value::as_str)
    {
        value.to_string()
    } else {
        flake_raw(
            raiz,
            &format!("nixosConfigurations.{equipo}.config.korunix.hardware.firmware"),
        )
        .unwrap_or_default()
    };

    let virtualizacion_raw = capturar_opcional("systemd-detect-virt", &[], raiz);
    let (virtualizacion, virtualizado) =
        if virtualizacion_raw.is_empty() || virtualizacion_raw == "none" {
            ("physical".to_string(), false)
        } else {
            (virtualizacion_raw, true)
        };

    let fabricante = {
        let valor = leer_texto(Path::new("/sys/class/dmi/id/sys_vendor"));
        if valor.is_empty() {
            "desconocido".to_string()
        } else {
            valor
        }
    };
    let modelo = {
        let valor = leer_texto(Path::new("/sys/class/dmi/id/product_name"));
        if valor.is_empty() {
            "desconocido".to_string()
        } else {
            valor
        }
    };
    let placa_fabricante = {
        let valor = leer_texto(Path::new("/sys/class/dmi/id/board_vendor"));
        if valor.is_empty() {
            "desconocido".to_string()
        } else {
            valor
        }
    };
    let placa_modelo = {
        let valor = leer_texto(Path::new("/sys/class/dmi/id/board_name"));
        if valor.is_empty() {
            "desconocido".to_string()
        } else {
            valor
        }
    };
    let chasis = leer_texto(Path::new("/sys/class/dmi/id/chassis_type"));

    let cpu_modelo = {
        let valor = valor_cpu("modelo");
        if valor.is_empty() {
            "desconocido".to_string()
        } else {
            valor
        }
    };
    let cpu_fabricante = {
        let valor = valor_cpu("fabricante");
        if valor.is_empty() {
            "desconocido".to_string()
        } else {
            valor
        }
    };
    let microcode = {
        let valor = valor_cpu("microcode");
        if valor.is_empty() {
            "desconocido".to_string()
        } else {
            valor
        }
    };

    let graphics = lineas_pci(raiz, "graphics");
    let network = lineas_pci(raiz, "network");
    let drivers = drivers_graficos(raiz);
    let graphics_devices = dispositivos_graficos_json(raiz);

    let storage = capturar_opcional(
        "lsblk",
        &[
            "-J",
            "-b",
            "-e",
            "7",
            "-o",
            "NAME,PATH,TYPE,SIZE,FSTYPE,MOUNTPOINTS,MODEL,TRAN,ROTA,RM",
        ],
        raiz,
    );
    let storage = if storage.is_empty() {
        "{\"blockdevices\":[]}".to_string()
    } else {
        storage
    };

    Ok(format!(
        concat!(
            "{{",
            "\"schemaVersion\":1,",
            "\"hostId\":{},",
            "\"detectedAt\":{},",
            "\"machine\":{{",
            "\"type\":{},",
            "\"chassisType\":{},",
            "\"vendor\":{},",
            "\"model\":{},",
            "\"boardVendor\":{},",
            "\"boardModel\":{},",
            "\"systemBattery\":{}",
            "}},",
            "\"platform\":{{\"detected\":{},\"declared\":{},\"matches\":{}}},",
            "\"firmware\":{{\"detected\":{},\"declared\":{},\"matches\":{}}},",
            "\"virtualization\":{{\"kind\":{},\"virtualized\":{}}},",
            "\"cpu\":{{",
            "\"model\":{},",
            "\"vendor\":{},",
            "\"logicalProcessors\":{},",
            "\"microcode\":{}",
            "}},",
            "\"memory\":{{\"bytes\":{}}},",
            "\"graphics\":{},",
            "\"graphicsDrivers\":{},",
            "\"graphicsDevices\":{},",
            "\"network\":{},",
            "\"storage\":{}",
            "}}"
        ),
        json_texto(&equipo),
        json_texto(&fecha_iso(raiz)?),
        json_texto(tipo_equipo(&chasis)),
        json_texto(&chasis),
        json_texto(&fabricante),
        json_texto(&modelo),
        json_texto(&placa_fabricante),
        json_texto(&placa_modelo),
        bateria_de_sistema(),
        json_texto(&arch_detectada),
        json_texto(&arch_declarada),
        arch_detectada == arch_declarada,
        json_texto(firmware_detectado),
        json_texto(&firmware_declarado),
        firmware_detectado == firmware_declarado,
        json_texto(&virtualizacion),
        virtualizado,
        json_texto(&cpu_modelo),
        json_texto(&cpu_fabricante),
        procesadores_logicos(),
        json_texto(&microcode),
        memoria_bytes(),
        json_lista_textos(&graphics),
        json_lista_textos(&drivers),
        graphics_devices,
        json_lista_textos(&network),
        storage,
    ))
}

fn uid_minimo() -> u32 {
    let texto = fs::read_to_string("/etc/login.defs").unwrap_or_default();

    for linea in texto.lines() {
        let sin_espacios = linea.trim_start();
        if sin_espacios.starts_with('#') {
            continue;
        }

        let mut partes = sin_espacios.split_whitespace();
        if partes.next() == Some("UID_MIN") {
            if let Some(valor) = partes.next().and_then(|valor| valor.parse::<u32>().ok()) {
                return valor;
            }
        }
    }

    1000
}

fn cuenta_tecnica(home: &str, shell: &str) -> bool {
    home == "/var/empty"
        || home.starts_with("/run/gdm/")
        || shell.ends_with("nologin")
        || shell.ends_with("false")
}

fn perfil_base_json(raiz: &Path, ruta: &Path, id: &str) -> Result<String, String> {
    let base = nix_archivo_json(raiz, ruta)?;

    jq_con_entrada(
        raiz,
        &[
            "-cn".to_string(),
            "--arg".to_string(),
            "id".to_string(),
            id.to_string(),
            "--argjson".to_string(),
            "u".to_string(),
            base,
            r#"{
                id: $id,
                accountName: ($u.accountName // $id),
                fullName: ($u.fullName // ""),
                language: ($u.language // null),
                interfaceLanguage: ($u.interfaceLanguage // null),
                inputMethods: ($u.inputMethods // []),
                capabilities: ($u.capabilities // []),
                avatarPath: ($u.avatar // null)
            }"#
            .to_string(),
        ],
        "",
    )
}

fn usuarios_json_runtime(
    raiz: &Path,
    equipo: &str,
    state: &serde_json::Value,
) -> Result<Option<String>, String> {
    let Some(people) = state.get("people") else {
        return Ok(None);
    };
    let Some(raw_profiles) = people.get("profiles").and_then(serde_json::Value::as_array) else {
        return Ok(None);
    };

    let passwd = capturar_opcional("getent", &["passwd"], raiz);
    let minimo = uid_minimo();

    let mut profiles = raw_profiles.clone();
    for profile in &mut profiles {
        let Some(object) = profile.as_object_mut() else {
            return Ok(None);
        };
        let account = object
            .get("effectiveAccountName")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let exists = passwd
            .lines()
            .any(|line| line.split(':').next() == Some(account));
        object.insert("accountExists".to_string(), serde_json::json!(exists));
    }

    let mut cuentas = Vec::<serde_json::Value>::new();
    let mut detectados_admin = 0usize;
    let mut adoptados = 0usize;
    let mut adoptables = 0usize;

    for linea in passwd.lines() {
        let campos = linea.split(':').collect::<Vec<_>>();
        if campos.len() < 7 {
            continue;
        }

        let cuenta = campos[0];
        let Ok(uid) = campos[2].parse::<u32>() else {
            continue;
        };

        if uid < minimo || uid >= 65534 {
            continue;
        }

        let home = campos[5];
        let shell = campos[6];
        if cuenta_tecnica(home, shell) {
            continue;
        }

        let display_name = campos[4]
            .split(',')
            .next()
            .filter(|value| !value.is_empty())
            .unwrap_or(cuenta);

        let grupos_texto = capturar_opcional("id", &["-nG", cuenta], raiz);
        let mut grupos = grupos_texto
            .split_whitespace()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        grupos.sort();
        grupos.dedup();

        let administrador = grupos.iter().any(|group| group == "wheel");
        if administrador {
            detectados_admin += 1;
        }

        let profile = profiles.iter().find(|profile| {
            profile
                .get("effectiveAccountName")
                .and_then(serde_json::Value::as_str)
                == Some(cuenta)
        });

        let (profile_id, status) = match profile {
            Some(profile)
                if profile
                    .get("assignedToHost")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false) =>
            {
                adoptados += 1;
                (
                    profile
                        .get("id")
                        .cloned()
                        .unwrap_or(serde_json::Value::Null),
                    "adopted",
                )
            }
            Some(profile) => (
                profile
                    .get("id")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null),
                "profile-available",
            ),
            None => {
                adoptables += 1;
                (serde_json::Value::Null, "adoptable")
            }
        };

        cuentas.push(serde_json::json!({
            "accountName": cuenta,
            "displayName": display_name,
            "uid": uid,
            "home": home,
            "shell": shell,
            "administrator": administrador,
            "groups": grupos,
            "profileId": profile_id,
            "status": status
        }));
    }

    let declarados_admin = profiles
        .iter()
        .filter(|profile| {
            profile
                .get("assignedToHost")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
                && profile
                    .get("administrator")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false)
        })
        .count();

    let output = serde_json::json!({
        "schemaVersion": 2,
        "hostId": equipo,
        "detectedAt": fecha_iso(raiz)?,
        "accounts": cuentas,
        "profiles": profiles,
        "summary": {
            "humanAccounts": cuentas.len(),
            "adoptedAccounts": adoptados,
            "adoptableAccounts": adoptables,
            "detectedAdministrators": detectados_admin,
            "declaredAdministrators": declarados_admin
        },
        "hostUserIds": people
            .get("hostUserIds")
            .cloned()
            .unwrap_or_else(|| serde_json::json!([])),
        "policy": {
            "mutableUsers": people
                .get("mutableUsers")
                .cloned()
                .unwrap_or(serde_json::Value::Bool(true)),
            "preserveExistingPasswords": true,
            "repositoryStoresPasswords": false,
            "newPasswordMethod": "system-passwd",
            "androidAccessModel": "systemd-uaccess",
            "portableProfileSchemaVersion": 3,
            "compatiblePortableProfileSchemaVersions": [1, 2, 3],
            "portableFields": [
                "id",
                "accountName",
                "fullName",
                "language",
                "interfaceLanguage",
                "inputMethods",
                "capabilities",
                "avatar"
            ],
            "hostLocalFields": [
                "homeDirectory",
                "administrator",
                "deferredCapabilities",
                "deferredInputMethods",
                "preservedGroups",
                "password"
            ]
        }
    });

    Ok(Some(output.to_string()))
}

fn usuarios_json(raiz: &Path) -> Result<String, String> {
    let equipo = resolver_equipo(raiz)?;

    if let Some(runtime) = runtime_state_current(raiz)? {
        if let Some(output) = usuarios_json_runtime(raiz, &equipo, &runtime)? {
            return Ok(output);
        }
    }

    let host_users = flake_json(
        raiz,
        &format!("nixosConfigurations.{equipo}.config.korunix.users"),
    )?;
    let settings = flake_json(
        raiz,
        &format!("nixosConfigurations.{equipo}.config.korunix.userSettings"),
    )?;
    let mutable_users = flake_json(
        raiz,
        &format!("nixosConfigurations.{equipo}.config.users.mutableUsers"),
    )?;

    let carpeta = raiz.join("configuracion/personas");
    let mut archivos: Vec<PathBuf> = fs::read_dir(&carpeta)
        .map_err(|error| format!("No pude leer {}: {error}", carpeta.display()))?
        .flatten()
        .map(|entrada| entrada.path())
        .filter(|ruta| ruta.is_file() && ruta.extension().and_then(|e| e.to_str()) == Some("nix"))
        .collect();
    archivos.sort();

    let mut perfiles = Vec::new();
    let mut perfil_cuenta = Vec::new();
    let mut declarados_admin = 0usize;

    for perfil in archivos {
        let id = perfil
            .file_stem()
            .and_then(|valor| valor.to_str())
            .ok_or_else(|| "Encontré un perfil con nombre no válido.".to_string())?
            .to_string();

        let base = perfil_base_json(raiz, &perfil, &id)?;
        let assigned = jq_texto(
            raiz,
            &host_users,
            &format!(r#"index({}) != null"#, json_texto(&id)),
        )? == "true";

        let local_settings = jq_compacto(
            raiz,
            &settings,
            &format!(r#".[{}] // {{}}"#, json_texto(&id)),
        )?;

        let portable_account = jq_texto(raiz, &base, ".accountName")?;
        let local_account = jq_texto(raiz, &local_settings, ".accountName // empty")?;
        let effective_account = if local_account.is_empty() {
            portable_account
        } else {
            local_account
        };

        let account_exists = comando_exitoso("getent", &["passwd", &effective_account], raiz);

        let declared_groups = if assigned {
            flake_json(
                raiz,
                &format!(
                    "nixosConfigurations.{equipo}.config.users.users.\"{effective_account}\".extraGroups"
                ),
            )
            .unwrap_or_else(|_| "[]".to_string())
        } else {
            "[]".to_string()
        };

        let enriquecido = jq_con_entrada(
            raiz,
            &[
                "-cn".to_string(),
                "--argjson".to_string(),
                "base".to_string(),
                base,
                "--argjson".to_string(),
                "settings".to_string(),
                local_settings,
                "--argjson".to_string(),
                "assigned".to_string(),
                assigned.to_string(),
                "--argjson".to_string(),
                "accountExists".to_string(),
                account_exists.to_string(),
                "--argjson".to_string(),
                "declaredGroups".to_string(),
                declared_groups,
                "--arg".to_string(),
                "effectiveAccount".to_string(),
                effective_account.clone(),
                r#"
                  ($settings.deferredCapabilities // []) as $deferred
                  | ($settings.deferredInputMethods // []) as $deferredInput
                  | ($settings.homeDirectory // ("/home/" + $effectiveAccount)) as $home
                  | ($settings.administrator // false) as $administrator
                  | ($settings.preservedGroups // []) as $preserved
                  | $base
                  | del(.avatarPath)
                  + {
                      assignedToHost: $assigned,
                      effectiveAccountName: $effectiveAccount,
                      homeDirectory: $home,
                      administrator: $administrator,
                      enabledCapabilities:
                        (.capabilities
                         | map(select(. as $c | $deferred | index($c) == null))),
                      deferredCapabilities: $deferred,
                      enabledInputMethods:
                        (.inputMethods
                         | map(select(. as $m | $deferredInput | index($m) == null))),
                      deferredInputMethods: $deferredInput,
                      preservedGroups: $preserved,
                      accountExists: $accountExists,
                      declaredGroups: $declaredGroups
                    }
                "#
                .to_string(),
            ],
            "",
        )?;

        let admin = jq_texto(raiz, &enriquecido, ".administrator")? == "true";
        if assigned && admin {
            declarados_admin += 1;
        }

        perfil_cuenta.push((effective_account, id, assigned));
        perfiles.push(enriquecido);
    }

    let perfiles_json = format!("[{}]", perfiles.join(","));
    let passwd = capturar_opcional("getent", &["passwd"], raiz);
    let minimo = uid_minimo();

    let mut cuentas = Vec::new();
    let mut detectados_admin = 0usize;
    let mut adoptados = 0usize;
    let mut adoptables = 0usize;

    for linea in passwd.lines() {
        let campos: Vec<&str> = linea.split(':').collect();
        if campos.len() < 7 {
            continue;
        }

        let cuenta = campos[0];
        let Ok(uid) = campos[2].parse::<u32>() else {
            continue;
        };

        if uid < minimo || uid >= 65534 {
            continue;
        }

        let real_home = campos[5];
        let shell = campos[6];
        if cuenta_tecnica(real_home, shell) {
            continue;
        }

        let nombre = campos[4]
            .split(',')
            .next()
            .filter(|valor| !valor.is_empty())
            .unwrap_or(cuenta);

        let grupos_texto = capturar_opcional("id", &["-nG", cuenta], raiz);
        let mut grupos: Vec<String> = grupos_texto
            .split_whitespace()
            .map(ToString::to_string)
            .collect();
        grupos.sort();
        grupos.dedup();

        let administrador = grupos.iter().any(|grupo| grupo == "wheel");
        if administrador {
            detectados_admin += 1;
        }

        let perfil = perfil_cuenta
            .iter()
            .find(|(account, _, _)| account == cuenta);

        let (profile_id, estado) = match perfil {
            Some((_, id, true)) => {
                adoptados += 1;
                (json_texto(id), "adopted")
            }
            Some((_, id, false)) => (json_texto(id), "profile-available"),
            None => {
                adoptables += 1;
                ("null".to_string(), "adoptable")
            }
        };

        cuentas.push(format!(
            concat!(
                "{{",
                "\"accountName\":{},",
                "\"displayName\":{},",
                "\"uid\":{},",
                "\"home\":{},",
                "\"shell\":{},",
                "\"administrator\":{},",
                "\"groups\":{},",
                "\"profileId\":{},",
                "\"status\":{}",
                "}}"
            ),
            json_texto(cuenta),
            json_texto(nombre),
            uid,
            json_texto(real_home),
            json_texto(shell),
            administrador,
            json_lista_textos(&grupos),
            profile_id,
            json_texto(estado),
        ));
    }

    let cuentas_json = format!("[{}]", cuentas.join(","));

    Ok(format!(
        concat!(
            "{{",
            "\"schemaVersion\":2,",
            "\"hostId\":{},",
            "\"detectedAt\":{},",
            "\"accounts\":{},",
            "\"profiles\":{},",
            "\"summary\":{{",
            "\"humanAccounts\":{},",
            "\"adoptedAccounts\":{},",
            "\"adoptableAccounts\":{},",
            "\"detectedAdministrators\":{},",
            "\"declaredAdministrators\":{}",
            "}},",
            "\"hostUserIds\":{},",
            "\"policy\":{{",
            "\"mutableUsers\":{},",
            "\"preserveExistingPasswords\":true,",
            "\"repositoryStoresPasswords\":false,",
            "\"newPasswordMethod\":\"system-passwd\",",
            "\"androidAccessModel\":\"systemd-uaccess\",",
            "\"portableProfileSchemaVersion\":3,",
            "\"compatiblePortableProfileSchemaVersions\":[1,2,3],",
            "\"portableFields\":[",
            "\"id\",\"accountName\",\"fullName\",\"language\",\"interfaceLanguage\",",
            "\"inputMethods\",\"capabilities\",\"avatar\"",
            "],",
            "\"hostLocalFields\":[",
            "\"homeDirectory\",\"administrator\",\"deferredCapabilities\",",
            "\"deferredInputMethods\",\"preservedGroups\",\"password\"",
            "]",
            "}}",
            "}}"
        ),
        json_texto(&equipo),
        json_texto(&fecha_iso(raiz)?),
        cuentas_json,
        perfiles_json,
        cuentas.len(),
        adoptados,
        adoptables,
        detectados_admin,
        declarados_admin,
        host_users,
        mutable_users,
    ))
}

fn noctalia_idiomas_json(raiz: &Path) -> Result<String, String> {
    let expresion = r#"
      let
        flake = builtins.getFlake (toString ./.);
      in
        toString flake.inputs.noctalia.outPath
    "#;

    let source = ejecutar_capturando(
        "nix",
        &[
            "eval".to_string(),
            "--raw".to_string(),
            "--impure".to_string(),
            "--expr".to_string(),
            expresion.to_string(),
        ],
        raiz,
    )
    .unwrap_or_default();

    if source.is_empty() {
        return Ok("[]".to_string());
    }

    let carpeta = PathBuf::from(source).join("assets/translations");
    let Ok(entradas) = fs::read_dir(carpeta) else {
        return Ok("[]".to_string());
    };

    let mut idiomas = Vec::new();
    for entrada in entradas.flatten() {
        let ruta = entrada.path();
        if ruta.is_file() && ruta.extension().and_then(|ext| ext.to_str()) == Some("json") {
            if let Some(id) = ruta.file_stem().and_then(|nombre| nombre.to_str()) {
                idiomas.push(id.to_string());
            }
        }
    }

    idiomas.sort();
    Ok(json_lista_textos(&idiomas))
}

fn localectl_campo(raiz: &Path, campo: &str) -> String {
    let salida = capturar_opcional("localectl", &["status", "--no-pager"], raiz);
    let prefijo = format!("{campo}:");

    for linea in salida.lines() {
        let limpia = linea.trim_start();
        if let Some(valor) = limpia.strip_prefix(&prefijo) {
            return valor.trim_start().to_string();
        }
    }

    String::new()
}

fn runtime_lang(raiz: &Path) -> String {
    let salida = capturar_opcional("locale", &[], raiz);

    for linea in salida.lines() {
        if let Some(valor) = linea.strip_prefix("LANG=") {
            return valor.trim_matches('"').to_string();
        }
    }

    String::new()
}

fn runtime_console() -> String {
    let texto = fs::read_to_string("/etc/vconsole.conf").unwrap_or_default();

    for linea in texto.lines() {
        if let Some(valor) = linea.strip_prefix("KEYMAP=") {
            return valor.trim_matches('"').to_string();
        }
    }

    String::new()
}

fn proceso_usuario_activo(raiz: &Path, nombre: &str) -> bool {
    let uid = capturar_opcional("id", &["-u"], raiz);
    if uid.is_empty() {
        return false;
    }

    comando_exitoso("pgrep", &["-u", &uid, "-x", nombre], raiz)
}

fn locale_normalizado_runtime(value: &str) -> String {
    value.to_ascii_lowercase().replace('-', "").replace('.', "")
}

fn localizacion_json_runtime(
    raiz: &Path,
    equipo: &str,
    state: &serde_json::Value,
) -> Result<Option<String>, String> {
    let Some(localization) = state.get("localization") else {
        return Ok(None);
    };
    let Some(declared) = localization.get("declared") else {
        return Ok(None);
    };
    let Some(derived) = localization.get("derived") else {
        return Ok(None);
    };
    let Some(input_method) = localization.get("inputMethod") else {
        return Ok(None);
    };

    let actual_lang = runtime_lang(raiz);
    let actual_timezone = capturar_opcional(
        "timedatectl",
        &["show", "--property=Timezone", "--value"],
        raiz,
    );
    let actual_layout = localectl_campo(raiz, "X11 Layout");
    let actual_variant = localectl_campo(raiz, "X11 Variant");
    let actual_options = localectl_campo(raiz, "X11 Options");
    let actual_console = runtime_console();

    let noctalia = localization
        .get("noctaliaLanguages")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();

    let profiles = state
        .pointer("/people/profiles")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut people = Vec::<serde_json::Value>::new();
    let mut any_enabled_input = false;

    for profile in profiles {
        let enabled = profile
            .get("enabledInputMethods")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();

        if !enabled.is_empty() {
            any_enabled_input = true;
        }

        let language = profile.get("language").and_then(serde_json::Value::as_str);

        let noctalia_available = match language {
            Some(language) => serde_json::Value::Bool(
                noctalia
                    .iter()
                    .any(|value| value.as_str() == Some(language)),
            ),
            None => serde_json::Value::Null,
        };

        people.push(serde_json::json!({
            "id": profile.get("id").cloned().unwrap_or(serde_json::Value::Null),
            "fullName": profile.get("fullName").cloned().unwrap_or(serde_json::Value::Null),
            "language": profile.get("language").cloned().unwrap_or(serde_json::Value::Null),
            "inputMethods": profile.get("inputMethods").cloned().unwrap_or_else(|| serde_json::json!([])),
            "enabledInputMethods": enabled,
            "deferredInputMethods": profile.get("deferredInputMethods").cloned().unwrap_or_else(|| serde_json::json!([])),
            "effectiveAccountName": profile.get("effectiveAccountName").cloned().unwrap_or(serde_json::Value::Null),
            "noctaliaTranslationAvailable": noctalia_available
        }));
    }

    let system_locale = derived
        .get("systemLocale")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    let keyboard = derived
        .get("keyboard")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    let declared_timezone = declared
        .get("timeZone")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");

    let xkb_layout = keyboard
        .get("layout")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    let xkb_variant = keyboard
        .get("variant")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    let xkb_options = keyboard
        .get("options")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    let console_map = keyboard
        .get("console")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");

    let input_type = input_method
        .pointer("/nixos/type")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("none");

    let mut contradictions = Vec::<serde_json::Value>::new();

    if any_enabled_input && input_type != "fcitx5" {
        contradictions.push(serde_json::json!({
            "field": "inputMethod.backend",
            "declared": "fcitx5",
            "actual": input_type,
            "message": "Hay métodos de entrada avanzados efectivos pero el backend candidato no es Fcitx5."
        }));
    }

    if !actual_lang.is_empty()
        && locale_normalizado_runtime(&actual_lang) != locale_normalizado_runtime(system_locale)
    {
        contradictions.push(serde_json::json!({
            "field": "systemLocale",
            "declared": system_locale,
            "actual": actual_lang,
            "message": "LANG activo no coincide con el locale declarado."
        }));
    }

    if !actual_timezone.is_empty() && actual_timezone != declared_timezone {
        contradictions.push(serde_json::json!({
            "field": "timeZone",
            "declared": declared_timezone,
            "actual": actual_timezone,
            "message": "La zona horaria activa no coincide con la declarada."
        }));
    }

    for (field, declared_value, actual_value, message) in [
        (
            "keyboard.layout",
            xkb_layout,
            actual_layout.as_str(),
            "Los layouts XKB activos no coinciden con Korunix.",
        ),
        (
            "keyboard.variant",
            xkb_variant,
            actual_variant.as_str(),
            "Las variantes XKB activas no coinciden con Korunix.",
        ),
        (
            "keyboard.options",
            xkb_options,
            actual_options.as_str(),
            "Las opciones XKB activas no coinciden con Korunix.",
        ),
        (
            "keyboard.console",
            console_map,
            actual_console.as_str(),
            "El mapa de consola activo no coincide con Korunix.",
        ),
    ] {
        if !actual_value.is_empty() && actual_value != declared_value {
            contradictions.push(serde_json::json!({
                "field": field,
                "declared": declared_value,
                "actual": actual_value,
                "message": message
            }));
        }
    }

    let runtime_gtk_im = env::var("GTK_IM_MODULE").unwrap_or_default();
    let runtime_qt_im = env::var("QT_IM_MODULE").unwrap_or_default();
    let runtime_xmodifiers = env::var("XMODIFIERS").unwrap_or_default();
    let runtime_desktop = env::var("XDG_CURRENT_DESKTOP").ok();

    let optional_runtime = |value: String| {
        if value.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::Value::String(value)
        }
    };

    let output = serde_json::json!({
        "schemaVersion": 2,
        "host": equipo,
        "ownership": {
            "portableUserFields": ["language", "inputMethods"],
            "hostLocalFields": [
                "systemLanguage",
                "region",
                "formats",
                "timeZone",
                "keyboard",
                "deferredInputMethods"
            ]
        },
        "declared": declared,
        "derived": derived,
        "runtime": {
            "lang": actual_lang,
            "timeZone": actual_timezone,
            "keyboard": {
                "layout": actual_layout,
                "variant": actual_variant,
                "options": actual_options,
                "console": actual_console
            },
            "desktop": runtime_desktop
        },
        "people": people,
        "noctalia": {
            "supportedLanguages": noctalia
        },
        "inputMethod": {
            "candidate": input_method
                .get("candidate")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({})),
            "nixos": {
                "enabled": input_method
                    .pointer("/nixos/enabled")
                    .cloned()
                    .unwrap_or(serde_json::Value::Bool(false)),
                "type": input_type,
                "package": input_method
                    .pointer("/nixos/package")
                    .cloned()
                    .unwrap_or_else(|| serde_json::Value::String(String::new()))
            },
            "runtime": {
                "gtkImModule": optional_runtime(runtime_gtk_im),
                "qtImModule": optional_runtime(runtime_qt_im),
                "xmodifiers": optional_runtime(runtime_xmodifiers),
                "fcitx5Running": proceso_usuario_activo(raiz, "fcitx5"),
                "ibusRunning": proceso_usuario_activo(raiz, "ibus-daemon")
            }
        },
        "contradictions": contradictions
    });

    Ok(Some(output.to_string()))
}

fn localizacion_json(raiz: &Path) -> Result<String, String> {
    let equipo = resolver_equipo(raiz)?;

    if let Some(runtime) = runtime_state_current(raiz)? {
        if let Some(output) = localizacion_json_runtime(raiz, &equipo, &runtime)? {
            return Ok(output);
        }
    }

    let declared = flake_json(
        raiz,
        &format!("nixosConfigurations.{equipo}.config.korunix.localization"),
    )?;
    let system_locale = flake_raw(
        raiz,
        &format!("nixosConfigurations.{equipo}.config.i18n.defaultLocale"),
    )?;
    let format_locale = flake_raw(
        raiz,
        &format!("nixosConfigurations.{equipo}.config.i18n.extraLocaleSettings.LC_TIME"),
    )?;
    let xkb_layout = flake_raw(
        raiz,
        &format!("nixosConfigurations.{equipo}.config.services.xserver.xkb.layout"),
    )?;
    let xkb_variant = flake_raw(
        raiz,
        &format!("nixosConfigurations.{equipo}.config.services.xserver.xkb.variant"),
    )?;
    let xkb_options = flake_raw(
        raiz,
        &format!("nixosConfigurations.{equipo}.config.services.xserver.xkb.options"),
    )?;
    let console_map = flake_raw(
        raiz,
        &format!("nixosConfigurations.{equipo}.config.console.keyMap"),
    )?;

    let input_method_model = flake_raw(
        raiz,
        &format!(
            "nixosConfigurations.{equipo}.config.environment.etc.\"korunix/input-methods.json\".text"
        ),
    )?;
    let input_method_enabled = flake_json(
        raiz,
        &format!("nixosConfigurations.{equipo}.config.i18n.inputMethod.enable"),
    )?;
    let input_method_type_raw = flake_json(
        raiz,
        &format!("nixosConfigurations.{equipo}.config.i18n.inputMethod.type"),
    )?;
    let input_method_package_raw = flake_json(
        raiz,
        &format!("nixosConfigurations.{equipo}.config.i18n.inputMethod.package"),
    )?;

    let input_method_type = jq_texto(
        raiz,
        &input_method_type_raw,
        r#"if type == "string" then . else "none" end"#,
    )?;
    let input_method_package = jq_texto(
        raiz,
        &input_method_package_raw,
        r#"if type == "string" then . else "" end"#,
    )?;

    let actual_lang = runtime_lang(raiz);
    let actual_timezone = capturar_opcional(
        "timedatectl",
        &["show", "--property=Timezone", "--value"],
        raiz,
    );
    let actual_layout = localectl_campo(raiz, "X11 Layout");
    let actual_variant = localectl_campo(raiz, "X11 Variant");
    let actual_options = localectl_campo(raiz, "X11 Options");
    let actual_console = runtime_console();

    let usuarios = usuarios_json(raiz)?;
    let people = jq_compacto(
        raiz,
        &usuarios,
        r#"[
          .profiles[]
          | {
              id,
              fullName,
              language,
              inputMethods,
              enabledInputMethods,
              deferredInputMethods,
              effectiveAccountName
            }
        ]"#,
    )?;

    let noctalia_languages = noctalia_idiomas_json(raiz)?;

    let runtime_gtk_im = env::var("GTK_IM_MODULE").unwrap_or_default();
    let runtime_qt_im = env::var("QT_IM_MODULE").unwrap_or_default();
    let runtime_xmodifiers = env::var("XMODIFIERS").unwrap_or_default();
    let runtime_desktop = env::var("XDG_CURRENT_DESKTOP").ok();
    let runtime_desktop_json = runtime_desktop
        .as_deref()
        .map(json_texto)
        .unwrap_or_else(|| "null".to_string());

    let filtro = r#"
      def normlocale:
        ascii_downcase
        | gsub("-"; "")
        | gsub("\\."; "");

      def contradiction($field; $declared; $actual; $message):
        {
          field: $field,
          declared: $declared,
          actual: $actual,
          message: $message
        };

      {
        schemaVersion: 2,
        host: $host,

        ownership: {
          portableUserFields: [
            "language",
            "inputMethods"
          ],
          hostLocalFields: [
            "systemLanguage",
            "region",
            "formats",
            "timeZone",
            "keyboard",
            "deferredInputMethods"
          ]
        },

        declared: $declared,

        derived: {
          systemLocale: $systemLocale,
          formatLocale: $formatLocale,
          keyboard: {
            layout: $xkbLayout,
            variant: $xkbVariant,
            options: $xkbOptions,
            console: $consoleMap
          }
        },

        runtime: {
          lang: $runtimeLang,
          timeZone: $runtimeTimezone,
          keyboard: {
            layout: $runtimeLayout,
            variant: $runtimeVariant,
            options: $runtimeOptions,
            console: $runtimeConsole
          },
          desktop: $runtimeDesktop
        },

        people:
          [
            $people[]
            | .language as $language
            | . + {
                noctaliaTranslationAvailable:
                  (
                    if $language == null then
                      null
                    else
                      ($noctaliaLanguages | index($language)) != null
                    end
                  )
              }
          ],

        noctalia: {
          supportedLanguages: $noctaliaLanguages
        },

        inputMethod: {
          candidate: $inputMethodModel,

          nixos: {
            enabled: $inputMethodEnabled,
            type: $inputMethodType,
            package: $inputMethodPackage
          },

          runtime: {
            gtkImModule:
              if $runtimeGtkIm == ""
              then null
              else $runtimeGtkIm
              end,

            qtImModule:
              if $runtimeQtIm == ""
              then null
              else $runtimeQtIm
              end,

            xmodifiers:
              if $runtimeXmodifiers == ""
              then null
              else $runtimeXmodifiers
              end,

            fcitx5Running: $fcitxRunning,
            ibusRunning: $ibusRunning
          }
        },

        contradictions:
          [
            if (
              (
                [
                  $people[]
                  | .enabledInputMethods[]?
                ]
                | length
              ) > 0
              and $inputMethodType != "fcitx5"
            ) then
              contradiction(
                "inputMethod.backend";
                "fcitx5";
                $inputMethodType;
                "Hay métodos de entrada avanzados efectivos pero el backend candidato no es Fcitx5."
              )
            else empty end,

            if (
              ($runtimeLang | length) > 0
              and (($runtimeLang | normlocale) != ($systemLocale | normlocale))
            ) then
              contradiction(
                "systemLocale";
                $systemLocale;
                $runtimeLang;
                "LANG activo no coincide con el locale declarado."
              )
            else empty end,

            if (
              ($runtimeTimezone | length) > 0
              and $runtimeTimezone != $declared.timeZone
            ) then
              contradiction(
                "timeZone";
                $declared.timeZone;
                $runtimeTimezone;
                "La zona horaria activa no coincide con la declarada."
              )
            else empty end,

            if (
              ($runtimeLayout | length) > 0
              and $runtimeLayout != $xkbLayout
            ) then
              contradiction(
                "keyboard.layout";
                $xkbLayout;
                $runtimeLayout;
                "Los layouts XKB activos no coinciden con Korunix."
              )
            else empty end,

            if (
              ($runtimeVariant | length) > 0
              and $runtimeVariant != $xkbVariant
            ) then
              contradiction(
                "keyboard.variant";
                $xkbVariant;
                $runtimeVariant;
                "Las variantes XKB activas no coinciden con Korunix."
              )
            else empty end,

            if (
              ($runtimeOptions | length) > 0
              and $runtimeOptions != $xkbOptions
            ) then
              contradiction(
                "keyboard.options";
                $xkbOptions;
                $runtimeOptions;
                "Las opciones XKB activas no coinciden con Korunix."
              )
            else empty end,

            if (
              ($runtimeConsole | length) > 0
              and $runtimeConsole != $consoleMap
            ) then
              contradiction(
                "keyboard.console";
                $consoleMap;
                $runtimeConsole;
                "El mapa de consola activo no coincide con Korunix."
              )
            else empty end
          ]
      }
    "#;

    jq_con_entrada(
        raiz,
        &[
            "-cn".to_string(),
            "--argjson".to_string(),
            "declared".to_string(),
            declared,
            "--arg".to_string(),
            "host".to_string(),
            equipo,
            "--arg".to_string(),
            "systemLocale".to_string(),
            system_locale,
            "--arg".to_string(),
            "formatLocale".to_string(),
            format_locale,
            "--arg".to_string(),
            "xkbLayout".to_string(),
            xkb_layout,
            "--arg".to_string(),
            "xkbVariant".to_string(),
            xkb_variant,
            "--arg".to_string(),
            "xkbOptions".to_string(),
            xkb_options,
            "--arg".to_string(),
            "consoleMap".to_string(),
            console_map,
            "--arg".to_string(),
            "runtimeLang".to_string(),
            actual_lang,
            "--arg".to_string(),
            "runtimeTimezone".to_string(),
            actual_timezone,
            "--arg".to_string(),
            "runtimeLayout".to_string(),
            actual_layout,
            "--arg".to_string(),
            "runtimeVariant".to_string(),
            actual_variant,
            "--arg".to_string(),
            "runtimeOptions".to_string(),
            actual_options,
            "--arg".to_string(),
            "runtimeConsole".to_string(),
            actual_console,
            "--argjson".to_string(),
            "noctaliaLanguages".to_string(),
            noctalia_languages,
            "--argjson".to_string(),
            "people".to_string(),
            people,
            "--argjson".to_string(),
            "inputMethodModel".to_string(),
            input_method_model,
            "--argjson".to_string(),
            "inputMethodEnabled".to_string(),
            input_method_enabled,
            "--arg".to_string(),
            "inputMethodType".to_string(),
            input_method_type,
            "--arg".to_string(),
            "inputMethodPackage".to_string(),
            input_method_package,
            "--arg".to_string(),
            "runtimeGtkIm".to_string(),
            runtime_gtk_im,
            "--arg".to_string(),
            "runtimeQtIm".to_string(),
            runtime_qt_im,
            "--arg".to_string(),
            "runtimeXmodifiers".to_string(),
            runtime_xmodifiers,
            "--argjson".to_string(),
            "runtimeDesktop".to_string(),
            runtime_desktop_json,
            "--argjson".to_string(),
            "fcitxRunning".to_string(),
            proceso_usuario_activo(raiz, "fcitx5").to_string(),
            "--argjson".to_string(),
            "ibusRunning".to_string(),
            proceso_usuario_activo(raiz, "ibus-daemon").to_string(),
            filtro.to_string(),
        ],
        "",
    )
}

fn ejecutable(ruta: &Path) -> bool {
    fs::metadata(ruta)
        .map(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

fn buscar_programa(nombre: &str) -> Option<PathBuf> {
    let ruta = env::var_os("PATH")?;
    for carpeta in env::split_paths(&ruta) {
        let candidato = carpeta.join(nombre);
        if ejecutable(&candidato) {
            return Some(candidato);
        }
    }
    None
}

fn permiso_backend() -> &'static str {
    if env::var("KORUNIX_TEST_MODE").ok().as_deref() == Some("1") {
        if let Some(ruta) = env::var_os("KORUNIX_PKEXEC") {
            if ejecutable(Path::new(&ruta)) {
                return "polkit";
            }
        }
    }

    if buscar_programa("pkexec").is_some() {
        return "polkit";
    }

    if io::stdin().is_terminal() && io::stdout().is_terminal() && buscar_programa("sudo").is_some()
    {
        return "terminal-sudo-fallback";
    }

    "unavailable"
}

fn permisos_json() -> String {
    let backend = permiso_backend();
    let gui = backend == "polkit";
    let terminal = backend == "terminal-sudo-fallback";

    format!(
        concat!(
            "{{",
            "\"schemaVersion\":1,",
            "\"kind\":\"korunix-privilege-model\",",
            "\"guiRunsAsRoot\":false,",
            "\"backend\":{},",
            "\"guiUsable\":{},",
            "\"terminalSudoFallback\":{},",
            "\"internalPasswordAutomation\":false,",
            "\"pseudoTtyAllowed\":false,",
            "\"policy\":{{",
            "\"systemMutationsUsePolkitWhenAvailable\":true,",
            "\"firmwareUsesFwupdDbusPolkit\":true,",
            "\"nonPrivilegedOperationsStayUnprivileged\":true",
            "}}}}"
        ),
        json_texto(backend),
        gui,
        terminal
    )
}

fn mostrar_permisos() {
    let backend = permiso_backend();
    println!("=== Permisos administrativos ===");
    println!("Korunix no ejecuta la interfaz gráfica como administrador.");

    match backend {
        "polkit" => println!("Cuando haga falta, el sistema puede mostrar una ventana de autorización."),
        "terminal-sudo-fallback" => println!(
            "Polkit no está disponible. Una terminal interactiva puede usar sudo como último recurso."
        ),
        _ => println!("Ahora mismo no hay una forma disponible de pedir permisos administrativos."),
    }

    println!("Korunix nunca automatiza contraseñas ni fabrica terminales falsas.");
}

fn modelo_equipos(raiz: &Path) -> Result<String, String> {
    let ids = equipos_disponibles(raiz)?;
    let mut partes = Vec::new();

    for id in ids {
        let archivo = raiz
            .join("configuracion")
            .join("equipos")
            .join(format!("{id}.nix"));
        let datos = nix_archivo_json(raiz, &archivo)?;
        partes.push(format!("{}:{}", json_texto(&id), datos));
    }

    Ok(format!("{{{}}}", partes.join(",")))
}

fn ejecutar_modelo(raiz: &Path, argumentos: &[OsString]) -> Result<ExitCode, String> {
    let texto: Vec<String> = argumentos
        .iter()
        .map(|valor| valor.to_string_lossy().into_owned())
        .collect();

    let salida = match texto.as_slice() {
        [] => {
            let canales = nix_archivo_json(raiz, &raiz.join("sistema/canales.nix"))?;
            let predeterminados =
                nix_archivo_json(raiz, &raiz.join("sistema/predeterminados.nix"))?;
            let equipos = modelo_equipos(raiz)?;
            format!(
                "{{\"esquema\":1,\"canales\":{canales},\"predeterminados\":{predeterminados},\"equipos\":{equipos}}}"
            )
        }
        [seccion] if seccion == "canales" => {
            nix_archivo_json(raiz, &raiz.join("sistema/canales.nix"))?
        }
        [seccion] if seccion == "predeterminados" => {
            nix_archivo_json(raiz, &raiz.join("sistema/predeterminados.nix"))?
        }
        [seccion] if seccion == "equipos" => modelo_equipos(raiz)?,
        [seccion, equipo] if seccion == "equipos" && id_valido(equipo) => {
            let archivo = raiz
                .join("configuracion")
                .join("equipos")
                .join(format!("{equipo}.nix"));
            if !archivo.is_file() {
                return Err(format!("No existe configuracion/equipos/{equipo}.nix."));
            }
            nix_archivo_json(raiz, &archivo)?
        }
        _ => return Err("Uso: korunix modelo [canales|predeterminados|equipos [id]]".to_string()),
    };

    println!("{salida}");
    Ok(ExitCode::SUCCESS)
}

fn ejecutar() -> Result<ExitCode, String> {
    let raiz = raiz_repositorio()?;
    let mut argumentos: Vec<OsString> = env::args_os().skip(1).collect();

    if argumentos.is_empty() {
        print!("{AYUDA}");
        return Ok(ExitCode::SUCCESS);
    }

    let comando = argumentos.remove(0);
    let comando_texto = comando.to_string_lossy();

    match comando_texto.as_ref() {
        "-h" | "--help" | "help" | "ayuda" => {
            print!("{AYUDA}");
            Ok(ExitCode::SUCCESS)
        }
        "modelo" => ejecutar_modelo(&raiz, &argumentos),
        "channel" => ejecutar_canal(&raiz, &argumentos),
        "hardware" if argumentos.len() == 1 && argumentos[0] == "--json" => {
            println!("{}", hardware_json(&raiz)?);
            Ok(ExitCode::SUCCESS)
        }
        "localization" if argumentos.len() == 1 && argumentos[0] == "--json" => {
            println!("{}", localizacion_json(&raiz)?);
            Ok(ExitCode::SUCCESS)
        }
        "users" if argumentos.len() == 1 && argumentos[0] == "--json" => {
            println!("{}", usuarios_json(&raiz)?);
            Ok(ExitCode::SUCCESS)
        }
        "privileges" if argumentos.is_empty() => {
            mostrar_permisos();
            Ok(ExitCode::SUCCESS)
        }
        "privileges" if argumentos.len() == 1 && argumentos[0] == "--json" => {
            println!("{}", permisos_json());
            Ok(ExitCode::SUCCESS)
        }
        "privileges" => Err("Uso: korunix privileges [--json].".to_string()),
        _ => operaciones::ejecutar_operacion(&raiz, comando_texto.as_ref(), &argumentos),
    }
}

fn main() -> ExitCode {
    match ejecutar() {
        Ok(codigo) => codigo,
        Err(error) => {
            eprintln!("ERROR: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::{host_id_marker_value, id_valido, json_texto, texto_con_canal};

    #[test]
    fn acepta_identificadores_humanos_simples() {
        assert!(id_valido("korunix"));
        assert!(id_valido("portatil-1"));
        assert!(id_valido("equipo_casa"));
    }

    #[test]
    fn rechaza_rutas_disfrazadas_de_identificador() {
        assert!(!id_valido("../otro"));
        assert!(!id_valido("equipo.casa"));
        assert!(!id_valido(""));
    }

    #[test]
    fn acepta_host_id_persistido_con_salto_final() {
        assert_eq!(
            host_id_marker_value("portatil-1\n").expect("hostId válido"),
            "portatil-1"
        );
        assert!(host_id_marker_value("../otro").is_err());
    }

    #[test]
    fn escapa_texto_para_json() {
        assert_eq!(json_texto("hola\n\"mundo\""), "\"hola\\n\\\"mundo\\\"\"");
    }

    #[test]
    fn cambia_exactamente_un_canal() {
        let original = "{\n  channel = \"unstable\";\n  stateVersion = \"26.05\";\n}\n";
        let nuevo = texto_con_canal(original, "stable").expect("el canal debe cambiar");

        assert!(nuevo.contains("channel = \"stable\";"));
        assert!(nuevo.contains("stateVersion = \"26.05\";"));
    }

    #[test]
    fn rechaza_dos_declaraciones_de_canal() {
        let ambiguo = "channel = \"stable\";\nchannel = \"unstable\";\n";
        assert!(texto_con_canal(ambiguo, "stable").is_err());
    }
}

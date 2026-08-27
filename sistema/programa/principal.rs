//! Motor operativo de Korunix.
//!
//! Este archivo es parte interna del sistema.
//!
//! Una persona no necesita editarlo para cambiar su computadora. Las decisiones
//! humanas viven en los archivos de configuración. Este programa toma esas
//! decisiones y realiza las consultas u operaciones necesarias.
//!
//! Durante D.2 todavía existen operaciones antiguas en `scripts/korunix`.
//! Cuando una operación aún no vive aquí, Korunix la entrega temporalmente a ese
//! archivo. La meta de D.2 es ir quitando esas entregas hasta que Rust sea el
//! único motor.

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
  korunix modelo
  korunix modelo canales
  korunix modelo predeterminados
  korunix modelo equipos
  korunix modelo equipos <id>
  korunix channel
  korunix channel --json
  korunix channel stable --plan
  korunix channel unstable --plan
  korunix channel stable --plan --json
  korunix channel unstable --plan --json
  korunix channel stable
  korunix channel unstable
  korunix channel stable --yes
  korunix channel unstable --yes
  korunix privileges --json
  korunix <operación todavía en transición>

"modelo" lee las fuentes declarativas de Nix.

"channel" administra el canal completo:
- consultar no cambia nada;
- --plan evalúa una copia temporal y no toca tu configuración;
- cambiar el canal modifica una sola declaración;
- --yes indica que una interfaz superior ya confirmó el cambio;
- nunca aplica una generación y nunca cambia system.stateVersion.

"privileges --json" explica cómo Korunix podría pedir permisos sin pedirlos.
Las operaciones todavía no migradas se entregan temporalmente a scripts/korunix.
"#;

fn es_raiz_korunix(ruta: &Path) -> bool {
    ruta.join("flake.nix").is_file()
        && ruta.join("sistema/canales.nix").is_file()
        && ruta.join("sistema/predeterminados.nix").is_file()
        && ruta.join("scripts/korunix").is_file()
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
            "Hay varias computadoras configuradas. Define KORUNIX_HOST para elegir una."
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

fn canal_json(raiz: &Path) -> Result<String, String> {
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

fn ejecutar_compatibilidad(raiz: &Path, argumentos: &[OsString]) -> Result<ExitCode, String> {
    let heredado = raiz.join("scripts/korunix");

    let estado = Command::new(&heredado)
        .args(argumentos)
        .current_dir(raiz)
        .env("KORUNIX_MOTOR", "rust")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|error| {
            format!(
                "No pude entregar la operación temporal a {}: {error}",
                heredado.display()
            )
        })?;

    Ok(ExitCode::from(
        estado.code().unwrap_or(1).clamp(0, u8::MAX as i32) as u8,
    ))
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
        "privileges" if argumentos.is_empty() => {
            mostrar_permisos();
            Ok(ExitCode::SUCCESS)
        }
        "privileges" if argumentos.len() == 1 && argumentos[0] == "--json" => {
            println!("{}", permisos_json());
            Ok(ExitCode::SUCCESS)
        }
        "privileges" => Err("Uso: korunix privileges [--json].".to_string()),
        _ => {
            let mut delegados = vec![comando];
            delegados.extend(argumentos);
            ejecutar_compatibilidad(&raiz, &delegados)
        }
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
    use super::{id_valido, json_texto, texto_con_canal};

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

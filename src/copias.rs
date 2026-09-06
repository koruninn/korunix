use crate::configuracion;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::ffi::{c_char, CString};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::PermissionsExt;
use std::path::{Component, Path, PathBuf};
use std::process::{self, Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

const TIPO_COPIA: &str = "configuracion-korunix";
const TIPO_PROTECCION: &str = "proteccion-antes-de-restaurar";
const TIPO_RESTAURACION: &str = "restauracion-configuracion-korunix";
const VERSION_FORMATO: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
struct ArchivoTexto {
    contenido: String,
    sha256: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Recurso {
    ruta: String,
    datos: Vec<u8>,
    sha256: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CopiaKorunix {
    version_formato: u32,
    tipo: String,
    creada_unix: u64,
    equipo: String,
    version_korunix: String,
    configuracion: ArchivoTexto,
    flake_lock: ArchivoTexto,
    recursos: Vec<Recurso>,
    incluye_hardware: bool,
    incluye_credenciales: bool,
    incluye_historial: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct EntradaHistorial {
    version: u32,
    tipo: String,
    momento_unix: u64,
    resumen: String,
    archivo: String,
    sha256: String,
}

#[derive(Clone, Debug)]
pub struct ResumenCopia {
    pub equipo: String,
    pub creada_unix: u64,
    pub recursos: usize,
    pub tamano: u64,
    pub sha256: String,
}

#[derive(Clone, Debug)]
pub struct HistorialVisible {
    pub resumen: String,
    pub archivo: String,
    pub cuando: String,
    pub estado: String,
}

#[derive(Clone, Debug)]
pub struct PlanRestauracion {
    pub equipo_actual: String,
    pub equipo_copia: String,
    pub canal_actual: String,
    pub canal_copia: String,
    pub escritorio_actual: String,
    pub escritorio_copia: String,
    pub personas_actual: usize,
    pub personas_copia: usize,
    pub aplicaciones_actual: usize,
    pub aplicaciones_copia: usize,
    pub configuracion_cambia: bool,
    pub flake_lock_cambia: bool,
    pub recursos_cambian: usize,
    pub recursos_total: usize,
    pub hay_cambios: bool,
    pub sha256: String,
}

#[derive(Clone, Debug)]
pub struct ResultadoRestauracion {
    pub plan: PlanRestauracion,
    pub proteccion: Option<PathBuf>,
}

#[derive(Clone)]
struct CambioArchivo {
    ruta: PathBuf,
    nuevo: Vec<u8>,
    anterior: Option<Vec<u8>>,
    modo_anterior: Option<u32>,
}

fn ahora() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn cuando_humano(momento: u64) -> String {
    let segundos = ahora().saturating_sub(momento);

    if segundos < 60 {
        "hace unos segundos".to_string()
    } else if segundos < 3_600 {
        format!("hace {} min", segundos / 60)
    } else if segundos < 86_400 {
        format!("hace {} h", segundos / 3_600)
    } else {
        format!("hace {} días", segundos / 86_400)
    }
}

pub fn tamano_humano(bytes: u64) -> String {
    let bytes = bytes as f64;

    if bytes >= 1_000_000_000.0 {
        format!("{:.1} GB", bytes / 1_000_000_000.0)
    } else if bytes >= 1_000_000.0 {
        format!("{:.1} MB", bytes / 1_000_000.0)
    } else if bytes >= 1_000.0 {
        format!("{:.1} kB", bytes / 1_000.0)
    } else {
        format!("{bytes:.0} B")
    }
}

fn programa_sha256() -> PathBuf {
    if let Some(programa) = env::var_os("KORUNIX_SHA256SUM_BIN") {
        return PathBuf::from(programa);
    }

    let sistema = Path::new("/run/current-system/sw/bin/sha256sum");
    if sistema.is_file() {
        sistema.to_path_buf()
    } else {
        PathBuf::from("sha256sum")
    }
}

fn sha256_bytes(datos: &[u8]) -> Result<String, String> {
    let mut hijo = Command::new(programa_sha256())
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            format!("No pude calcular la integridad de la copia.\nDetalle: {error}")
        })?;

    hijo.stdin
        .take()
        .ok_or_else(|| "No pude preparar el cálculo de integridad.".to_string())?
        .write_all(datos)
        .map_err(|error| {
            format!("No pude calcular la integridad de la copia.\nDetalle: {error}")
        })?;

    let salida = hijo
        .wait_with_output()
        .map_err(|error| format!("No pude terminar el cálculo de integridad.\nDetalle: {error}"))?;

    if !salida.status.success() {
        let detalle = String::from_utf8_lossy(&salida.stderr).trim().to_string();
        return Err(if detalle.is_empty() {
            "No pude calcular la integridad de la copia.".to_string()
        } else {
            format!("No pude calcular la integridad de la copia.\nDetalle: {detalle}")
        });
    }

    let texto = String::from_utf8_lossy(&salida.stdout);
    let hash = texto.split_whitespace().next().unwrap_or_default();

    if hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("El cálculo de integridad devolvió un resultado inesperado.".to_string());
    }

    Ok(hash.to_ascii_lowercase())
}

fn ruta_recurso_valida(ruta: &str) -> bool {
    let ruta = Path::new(ruta);
    let prohibidos = [
        "configuracion.toml",
        "flake.lock",
        "flake.nix",
        "hardware.nix",
        "sistema.nix",
        "RUTA.md",
    ];

    if ruta.as_os_str().is_empty() || ruta.is_absolute() {
        return false;
    }

    if ruta.components().any(|parte| {
        matches!(
            parte,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        ) || parte.as_os_str() == ".git"
    }) {
        return false;
    }

    if ruta.components().count() == 1 {
        if let Some(nombre) = ruta.file_name().and_then(|valor| valor.to_str()) {
            if prohibidos.contains(&nombre) {
                return false;
            }
        }
    }

    true
}

fn validar_flake_lock(texto: &str) -> Result<(), String> {
    let valor: serde_json::Value = serde_json::from_str(texto)
        .map_err(|error| format!("flake.lock no es JSON válido.\nDetalle: {error}"))?;

    let objeto = valor
        .as_object()
        .ok_or_else(|| "flake.lock no tiene la forma esperada.".to_string())?;

    if !objeto
        .get("version")
        .is_some_and(serde_json::Value::is_number)
        || !objeto
            .get("nodes")
            .is_some_and(serde_json::Value::is_object)
        || !objeto.get("root").is_some_and(serde_json::Value::is_string)
    {
        return Err(
            "flake.lock no contiene version, nodes y root con la forma esperada.".to_string(),
        );
    }

    Ok(())
}

fn recursos_humanos(
    raiz: &Path,
    configuracion: &configuracion::Configuracion,
) -> Result<Vec<Recurso>, String> {
    let mut vistos = HashSet::new();
    let mut recursos = Vec::new();

    for persona in &configuracion.personas {
        let Some(avatar) = persona.avatar.as_deref() else {
            continue;
        };

        if !vistos.insert(avatar.to_string()) {
            continue;
        }

        if !ruta_recurso_valida(avatar) {
            return Err(format!(
                "El avatar «{avatar}» no tiene una ruta portable que pueda guardar."
            ));
        }

        let ruta = raiz.join(avatar);
        let datos = fs::read(&ruta).map_err(|error| {
            format!("No pude incluir el avatar «{avatar}» en la copia.\nDetalle: {error}")
        })?;

        if !ruta.is_file() {
            return Err(format!("«{avatar}» no es un archivo normal."));
        }

        recursos.push(Recurso {
            ruta: avatar.to_string(),
            sha256: sha256_bytes(&datos)?,
            datos,
        });
    }

    Ok(recursos)
}

fn estado_raiz() -> Result<PathBuf, String> {
    if let Some(ruta) = env::var_os("XDG_STATE_HOME") {
        return Ok(PathBuf::from(ruta).join("korunix"));
    }

    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "No encontré HOME para guardar el Historial.".to_string())?;

    Ok(home.join(".local/state/korunix"))
}

fn historial_ruta() -> Result<PathBuf, String> {
    Ok(estado_raiz()?.join("historial.jsonl"))
}

fn temporal_para(destino: &Path) -> Result<PathBuf, String> {
    let carpeta = destino
        .parent()
        .ok_or_else(|| "La copia necesita una carpeta de destino.".to_string())?;
    let nombre = destino
        .file_name()
        .and_then(|valor| valor.to_str())
        .unwrap_or("copia-korunix");
    let momento = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    Ok(carpeta.join(format!(
        ".{nombre}.korunix-parcial-{}-{momento}",
        process::id()
    )))
}

#[cfg(target_os = "linux")]
fn publicar_sin_sobrescribir(temporal: &Path, destino: &Path) -> Result<(), String> {
    const AT_FDCWD: i32 = -100;
    const RENAME_NOREPLACE: u32 = 1;

    extern "C" {
        fn renameat2(
            olddirfd: i32,
            oldpath: *const c_char,
            newdirfd: i32,
            newpath: *const c_char,
            flags: u32,
        ) -> i32;
    }

    let temporal_c = CString::new(temporal.as_os_str().as_bytes())
        .map_err(|_| "La ruta temporal contiene un carácter no válido.".to_string())?;
    let destino_c = CString::new(destino.as_os_str().as_bytes())
        .map_err(|_| "La ruta final contiene un carácter no válido.".to_string())?;

    let resultado = unsafe {
        renameat2(
            AT_FDCWD,
            temporal_c.as_ptr(),
            AT_FDCWD,
            destino_c.as_ptr(),
            RENAME_NOREPLACE,
        )
    };

    if resultado == 0 {
        Ok(())
    } else if destino.exists() {
        Err(format!(
            "Ya existe «{}». Korunix no va a sobrescribir esa copia.",
            destino.display()
        ))
    } else {
        Err(format!(
            "No pude publicar la copia terminada.\nDetalle: {}",
            std::io::Error::last_os_error()
        ))
    }
}

#[cfg(not(target_os = "linux"))]
fn publicar_sin_sobrescribir(_temporal: &Path, _destino: &Path) -> Result<(), String> {
    Err("La publicación segura de copias de Korunix requiere Linux.".to_string())
}

fn escribir_copia(destino: &Path, datos: &[u8]) -> Result<(), String> {
    if destino.exists() {
        return Err(format!(
            "Ya existe «{}». Korunix no va a sobrescribir esa copia.",
            destino.display()
        ));
    }

    let carpeta = destino
        .parent()
        .ok_or_else(|| "La copia necesita una carpeta de destino.".to_string())?;
    fs::create_dir_all(carpeta).map_err(|error| {
        format!(
            "No pude preparar la carpeta «{}».\nDetalle: {error}",
            carpeta.display()
        )
    })?;

    let temporal = temporal_para(destino)?;
    let resultado = (|| {
        let mut archivo = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporal)
            .map_err(|error| format!("No pude preparar la copia temporal.\nDetalle: {error}"))?;

        archivo
            .write_all(datos)
            .map_err(|error| format!("No pude escribir la copia.\nDetalle: {error}"))?;
        archivo.sync_all().map_err(|error| {
            format!("No pude confirmar la copia en el disco.\nDetalle: {error}")
        })?;

        let mut permisos = archivo
            .metadata()
            .map_err(|error| format!("No pude revisar la copia temporal.\nDetalle: {error}"))?
            .permissions();
        permisos.set_mode(0o600);
        fs::set_permissions(&temporal, permisos).map_err(|error| {
            format!("No pude proteger los permisos de la copia.\nDetalle: {error}")
        })?;
        drop(archivo);

        publicar_sin_sobrescribir(&temporal, destino)?;

        if let Ok(carpeta_abierta) = File::open(carpeta) {
            let _ = carpeta_abierta.sync_all();
        }

        Ok(())
    })();

    if resultado.is_err() {
        let _ = fs::remove_file(&temporal);
    }

    resultado
}

fn guardar_entrada_historial(historial: &Path, entrada: &EntradaHistorial) -> Result<(), String> {
    let carpeta = historial
        .parent()
        .ok_or_else(|| "Historial no tiene carpeta padre.".to_string())?;
    fs::create_dir_all(carpeta)
        .map_err(|error| format!("No pude preparar Historial.\nDetalle: {error}"))?;

    let linea = serde_json::to_string(entrada)
        .map_err(|error| format!("No pude preparar Historial.\nDetalle: {error}"))?;

    let mut archivo = OpenOptions::new()
        .create(true)
        .append(true)
        .open(historial)
        .map_err(|error| format!("No pude abrir Historial.\nDetalle: {error}"))?;
    writeln!(archivo, "{linea}")
        .map_err(|error| format!("No pude guardar Historial.\nDetalle: {error}"))?;
    archivo
        .sync_all()
        .map_err(|error| format!("No pude confirmar Historial en el disco.\nDetalle: {error}"))?;

    let mut permisos = archivo
        .metadata()
        .map_err(|error| format!("No pude revisar Historial.\nDetalle: {error}"))?
        .permissions();
    permisos.set_mode(0o600);
    fs::set_permissions(historial, permisos)
        .map_err(|error| format!("No pude proteger Historial.\nDetalle: {error}"))?;

    Ok(())
}

fn registrar_historial_en(
    historial: &Path,
    ruta: &Path,
    resumen: &ResumenCopia,
) -> Result<(), String> {
    guardar_entrada_historial(
        historial,
        &EntradaHistorial {
            version: 1,
            tipo: TIPO_COPIA.to_string(),
            momento_unix: resumen.creada_unix,
            resumen: format!("Configuración de Korunix · {}", resumen.equipo),
            archivo: ruta.display().to_string(),
            sha256: resumen.sha256.clone(),
        },
    )
}

fn registrar_proteccion_en(
    historial: &Path,
    ruta: &Path,
    resumen: &ResumenCopia,
) -> Result<(), String> {
    guardar_entrada_historial(
        historial,
        &EntradaHistorial {
            version: 1,
            tipo: TIPO_PROTECCION.to_string(),
            momento_unix: resumen.creada_unix,
            resumen: format!("Protección antes de restaurar · {}", resumen.equipo),
            archivo: ruta.display().to_string(),
            sha256: resumen.sha256.clone(),
        },
    )
}

fn registrar_restauracion_en(
    historial: &Path,
    ruta: &Path,
    plan: &PlanRestauracion,
) -> Result<(), String> {
    guardar_entrada_historial(
        historial,
        &EntradaHistorial {
            version: 1,
            tipo: TIPO_RESTAURACION.to_string(),
            momento_unix: ahora(),
            resumen: format!("Restauración de Korunix · {}", plan.equipo_copia),
            archivo: ruta.display().to_string(),
            sha256: plan.sha256.clone(),
        },
    )
}

fn crear_documento(raiz: &Path, destino: &Path) -> Result<ResumenCopia, String> {
    let ruta_configuracion = raiz.join("configuracion.toml");
    let texto_configuracion = fs::read_to_string(&ruta_configuracion).map_err(|error| {
        format!("No pude leer configuracion.toml para crear la copia.\nDetalle: {error}")
    })?;
    let configuracion = configuracion::leer_texto(
        &texto_configuracion,
        "configuracion.toml dentro de la copia",
    )?;

    let texto_lock = fs::read_to_string(raiz.join("flake.lock"))
        .map_err(|error| format!("No pude leer flake.lock.\nDetalle: {error}"))?;
    validar_flake_lock(&texto_lock)?;

    let recursos = recursos_humanos(raiz, &configuracion)?;
    let creada_unix = ahora();

    let copia = CopiaKorunix {
        version_formato: VERSION_FORMATO,
        tipo: TIPO_COPIA.to_string(),
        creada_unix,
        equipo: configuracion.nombre.clone(),
        version_korunix: env!("CARGO_PKG_VERSION").to_string(),
        configuracion: ArchivoTexto {
            sha256: sha256_bytes(texto_configuracion.as_bytes())?,
            contenido: texto_configuracion,
        },
        flake_lock: ArchivoTexto {
            sha256: sha256_bytes(texto_lock.as_bytes())?,
            contenido: texto_lock,
        },
        recursos,
        incluye_hardware: false,
        incluye_credenciales: false,
        incluye_historial: false,
    };

    let datos = serde_json::to_vec_pretty(&copia)
        .map_err(|error| format!("No pude preparar la copia.\nDetalle: {error}"))?;
    let sha256 = sha256_bytes(&datos)?;

    escribir_copia(destino, &datos)?;

    let resumen = ResumenCopia {
        equipo: copia.equipo,
        creada_unix,
        recursos: copia.recursos.len(),
        tamano: datos.len() as u64,
        sha256,
    };

    Ok(resumen)
}

fn crear_con_historial(
    raiz: &Path,
    destino: &Path,
    historial: &Path,
) -> Result<ResumenCopia, String> {
    let resumen = crear_documento(raiz, destino)?;

    if let Err(error) = registrar_historial_en(historial, destino, &resumen) {
        let _ = fs::remove_file(destino);
        return Err(format!(
            "No pude completar la copia porque no pude registrarla en Historial.\nDetalle: {error}"
        ));
    }

    Ok(resumen)
}

pub fn crear(raiz: &Path, destino: &Path) -> Result<ResumenCopia, String> {
    let historial = historial_ruta()?;
    crear_con_historial(raiz, destino, &historial)
}

fn leer_y_validar(ruta: &Path) -> Result<(CopiaKorunix, Vec<u8>), String> {
    let datos = fs::read(ruta)
        .map_err(|error| format!("No pude leer «{}».\nDetalle: {error}", ruta.display()))?;
    let copia: CopiaKorunix = serde_json::from_slice(&datos)
        .map_err(|error| format!("No entiendo esta copia de Korunix.\nDetalle: {error}"))?;

    if copia.version_formato != VERSION_FORMATO || copia.tipo != TIPO_COPIA {
        return Err("Esta copia no usa un formato de Korunix que conozca.".to_string());
    }

    if copia.incluye_hardware || copia.incluye_credenciales || copia.incluye_historial {
        return Err(
            "La copia declara contenido que este formato portable no debe guardar.".to_string(),
        );
    }

    if sha256_bytes(copia.configuracion.contenido.as_bytes())? != copia.configuracion.sha256 {
        return Err("configuracion.toml no coincide con su huella de integridad.".to_string());
    }

    if sha256_bytes(copia.flake_lock.contenido.as_bytes())? != copia.flake_lock.sha256 {
        return Err("flake.lock no coincide con su huella de integridad.".to_string());
    }

    let configuracion = configuracion::leer_texto(
        &copia.configuracion.contenido,
        "configuracion.toml guardada en la copia",
    )?;
    validar_flake_lock(&copia.flake_lock.contenido)?;

    if copia.equipo != configuracion.nombre {
        return Err("El nombre del equipo no coincide con configuracion.toml.".to_string());
    }

    let avatares: HashSet<String> = configuracion
        .personas
        .iter()
        .filter_map(|persona| persona.avatar.clone())
        .collect();
    let mut recursos_vistos = HashSet::new();

    for recurso in &copia.recursos {
        if !ruta_recurso_valida(&recurso.ruta) {
            return Err(format!(
                "La copia contiene una ruta de recurso no permitida: «{}».",
                recurso.ruta
            ));
        }

        if !avatares.contains(&recurso.ruta) {
            return Err(format!(
                "La copia contiene «{}», pero configuracion.toml no lo usa como avatar.",
                recurso.ruta
            ));
        }

        if !recursos_vistos.insert(recurso.ruta.clone()) {
            return Err(format!(
                "La copia contiene más de una vez el recurso «{}».",
                recurso.ruta
            ));
        }

        if sha256_bytes(&recurso.datos)? != recurso.sha256 {
            return Err(format!(
                "El recurso «{}» no coincide con su huella de integridad.",
                recurso.ruta
            ));
        }
    }

    for avatar in avatares {
        if !recursos_vistos.contains(&avatar) {
            return Err(format!(
                "La copia no contiene el avatar «{avatar}» que configuracion.toml necesita."
            ));
        }
    }

    Ok((copia, datos))
}

pub fn inspeccionar(ruta: &Path) -> Result<ResumenCopia, String> {
    let (copia, datos) = leer_y_validar(ruta)?;

    Ok(ResumenCopia {
        equipo: copia.equipo,
        creada_unix: copia.creada_unix,
        recursos: copia.recursos.len(),
        tamano: datos.len() as u64,
        sha256: sha256_bytes(&datos)?,
    })
}

fn comprobar_destino_local(raiz: &Path, destino: &Path) -> Result<(), String> {
    if !destino.starts_with(raiz) {
        return Err(format!(
            "Korunix rechazó una ruta fuera de su carpeta: {}",
            destino.display()
        ));
    }

    let relativo = destino
        .strip_prefix(raiz)
        .map_err(|_| "No pude comprobar la ruta que se restauraría.".to_string())?;

    let mut actual = raiz.to_path_buf();

    for parte in relativo.components() {
        actual.push(parte.as_os_str());

        match fs::symlink_metadata(&actual) {
            Ok(datos) if datos.file_type().is_symlink() => {
                return Err(format!(
                    "No voy a restaurar «{}» porque la ruta atraviesa un enlace simbólico.",
                    destino.display()
                ));
            }
            Ok(datos) if actual != destino && !datos.is_dir() => {
                return Err(format!(
                    "No voy a restaurar «{}» porque una carpeta intermedia no es un directorio.",
                    destino.display()
                ));
            }
            Ok(datos) if actual == destino && !datos.is_file() => {
                return Err(format!(
                    "No voy a reemplazar «{}» porque no es un archivo normal.",
                    destino.display()
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "No pude comprobar «{}» antes de restaurar.\nDetalle: {error}",
                    actual.display()
                ));
            }
        }
    }

    Ok(())
}

fn preparar_cambio(
    raiz: &Path,
    ruta: PathBuf,
    nuevo: Vec<u8>,
) -> Result<Option<CambioArchivo>, String> {
    comprobar_destino_local(raiz, &ruta)?;

    let anterior = match fs::read(&ruta) {
        Ok(datos) => Some(datos),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            return Err(format!(
                "No pude proteger «{}» antes de restaurar.\nDetalle: {error}",
                ruta.display()
            ));
        }
    };

    if anterior.as_deref() == Some(nuevo.as_slice()) {
        return Ok(None);
    }

    let modo_anterior = match fs::symlink_metadata(&ruta) {
        Ok(datos) => Some(datos.permissions().mode()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            return Err(format!(
                "No pude leer los permisos de «{}».\nDetalle: {error}",
                ruta.display()
            ));
        }
    };

    Ok(Some(CambioArchivo {
        ruta,
        nuevo,
        anterior,
        modo_anterior,
    }))
}

fn crear_carpetas_necesarias(
    raiz: &Path,
    destino: &Path,
    creadas: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let mut faltan = Vec::new();
    let mut actual = destino
        .parent()
        .ok_or_else(|| "El archivo que se restaurará no tiene carpeta padre.".to_string())?;

    while actual != raiz {
        if !actual.starts_with(raiz) {
            return Err("La restauración intentó salir de la carpeta de Korunix.".to_string());
        }

        if !actual.exists() {
            faltan.push(actual.to_path_buf());
        }

        actual = actual
            .parent()
            .ok_or_else(|| "No pude resolver las carpetas de restauración.".to_string())?;
    }

    for carpeta in faltan.into_iter().rev() {
        fs::create_dir(&carpeta).map_err(|error| {
            format!(
                "No pude preparar la carpeta «{}».\nDetalle: {error}",
                carpeta.display()
            )
        })?;
        creadas.push(carpeta);
    }

    Ok(())
}

fn escribir_reemplazo_atomico(ruta: &Path, datos: &[u8], modo: u32) -> Result<(), String> {
    let carpeta = ruta
        .parent()
        .ok_or_else(|| "El archivo que se restaurará no tiene carpeta padre.".to_string())?;
    let nombre = ruta
        .file_name()
        .and_then(|valor| valor.to_str())
        .unwrap_or("archivo");
    let momento = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temporal = carpeta.join(format!(
        ".{nombre}.korunix-restaurando-{}-{momento}",
        process::id()
    ));

    let resultado = (|| {
        let mut archivo = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporal)
            .map_err(|error| {
                format!(
                    "No pude preparar «{}» para restaurarlo.\nDetalle: {error}",
                    ruta.display()
                )
            })?;

        archivo.write_all(datos).map_err(|error| {
            format!(
                "No pude escribir «{}» durante la restauración.\nDetalle: {error}",
                ruta.display()
            )
        })?;
        archivo.sync_all().map_err(|error| {
            format!(
                "No pude confirmar «{}» en el disco.\nDetalle: {error}",
                ruta.display()
            )
        })?;

        let mut permisos = archivo
            .metadata()
            .map_err(|error| error.to_string())?
            .permissions();
        permisos.set_mode(modo);
        fs::set_permissions(&temporal, permisos).map_err(|error| {
            format!(
                "No pude conservar los permisos de «{}».\nDetalle: {error}",
                ruta.display()
            )
        })?;
        drop(archivo);

        fs::rename(&temporal, ruta).map_err(|error| {
            format!(
                "No pude publicar «{}» durante la restauración.\nDetalle: {error}",
                ruta.display()
            )
        })?;

        if let Ok(carpeta_abierta) = File::open(carpeta) {
            let _ = carpeta_abierta.sync_all();
        }

        Ok(())
    })();

    if resultado.is_err() {
        let _ = fs::remove_file(&temporal);
    }

    resultado
}

fn devolver_cambios(cambios: &[CambioArchivo], carpetas_creadas: &[PathBuf]) -> Result<(), String> {
    let mut errores = Vec::new();

    for cambio in cambios.iter().rev() {
        let resultado = match &cambio.anterior {
            Some(anterior) => escribir_reemplazo_atomico(
                &cambio.ruta,
                anterior,
                cambio.modo_anterior.unwrap_or(0o600),
            ),
            None => match fs::remove_file(&cambio.ruta) {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(error) => Err(format!(
                    "No pude retirar «{}» al deshacer la restauración.\nDetalle: {error}",
                    cambio.ruta.display()
                )),
            },
        };

        if let Err(error) = resultado {
            errores.push(error);
        }
    }

    for carpeta in carpetas_creadas.iter().rev() {
        match fs::remove_dir(carpeta) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) if error.kind() == std::io::ErrorKind::DirectoryNotEmpty => {}
            Err(error) => errores.push(format!(
                "No pude retirar la carpeta temporal «{}».\nDetalle: {error}",
                carpeta.display()
            )),
        }
    }

    if errores.is_empty() {
        Ok(())
    } else {
        Err(errores.join("\n"))
    }
}

fn plan_desde_copia(
    raiz: &Path,
    copia: &CopiaKorunix,
    datos_copia: &[u8],
) -> Result<PlanRestauracion, String> {
    let texto_actual = fs::read_to_string(raiz.join("configuracion.toml")).map_err(|error| {
        format!("No pude leer la configuración actual antes de restaurar.\nDetalle: {error}")
    })?;
    let actual = configuracion::leer_texto(&texto_actual, "configuracion.toml actual")?;

    let lock_actual = fs::read(raiz.join("flake.lock"))
        .map_err(|error| format!("No pude leer flake.lock actual.\nDetalle: {error}"))?;

    let copia_configuracion = configuracion::leer_texto(
        &copia.configuracion.contenido,
        "configuracion.toml guardada en la copia",
    )?;

    let mut recursos_cambian = 0usize;
    for recurso in &copia.recursos {
        let destino = raiz.join(&recurso.ruta);
        comprobar_destino_local(raiz, &destino)?;

        if fs::read(&destino).ok().as_deref() != Some(recurso.datos.as_slice()) {
            recursos_cambian += 1;
        }
    }

    let configuracion_cambia = texto_actual.as_bytes() != copia.configuracion.contenido.as_bytes();
    let flake_lock_cambia = lock_actual != copia.flake_lock.contenido.as_bytes();
    let hay_cambios = configuracion_cambia || flake_lock_cambia || recursos_cambian > 0;

    Ok(PlanRestauracion {
        equipo_actual: actual.nombre,
        equipo_copia: copia_configuracion.nombre,
        canal_actual: actual.canal,
        canal_copia: copia_configuracion.canal,
        escritorio_actual: actual.escritorio.principal,
        escritorio_copia: copia_configuracion.escritorio.principal,
        personas_actual: actual.personas.len(),
        personas_copia: copia_configuracion.personas.len(),
        aplicaciones_actual: actual.aplicaciones.instaladas.len(),
        aplicaciones_copia: copia_configuracion.aplicaciones.instaladas.len(),
        configuracion_cambia,
        flake_lock_cambia,
        recursos_cambian,
        recursos_total: copia.recursos.len(),
        hay_cambios,
        sha256: sha256_bytes(datos_copia)?,
    })
}

pub fn plan_restauracion(raiz: &Path, ruta: &Path) -> Result<PlanRestauracion, String> {
    let (copia, datos) = leer_y_validar(ruta)?;
    plan_desde_copia(raiz, &copia, &datos)
}

fn restaurar_con_estado(
    raiz: &Path,
    ruta: &Path,
    estado: &Path,
    historial: &Path,
    fallo_despues: Option<usize>,
) -> Result<ResultadoRestauracion, String> {
    let (copia, datos_copia) = leer_y_validar(ruta)?;
    let plan = plan_desde_copia(raiz, &copia, &datos_copia)?;

    if !plan.hay_cambios {
        return Ok(ResultadoRestauracion {
            plan,
            proteccion: None,
        });
    }

    let protecciones = estado.join("protecciones");
    fs::create_dir_all(&protecciones).map_err(|error| {
        format!("No pude preparar la protección anterior a la restauración.\nDetalle: {error}")
    })?;

    let momento = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let proteccion = protecciones.join(format!(
        "Antes-de-restaurar-{}-{momento}.korunix-copia",
        process::id()
    ));

    let resumen_proteccion = crear_documento(raiz, &proteccion)?;
    if let Err(error) = registrar_proteccion_en(historial, &proteccion, &resumen_proteccion) {
        let _ = fs::remove_file(&proteccion);
        return Err(format!(
            "No voy a restaurar porque no pude registrar la protección del estado actual.\nDetalle: {error}"
        ));
    }

    let mut cambios = Vec::new();

    for recurso in &copia.recursos {
        if let Some(cambio) =
            preparar_cambio(raiz, raiz.join(&recurso.ruta), recurso.datos.clone())?
        {
            cambios.push(cambio);
        }
    }

    if let Some(cambio) = preparar_cambio(
        raiz,
        raiz.join("flake.lock"),
        copia.flake_lock.contenido.as_bytes().to_vec(),
    )? {
        cambios.push(cambio);
    }

    if let Some(cambio) = preparar_cambio(
        raiz,
        raiz.join("configuracion.toml"),
        copia.configuracion.contenido.as_bytes().to_vec(),
    )? {
        cambios.push(cambio);
    }

    let mut carpetas_creadas = Vec::new();
    let mut escritos = 0usize;

    let aplicar = (|| -> Result<(), String> {
        for cambio in &cambios {
            crear_carpetas_necesarias(raiz, &cambio.ruta, &mut carpetas_creadas)?;
            escribir_reemplazo_atomico(
                &cambio.ruta,
                &cambio.nuevo,
                cambio.modo_anterior.unwrap_or(0o600),
            )?;
            escritos += 1;

            if fallo_despues == Some(escritos) {
                return Err("Fallo simulado después de empezar la restauración.".to_string());
            }
        }

        let configuracion_final =
            fs::read_to_string(raiz.join("configuracion.toml")).map_err(|error| {
                format!(
                    "No pude verificar configuracion.toml después de restaurar.\nDetalle: {error}"
                )
            })?;
        configuracion::leer_texto(&configuracion_final, "configuracion.toml restaurada")?;

        let lock_final = fs::read_to_string(raiz.join("flake.lock")).map_err(|error| {
            format!("No pude verificar flake.lock restaurado.\nDetalle: {error}")
        })?;
        validar_flake_lock(&lock_final)?;

        if configuracion_final.as_bytes() != copia.configuracion.contenido.as_bytes() {
            return Err("configuracion.toml no quedó igual que la copia revisada.".to_string());
        }

        if lock_final.as_bytes() != copia.flake_lock.contenido.as_bytes() {
            return Err("flake.lock no quedó igual que la copia revisada.".to_string());
        }

        for recurso in &copia.recursos {
            let restaurado = fs::read(raiz.join(&recurso.ruta)).map_err(|error| {
                format!(
                    "No pude verificar «{}» después de restaurar.\nDetalle: {error}",
                    recurso.ruta
                )
            })?;

            if restaurado != recurso.datos {
                return Err(format!(
                    "«{}» no quedó igual que la copia revisada.",
                    recurso.ruta
                ));
            }
        }

        Ok(())
    })();

    if let Err(error) = aplicar {
        return match devolver_cambios(&cambios, &carpetas_creadas) {
            Ok(()) => Err(format!(
                "La restauración no se completó y Korunix devolvió los archivos al estado anterior.\nDetalle: {error}"
            )),
            Err(recuperacion) => Err(format!(
                "La restauración falló ({error}) y también hubo un problema al recuperar los archivos anteriores:\n{recuperacion}"
            )),
        };
    }

    if let Err(error) = registrar_restauracion_en(historial, ruta, &plan) {
        return Err(format!(
            "Los archivos quedaron restaurados y verificados, pero no pude registrar el resultado en Historial.\nLa protección anterior sigue guardada en «{}».\nDetalle: {error}",
            proteccion.display()
        ));
    }

    Ok(ResultadoRestauracion {
        plan,
        proteccion: Some(proteccion),
    })
}

pub fn restaurar(raiz: &Path, ruta: &Path) -> Result<ResultadoRestauracion, String> {
    let estado = estado_raiz()?;
    let historial = historial_ruta()?;
    restaurar_con_estado(raiz, ruta, &estado, &historial, None)
}

fn historial_desde(ruta: &Path) -> Result<Vec<HistorialVisible>, String> {
    if !ruta.exists() {
        return Ok(Vec::new());
    }

    let texto = fs::read_to_string(&ruta)
        .map_err(|error| format!("No pude leer Historial.\nDetalle: {error}"))?;
    let mut salida = Vec::new();

    for (indice, linea) in texto.lines().enumerate() {
        if linea.trim().is_empty() {
            continue;
        }

        let entrada: EntradaHistorial = serde_json::from_str(linea).map_err(|error| {
            format!(
                "Historial tiene una entrada dañada en la línea {}.\nDetalle: {error}",
                indice + 1
            )
        })?;

        if entrada.version != 1 {
            continue;
        }

        if !matches!(
            entrada.tipo.as_str(),
            TIPO_COPIA | TIPO_PROTECCION | TIPO_RESTAURACION
        ) {
            continue;
        }

        let ruta_copia = PathBuf::from(&entrada.archivo);
        let fuente = if !ruta_copia.is_file() {
            None
        } else {
            let datos = fs::read(&ruta_copia).map_err(|error| {
                format!(
                    "No pude comprobar «{}» desde Historial.\nDetalle: {error}",
                    ruta_copia.display()
                )
            })?;
            Some(sha256_bytes(&datos)? == entrada.sha256)
        };

        let estado = match entrada.tipo.as_str() {
            TIPO_COPIA => match fuente {
                Some(true) => "Disponible · íntegra".to_string(),
                Some(false) => "Cambió desde que Korunix la creó".to_string(),
                None => "No encontrada".to_string(),
            },
            TIPO_PROTECCION => match fuente {
                Some(true) => "Protección automática · íntegra".to_string(),
                Some(false) => "Protección automática · cambió después".to_string(),
                None => "Protección automática · ya no está disponible".to_string(),
            },
            TIPO_RESTAURACION => match fuente {
                Some(true) => "Restauración completada · fuente íntegra".to_string(),
                Some(false) => "Restauración completada · la fuente cambió después".to_string(),
                None => "Restauración completada · la fuente ya no está disponible".to_string(),
            },
            _ => unreachable!(),
        };

        let archivo = ruta_copia
            .file_name()
            .map(|nombre| nombre.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Copia de Korunix".to_string());

        salida.push(HistorialVisible {
            resumen: entrada.resumen,
            archivo,
            cuando: cuando_humano(entrada.momento_unix),
            estado,
        });
    }

    salida.reverse();
    Ok(salida)
}

pub fn historial() -> Result<Vec<HistorialVisible>, String> {
    let ruta = historial_ruta()?;
    historial_desde(&ruta)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prueba_raiz(nombre: &str) -> PathBuf {
        let raiz = env::temp_dir().join(format!(
            "korunix-copias-{nombre}-{}-{}",
            process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        fs::create_dir_all(&raiz).unwrap();
        fs::write(
            raiz.join("configuracion.toml"),
            r#"nombre = "prueba"

[[personas]]
cuenta = "ana"
nombre = "Ana"
avatar = "avatar-ana.bin"
clave_github = ".ssh/privada"

[escritorio]
principal = "niri"
instalados = ["niri"]
"#,
        )
        .unwrap();
        fs::write(raiz.join("avatar-ana.bin"), b"avatar de prueba").unwrap();
        fs::write(
            raiz.join("flake.lock"),
            r#"{"nodes":{"root":{}},"root":"root","version":7}"#,
        )
        .unwrap();
        raiz
    }

    #[test]
    fn crea_e_inspecciona_copia_portable_con_avatar() {
        let raiz = prueba_raiz("crear");
        let historial = raiz.join("estado/historial.jsonl");
        let destino = raiz.join("copia.korunix-copia");

        let creada = crear_con_historial(&raiz, &destino, &historial).unwrap();
        let revisada = inspeccionar(&destino).unwrap();

        assert_eq!(creada.equipo, "prueba");
        assert_eq!(creada.recursos, 1);
        assert_eq!(revisada.sha256, creada.sha256);

        let valor: serde_json::Value =
            serde_json::from_slice(&fs::read(&destino).unwrap()).unwrap();
        assert_eq!(valor["incluye_hardware"], false);
        assert_eq!(valor["incluye_credenciales"], false);
        assert_eq!(valor["incluye_historial"], false);
        assert_eq!(valor["recursos"][0]["ruta"], "avatar-ana.bin");
        assert!(valor["configuracion"]["contenido"]
            .as_str()
            .unwrap()
            .contains("clave_github"));

        fs::remove_dir_all(raiz).unwrap();
    }

    #[test]
    fn no_sobrescribe_una_copia_existente() {
        let raiz = prueba_raiz("sobrescribir");
        let historial = raiz.join("estado/historial.jsonl");
        let destino = raiz.join("copia.korunix-copia");

        crear_con_historial(&raiz, &destino, &historial).unwrap();
        let error = crear_con_historial(&raiz, &destino, &historial).unwrap_err();

        assert!(error.contains("no va a sobrescribir"));
        fs::remove_dir_all(raiz).unwrap();
    }

    #[test]
    fn detecta_configuracion_modificada_dentro_de_la_copia() {
        let raiz = prueba_raiz("corrupta");
        let historial = raiz.join("estado/historial.jsonl");
        let destino = raiz.join("copia.korunix-copia");

        crear_con_historial(&raiz, &destino, &historial).unwrap();

        let mut valor: serde_json::Value =
            serde_json::from_slice(&fs::read(&destino).unwrap()).unwrap();
        valor["configuracion"]["contenido"] =
            serde_json::Value::String("nombre = \"alterado\"".to_string());
        fs::write(&destino, serde_json::to_vec_pretty(&valor).unwrap()).unwrap();

        let error = inspeccionar(&destino).unwrap_err();
        assert!(error.contains("huella de integridad"));

        fs::remove_dir_all(raiz).unwrap();
    }

    #[test]
    fn plan_de_restauracion_no_modifica_archivos() {
        let raiz = prueba_raiz("plan-restauracion");
        let historial = raiz.join("estado/historial.jsonl");
        let copia = raiz.join("copia.korunix-copia");

        crear_con_historial(&raiz, &copia, &historial).unwrap();

        let config_antes = fs::read(raiz.join("configuracion.toml")).unwrap();
        let lock_antes = fs::read(raiz.join("flake.lock")).unwrap();
        let avatar_antes = fs::read(raiz.join("avatar-ana.bin")).unwrap();

        let plan = plan_restauracion(&raiz, &copia).unwrap();

        assert!(!plan.hay_cambios);
        assert_eq!(
            fs::read(raiz.join("configuracion.toml")).unwrap(),
            config_antes
        );
        assert_eq!(fs::read(raiz.join("flake.lock")).unwrap(), lock_antes);
        assert_eq!(fs::read(raiz.join("avatar-ana.bin")).unwrap(), avatar_antes);

        fs::remove_dir_all(raiz).unwrap();
    }

    #[test]
    fn restaura_configuracion_lock_y_recurso_y_protege_lo_anterior() {
        let raiz = prueba_raiz("restaurar");
        let estado = raiz.join("estado");
        let historial = estado.join("historial.jsonl");
        let copia = raiz.join("copia.korunix-copia");

        crear_con_historial(&raiz, &copia, &historial).unwrap();
        let config_copia = fs::read(raiz.join("configuracion.toml")).unwrap();
        let lock_copia = fs::read(raiz.join("flake.lock")).unwrap();
        let avatar_copia = fs::read(raiz.join("avatar-ana.bin")).unwrap();

        let config_modificada = String::from_utf8(config_copia.clone())
            .unwrap()
            .replace("nombre = \"prueba\"", "nombre = \"cambiada\"");
        fs::write(raiz.join("configuracion.toml"), config_modificada).unwrap();
        fs::write(
            raiz.join("flake.lock"),
            r#"{"nodes":{"otra":{}},"root":"otra","version":7}"#,
        )
        .unwrap();
        fs::write(raiz.join("avatar-ana.bin"), b"avatar modificado").unwrap();

        let resultado = restaurar_con_estado(&raiz, &copia, &estado, &historial, None).unwrap();

        assert!(resultado.plan.hay_cambios);
        assert!(resultado.proteccion.as_ref().unwrap().is_file());
        assert_eq!(
            fs::read(raiz.join("configuracion.toml")).unwrap(),
            config_copia
        );
        assert_eq!(fs::read(raiz.join("flake.lock")).unwrap(), lock_copia);
        assert_eq!(fs::read(raiz.join("avatar-ana.bin")).unwrap(), avatar_copia);

        let entradas = historial_desde(&historial).unwrap();
        assert!(entradas
            .iter()
            .any(|entrada| entrada.resumen.contains("Protección antes de restaurar")));
        assert!(entradas
            .iter()
            .any(|entrada| entrada.resumen.contains("Restauración de Korunix")));

        fs::remove_dir_all(raiz).unwrap();
    }

    #[test]
    fn fallo_intermedio_devuelve_todos_los_archivos() {
        let raiz = prueba_raiz("rollback-restauracion");
        let estado = raiz.join("estado");
        let historial = estado.join("historial.jsonl");
        let copia = raiz.join("copia.korunix-copia");

        crear_con_historial(&raiz, &copia, &historial).unwrap();

        let config_modificada = fs::read_to_string(raiz.join("configuracion.toml"))
            .unwrap()
            .replace("nombre = \"prueba\"", "nombre = \"cambiada\"");
        fs::write(raiz.join("configuracion.toml"), config_modificada).unwrap();
        fs::write(
            raiz.join("flake.lock"),
            r#"{"nodes":{"otra":{}},"root":"otra","version":7}"#,
        )
        .unwrap();
        fs::write(raiz.join("avatar-ana.bin"), b"avatar modificado").unwrap();

        let config_antes = fs::read(raiz.join("configuracion.toml")).unwrap();
        let lock_antes = fs::read(raiz.join("flake.lock")).unwrap();
        let avatar_antes = fs::read(raiz.join("avatar-ana.bin")).unwrap();

        let error = restaurar_con_estado(&raiz, &copia, &estado, &historial, Some(1)).unwrap_err();

        assert!(error.contains("devolvió los archivos al estado anterior"));
        assert_eq!(
            fs::read(raiz.join("configuracion.toml")).unwrap(),
            config_antes
        );
        assert_eq!(fs::read(raiz.join("flake.lock")).unwrap(), lock_antes);
        assert_eq!(fs::read(raiz.join("avatar-ana.bin")).unwrap(), avatar_antes);

        fs::remove_dir_all(raiz).unwrap();
    }

    #[test]
    fn restauracion_rechaza_un_enlace_simbolico_en_el_destino() {
        use std::os::unix::fs::symlink;

        let raiz = prueba_raiz("symlink");
        let estado = raiz.join("estado");
        let historial = estado.join("historial.jsonl");
        let copia = raiz.join("copia.korunix-copia");

        fs::create_dir_all(raiz.join("recursos")).unwrap();
        fs::rename(
            raiz.join("avatar-ana.bin"),
            raiz.join("recursos/avatar-ana.bin"),
        )
        .unwrap();

        let config = fs::read_to_string(raiz.join("configuracion.toml"))
            .unwrap()
            .replace("avatar-ana.bin", "recursos/avatar-ana.bin");
        fs::write(raiz.join("configuracion.toml"), config).unwrap();

        crear_con_historial(&raiz, &copia, &historial).unwrap();

        let exterior = raiz.with_extension("fuera");
        fs::create_dir_all(&exterior).unwrap();
        fs::remove_dir_all(raiz.join("recursos")).unwrap();
        symlink(&exterior, raiz.join("recursos")).unwrap();

        let error = restaurar_con_estado(&raiz, &copia, &estado, &historial, None).unwrap_err();
        assert!(error.contains("enlace simbólico"));
        assert!(!exterior.join("avatar-ana.bin").exists());

        fs::remove_file(raiz.join("recursos")).unwrap();
        fs::remove_dir_all(raiz).unwrap();
        fs::remove_dir_all(exterior).unwrap();
    }

    #[test]
    fn historial_distingue_copia_disponible_y_eliminada() {
        let raiz = prueba_raiz("historial");
        let historial = raiz.join("estado/historial.jsonl");
        let destino = raiz.join("copia.korunix-copia");

        crear_con_historial(&raiz, &destino, &historial).unwrap();
        let disponible = historial_desde(&historial).unwrap();
        assert_eq!(disponible.len(), 1);
        assert_eq!(disponible[0].estado, "Disponible · íntegra");

        fs::remove_file(&destino).unwrap();
        let eliminada = historial_desde(&historial).unwrap();
        assert_eq!(eliminada[0].estado, "No encontrada");

        fs::remove_dir_all(raiz).unwrap();
    }
}

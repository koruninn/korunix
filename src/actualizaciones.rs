use crate::{aplicar, configuracion, preview};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

const ARCHIVO_BUSQUEDA: &str = "busqueda.json";

#[derive(Clone, Debug)]
pub struct CambioActualizacion {
    pub nombre: String,
    pub antes: String,
    pub despues: String,
}

#[derive(Clone, Debug)]
pub struct ResultadoBusqueda {
    pub canal: String,
    pub cambios_directos: Vec<CambioActualizacion>,
    pub cambios_internos: usize,
    pub hay_cambios: bool,
}

#[derive(Clone, Debug)]
pub struct EstadoActualizaciones {
    pub canal: String,
    pub revision_actual: String,
    pub busqueda: Option<ResultadoBusqueda>,
    pub busqueda_antigua: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct BusquedaGuardada {
    base: String,
    candidata: String,
}

fn ahora_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

fn carpeta_actualizaciones() -> Result<PathBuf, String> {
    Ok(preview::carpeta_estado()?.join("actualizaciones"))
}

fn escribir_atomico(ruta: &Path, datos: &[u8]) -> Result<(), String> {
    let carpeta = ruta
        .parent()
        .ok_or_else(|| "La búsqueda de actualizaciones no tiene carpeta padre.".to_string())?;
    fs::create_dir_all(carpeta).map_err(|error| {
        format!("No pude preparar el estado de Actualizaciones.\nDetalle: {error}")
    })?;

    let nombre = ruta
        .file_name()
        .and_then(|valor| valor.to_str())
        .unwrap_or("actualizaciones");
    let temporal = carpeta.join(format!(
        ".{nombre}.escribiendo-{}-{}",
        process::id(),
        ahora_nanos()
    ));

    let resultado = (|| {
        let mut archivo = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporal)
            .map_err(|error| {
                format!("No pude preparar el estado de Actualizaciones.\nDetalle: {error}")
            })?;

        archivo.write_all(datos).map_err(|error| {
            format!("No pude guardar el estado de Actualizaciones.\nDetalle: {error}")
        })?;
        archivo.sync_all().map_err(|error| {
            format!("No pude confirmar el estado de Actualizaciones en el disco.\nDetalle: {error}")
        })?;
        drop(archivo);

        fs::rename(&temporal, ruta).map_err(|error| {
            format!("No pude terminar de guardar el estado de Actualizaciones.\nDetalle: {error}")
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

fn entender_lock(datos: &[u8], origen: &str) -> Result<Value, String> {
    let valor: Value = serde_json::from_slice(datos)
        .map_err(|error| format!("No pude entender {origen}.\nDetalle: {error}"))?;

    let objeto = valor
        .as_object()
        .ok_or_else(|| format!("{origen} no tiene la forma esperada."))?;

    if !objeto.get("version").is_some_and(Value::is_number)
        || !objeto.get("nodes").is_some_and(Value::is_object)
        || !objeto.get("root").is_some_and(Value::is_string)
    {
        return Err(format!(
            "{origen} no contiene version, nodes y root con la forma esperada."
        ));
    }

    Ok(valor)
}

fn leer_lock(ruta: &Path, origen: &str) -> Result<(Vec<u8>, Value), String> {
    let datos =
        fs::read(ruta).map_err(|error| format!("No pude leer {origen}.\nDetalle: {error}"))?;
    let valor = entender_lock(&datos, origen)?;
    Ok((datos, valor))
}

fn nodo_raiz<'a>(lock: &'a Value) -> Result<&'a Value, String> {
    let raiz = lock
        .get("root")
        .and_then(Value::as_str)
        .ok_or_else(|| "flake.lock no indica cuál es su nodo raíz.".to_string())?;

    lock.get("nodes")
        .and_then(Value::as_object)
        .and_then(|nodos| nodos.get(raiz))
        .ok_or_else(|| "flake.lock no contiene su nodo raíz.".to_string())
}

fn entradas_directas(lock: &Value) -> Result<BTreeMap<String, String>, String> {
    let entradas = nodo_raiz(lock)?
        .get("inputs")
        .and_then(Value::as_object)
        .ok_or_else(|| "flake.lock no contiene las entradas principales.".to_string())?;

    let mut salida = BTreeMap::new();

    for (nombre, referencia) in entradas {
        let Some(nodo) = referencia.as_str() else {
            return Err(format!(
                "La entrada principal «{nombre}» tiene una referencia que todavía no sé leer."
            ));
        };

        salida.insert(nombre.clone(), nodo.to_string());
    }

    Ok(salida)
}

fn nodos(lock: &Value) -> Result<&serde_json::Map<String, Value>, String> {
    lock.get("nodes")
        .and_then(Value::as_object)
        .ok_or_else(|| "flake.lock no contiene sus nodos.".to_string())
}

fn resumen_nodo(lock: &Value, clave: &str) -> String {
    let Some(nodo) = lock
        .get("nodes")
        .and_then(Value::as_object)
        .and_then(|nodos| nodos.get(clave))
    else {
        return "no estaba".to_string();
    };

    let Some(locked) = nodo.get("locked") else {
        return "sin revisión fija".to_string();
    };

    if let Some(revision) = locked.get("rev").and_then(Value::as_str) {
        return revision.chars().take(12).collect();
    }

    if let Some(referencia) = locked.get("ref").and_then(Value::as_str) {
        return referencia.to_string();
    }

    if let Some(hash) = locked.get("narHash").and_then(Value::as_str) {
        return hash.chars().take(20).collect();
    }

    "cambió".to_string()
}

fn nombre_humano(entrada: &str) -> String {
    match entrada {
        "nixpkgs-estable" => "Paquetes de NixOS (estable)".to_string(),
        "nixpkgs-inestable" => "Paquetes de NixOS (inestable)".to_string(),
        "aagl-estable" => "AAGL (estable)".to_string(),
        "aagl-inestable" => "AAGL (inestable)".to_string(),
        "millennium" => "Millennium para Steam".to_string(),
        "figma-linux-next" => "Figma".to_string(),
        "hatter" => "Iconos Hatter".to_string(),
        "nix-flatpak" => "Flatpak declarativo".to_string(),
        "spicetify-nix" => "Spotify con Spicetify".to_string(),
        otro => otro.to_string(),
    }
}

fn comparar(canal: &str, actual: &Value, candidata: &Value) -> Result<ResultadoBusqueda, String> {
    let entradas_actuales = entradas_directas(actual)?;
    let entradas_candidatas = entradas_directas(candidata)?;
    let nodos_actuales = nodos(actual)?;
    let nodos_candidatos = nodos(candidata)?;

    let nombres: BTreeSet<String> = entradas_actuales
        .keys()
        .chain(entradas_candidatas.keys())
        .cloned()
        .collect();

    let destinos_directos: BTreeSet<String> = entradas_actuales
        .values()
        .chain(entradas_candidatas.values())
        .cloned()
        .collect();

    let mut cambios_directos = Vec::new();

    for nombre in nombres {
        let antes = entradas_actuales.get(&nombre);
        let despues = entradas_candidatas.get(&nombre);

        let cambio = match (antes, despues) {
            (Some(antes), Some(despues)) => {
                antes != despues || nodos_actuales.get(antes) != nodos_candidatos.get(despues)
            }
            (None, None) => false,
            _ => true,
        };

        if !cambio {
            continue;
        }

        cambios_directos.push(CambioActualizacion {
            nombre: nombre_humano(&nombre),
            antes: antes
                .map(|clave| resumen_nodo(actual, clave))
                .unwrap_or_else(|| "no estaba".to_string()),
            despues: despues
                .map(|clave| resumen_nodo(candidata, clave))
                .unwrap_or_else(|| "ya no está".to_string()),
        });
    }

    let todas_las_claves: BTreeSet<String> = nodos_actuales
        .keys()
        .chain(nodos_candidatos.keys())
        .cloned()
        .collect();

    let cambios_internos = todas_las_claves
        .iter()
        .filter(|clave| {
            if destinos_directos.contains(*clave) {
                return false;
            }

            nodos_actuales.get(*clave) != nodos_candidatos.get(*clave)
        })
        .count();

    let hay_cambios = !cambios_directos.is_empty() || cambios_internos > 0;

    Ok(ResultadoBusqueda {
        canal: canal.to_string(),
        cambios_directos,
        cambios_internos,
        hay_cambios,
    })
}

fn revision_nixpkgs(canal: &str, lock: &Value) -> Result<String, String> {
    let entrada = match canal {
        "estable" => "nixpkgs-estable",
        "inestable" => "nixpkgs-inestable",
        otro => return Err(format!("No conozco el canal «{otro}».")),
    };

    let entradas = entradas_directas(lock)?;
    let clave = entradas
        .get(entrada)
        .ok_or_else(|| format!("flake.lock no contiene «{entrada}»."))?;

    Ok(resumen_nodo(lock, clave))
}

fn leer_busqueda_guardada(ruta: &Path) -> Result<Option<BusquedaGuardada>, String> {
    let datos = match fs::read(ruta) {
        Ok(datos) => datos,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "No pude leer la última búsqueda guardada.\nDetalle: {error}"
            ));
        }
    };

    let guardada: BusquedaGuardada = serde_json::from_slice(&datos)
        .map_err(|error| format!("La última búsqueda guardada está dañada.\nDetalle: {error}"))?;

    Ok(Some(guardada))
}

fn estado_en(raiz: &Path, estado: &Path) -> Result<EstadoActualizaciones, String> {
    let configuracion = configuracion::leer(&raiz.join("configuracion.toml"))?;
    let (lock_actual_bytes, lock_actual) =
        leer_lock(&raiz.join("flake.lock"), "flake.lock actual")?;
    let revision_actual = revision_nixpkgs(&configuracion.canal, &lock_actual)?;

    let ruta_busqueda = estado.join(ARCHIVO_BUSQUEDA);
    let Some(guardada) = leer_busqueda_guardada(&ruta_busqueda)? else {
        return Ok(EstadoActualizaciones {
            canal: configuracion.canal,
            revision_actual,
            busqueda: None,
            busqueda_antigua: false,
        });
    };

    if guardada.base.as_bytes() != lock_actual_bytes {
        return Ok(EstadoActualizaciones {
            canal: configuracion.canal,
            revision_actual,
            busqueda: None,
            busqueda_antigua: true,
        });
    }

    let base = entender_lock(guardada.base.as_bytes(), "la base de la búsqueda guardada")?;
    let candidata = entender_lock(
        guardada.candidata.as_bytes(),
        "la candidata de la búsqueda guardada",
    )?;
    let busqueda = comparar(&configuracion.canal, &base, &candidata)?;

    Ok(EstadoActualizaciones {
        canal: configuracion.canal,
        revision_actual,
        busqueda: Some(busqueda),
        busqueda_antigua: false,
    })
}

fn buscar_en(raiz: &Path, estado: &Path, nix: &OsStr) -> Result<ResultadoBusqueda, String> {
    let configuracion = configuracion::leer(&raiz.join("configuracion.toml"))?;
    let ruta_lock = raiz.join("flake.lock");
    let (base_bytes, base) = leer_lock(&ruta_lock, "flake.lock actual")?;

    fs::create_dir_all(estado).map_err(|error| {
        format!("No pude preparar el estado de Actualizaciones.\nDetalle: {error}")
    })?;

    let temporal = estado.join(format!(
        ".flake.lock-buscando-{}-{}",
        process::id(),
        ahora_nanos()
    ));
    let _ = fs::remove_file(&temporal);

    let salida = Command::new(nix)
        .arg("flake")
        .arg("update")
        .arg("--flake")
        .arg(raiz)
        .arg("--output-lock-file")
        .arg(&temporal)
        .current_dir(raiz)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| {
            format!("No pude iniciar la búsqueda de actualizaciones.\nDetalle: {error}")
        })?;

    let lock_despues = fs::read(&ruta_lock).map_err(|error| {
        format!("No pude comprobar flake.lock después de buscar.\nDetalle: {error}")
    })?;

    if lock_despues != base_bytes {
        let restaurar = escribir_atomico(&ruta_lock, &base_bytes);
        let _ = fs::remove_file(&temporal);

        return match restaurar {
            Ok(()) => Err(
                "Nix intentó cambiar flake.lock durante una búsqueda. Korunix devolvió el archivo original y descartó esa búsqueda."
                    .to_string(),
            ),
            Err(error) => Err(format!(
                "Nix cambió flake.lock durante una búsqueda y tampoco pude devolver el archivo original.\nDetalle: {error}"
            )),
        };
    }

    if !salida.status.success() {
        let _ = fs::remove_file(&temporal);
        let detalle = String::from_utf8_lossy(&salida.stderr).trim().to_string();

        return Err(if detalle.is_empty() {
            "No pude completar la búsqueda de actualizaciones. La última búsqueda válida sigue guardada."
                .to_string()
        } else {
            format!(
                "No pude completar la búsqueda de actualizaciones. La última búsqueda válida sigue guardada.\nDetalle: {detalle}"
            )
        });
    }

    let candidata_bytes = fs::read(&temporal).map_err(|error| {
        format!("Nix terminó la búsqueda, pero no dejó la candidata esperada.\nDetalle: {error}")
    })?;
    let candidata = entender_lock(&candidata_bytes, "el flake.lock candidato")?;
    let resultado = comparar(&configuracion.canal, &base, &candidata)?;

    let guardada = BusquedaGuardada {
        base: String::from_utf8(base_bytes)
            .map_err(|_| "flake.lock actual no es texto UTF-8 válido.".to_string())?,
        candidata: String::from_utf8(candidata_bytes)
            .map_err(|_| "El flake.lock candidato no es texto UTF-8 válido.".to_string())?,
    };
    let datos = serde_json::to_vec_pretty(&guardada).map_err(|error| {
        format!("No pude preparar la búsqueda para guardarla.\nDetalle: {error}")
    })?;

    escribir_atomico(&estado.join(ARCHIVO_BUSQUEDA), &datos)?;
    let _ = fs::remove_file(&temporal);

    Ok(resultado)
}

pub fn estado(raiz: &Path) -> Result<EstadoActualizaciones, String> {
    let estado = carpeta_actualizaciones()?;
    estado_en(raiz, &estado)
}

pub fn buscar(raiz: &Path) -> Result<ResultadoBusqueda, String> {
    let estado = carpeta_actualizaciones()?;
    let nix = env::var_os("KORUNIX_NIX_BIN").unwrap_or_else(|| "nix".into());
    buscar_en(raiz, &estado, nix.as_os_str())
}

pub fn preparar_preview(raiz: &Path) -> Result<(ResultadoBusqueda, preview::Preview), String> {
    let estado = carpeta_actualizaciones()?;
    let configuracion = configuracion::leer(&raiz.join("configuracion.toml"))?;
    let (lock_actual, _) = leer_lock(&raiz.join("flake.lock"), "flake.lock actual")?;

    let guardada = leer_busqueda_guardada(&estado.join(ARCHIVO_BUSQUEDA))?.ok_or_else(|| {
        "No hay una búsqueda de actualizaciones guardada. Busca primero.".to_string()
    })?;

    if guardada.base.as_bytes() != lock_actual {
        return Err(
            "La búsqueda guardada ya no corresponde a tu flake.lock actual. Busca otra vez."
                .to_string(),
        );
    }

    let base = entender_lock(guardada.base.as_bytes(), "la base de la búsqueda guardada")?;
    let candidata = entender_lock(
        guardada.candidata.as_bytes(),
        "la candidata de la búsqueda guardada",
    )?;
    let cambios = comparar(&configuracion.canal, &base, &candidata)?;

    if !cambios.hay_cambios {
        return Err(
            "La última búsqueda no encontró novedades. No hace falta construir otro preview."
                .to_string(),
        );
    }

    aplicar::conservar_aplicada_actual(raiz)?;

    let construido = preview::crear_con_lock(raiz, guardada.candidata.as_bytes())?;

    let lock_despues = fs::read(raiz.join("flake.lock")).map_err(|error| {
        format!("No pude comprobar flake.lock después del preview.\nDetalle: {error}")
    })?;

    if lock_despues != lock_actual {
        return Err(
            "flake.lock cambió mientras se construía el preview de actualización. No voy a considerar válida esa revisión."
                .to_string(),
        );
    }

    Ok((cambios, construido))
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn temporal(nombre: &str) -> PathBuf {
        env::temp_dir().join(format!(
            "korunix-actualizaciones-{nombre}-{}-{}",
            process::id(),
            ahora_nanos()
        ))
    }

    fn preparar_raiz(nombre: &str) -> PathBuf {
        let raiz = temporal(nombre);
        fs::create_dir_all(&raiz).unwrap();
        fs::write(
            raiz.join("configuracion.toml"),
            r#"nombre = "prueba"
canal = "inestable"

[[personas]]
cuenta = "ana"
nombre = "Ana"

[escritorio]
principal = "niri"
instalados = ["niri"]
"#,
        )
        .unwrap();
        fs::write(raiz.join("flake.lock"), lock_actual()).unwrap();
        raiz
    }

    fn lock_actual() -> String {
        serde_json::json!({
            "nodes": {
                "root": {
                    "inputs": {
                        "nixpkgs-inestable": "nixpkgs-inestable",
                        "hatter": "hatter"
                    }
                },
                "nixpkgs-inestable": {
                    "locked": {"rev": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}
                },
                "hatter": {
                    "locked": {"rev": "1111111111111111111111111111111111111111"}
                },
                "interno": {
                    "locked": {"rev": "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"}
                }
            },
            "root": "root",
            "version": 7
        })
        .to_string()
    }

    fn lock_candidato() -> String {
        serde_json::json!({
            "nodes": {
                "root": {
                    "inputs": {
                        "nixpkgs-inestable": "nixpkgs-inestable",
                        "hatter": "hatter"
                    }
                },
                "nixpkgs-inestable": {
                    "locked": {"rev": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}
                },
                "hatter": {
                    "locked": {"rev": "1111111111111111111111111111111111111111"}
                },
                "interno": {
                    "locked": {"rev": "yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy"}
                }
            },
            "root": "root",
            "version": 7
        })
        .to_string()
    }

    fn programa(carpeta: &Path, nombre: &str, cuerpo: &str) -> PathBuf {
        let ruta = carpeta.join(nombre);
        let temporal = carpeta.join(format!(".{nombre}.escribiendo-{}", process::id()));
        fs::write(&temporal, cuerpo).unwrap();
        let mut permisos = fs::metadata(&temporal).unwrap().permissions();
        permisos.set_mode(0o755);
        fs::set_permissions(&temporal, permisos).unwrap();
        fs::rename(&temporal, &ruta).unwrap();
        ruta
    }

    fn nix_que_escribe(carpeta: &Path, candidata: &str) -> PathBuf {
        let cuerpo = format!(
            r#"#!/bin/sh
set -eu
salida=""
while [ "$#" -gt 0 ]; do
  if [ "$1" = "--output-lock-file" ]; then
    salida="$2"
    shift 2
  else
    shift
  fi
done
[ -n "$salida" ]
cat > "$salida" <<'KORUNIX_LOCK'
{candidata}
KORUNIX_LOCK
"#
        );

        programa(carpeta, "nix-falso", &cuerpo)
    }

    #[test]
    fn buscar_no_modifica_flake_lock_y_guarda_la_candidata() {
        let raiz = preparar_raiz("buscar");
        let estado = raiz.join("estado");
        let nix = nix_que_escribe(&raiz, &lock_candidato());
        let antes = fs::read(raiz.join("flake.lock")).unwrap();

        let resultado = buscar_en(&raiz, &estado, nix.as_os_str()).unwrap();

        assert!(resultado.hay_cambios);
        assert_eq!(resultado.cambios_directos.len(), 1);
        assert_eq!(resultado.cambios_internos, 1);
        assert_eq!(fs::read(raiz.join("flake.lock")).unwrap(), antes);
        assert!(estado.join(ARCHIVO_BUSQUEDA).is_file());

        let visible = estado_en(&raiz, &estado).unwrap();
        assert!(visible.busqueda.is_some());
        assert!(!visible.busqueda_antigua);

        fs::remove_dir_all(raiz).unwrap();
    }

    #[test]
    fn una_busqueda_fallida_conserva_la_anterior() {
        let raiz = preparar_raiz("fallo");
        let estado = raiz.join("estado");
        let buena = nix_que_escribe(&raiz, &lock_candidato());

        buscar_en(&raiz, &estado, buena.as_os_str()).unwrap();
        let guardada_antes = fs::read(estado.join(ARCHIVO_BUSQUEDA)).unwrap();

        let mala = programa(&raiz, "nix-falla", "#!/bin/sh\nexit 7\n");

        assert!(
            buscar_en(&raiz, &estado, mala.as_os_str()).is_err(),
            "una búsqueda fallida debe terminar como error"
        );
        assert_eq!(
            fs::read(estado.join(ARCHIVO_BUSQUEDA)).unwrap(),
            guardada_antes,
            "una búsqueda fallida debe conservar byte por byte la última búsqueda válida"
        );

        fs::remove_dir_all(raiz).unwrap();
    }

    #[test]
    fn una_busqueda_vieja_no_se_presenta_como_vigente() {
        let raiz = preparar_raiz("antigua");
        let estado = raiz.join("estado");
        let nix = nix_que_escribe(&raiz, &lock_candidato());

        buscar_en(&raiz, &estado, nix.as_os_str()).unwrap();
        fs::write(
            raiz.join("flake.lock"),
            lock_actual().replace("aaaaaaaaaaaa", "cccccccccccc"),
        )
        .unwrap();

        let visible = estado_en(&raiz, &estado).unwrap();

        assert!(visible.busqueda.is_none());
        assert!(visible.busqueda_antigua);

        fs::remove_dir_all(raiz).unwrap();
    }

    #[test]
    fn una_busqueda_sin_cambios_se_reconoce_como_al_dia() {
        let raiz = preparar_raiz("sin-cambios");
        let estado = raiz.join("estado");
        let nix = nix_que_escribe(&raiz, &lock_actual());

        let resultado = buscar_en(&raiz, &estado, nix.as_os_str()).unwrap();

        assert!(!resultado.hay_cambios);
        assert!(resultado.cambios_directos.is_empty());
        assert_eq!(resultado.cambios_internos, 0);

        fs::remove_dir_all(raiz).unwrap();
    }
}

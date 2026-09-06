use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

const GENERACION_NIXOS: &str = ".#nixosConfigurations.korunix.config.system.build.toplevel";
const ARCHIVO_GENERACION: &str = "preview-generacion";
const ARCHIVO_CONFIGURACION: &str = "preview-configuracion.toml";
const ARCHIVO_LOCK_BASE: &str = "preview-flake-base.lock";
const ARCHIVO_LOCK_USADO: &str = "preview-flake-usado.lock";

#[derive(Debug)]
pub struct Preview {
    pub generacion: PathBuf,
    pub enlace: PathBuf,
}

pub fn carpeta_estado() -> Result<PathBuf, String> {
    if let Some(ruta) = env::var_os("XDG_STATE_HOME") {
        return Ok(PathBuf::from(ruta).join("korunix"));
    }

    if let Some(home) = env::var_os("HOME") {
        return Ok(PathBuf::from(home).join(".local/state/korunix"));
    }

    Err("No pude saber dónde guardar el preview.".to_string())
}

fn estado_enlace(ruta: &Path) -> Result<Option<PathBuf>, String> {
    match fs::symlink_metadata(ruta) {
        Ok(datos) if datos.file_type().is_symlink() => fs::read_link(ruta)
            .map(Some)
            .map_err(|error| format!("No pude leer el preview anterior.\nDetalle: {error}")),
        Ok(_) => Err(format!(
            "No voy a reemplazar {} porque no es un enlace de preview.",
            ruta.display()
        )),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!(
            "No pude revisar el preview anterior.\nDetalle: {error}"
        )),
    }
}

fn limpiar_enlace(ruta: &Path) {
    if matches!(fs::symlink_metadata(ruta), Ok(datos) if datos.file_type().is_symlink() || datos.is_file())
    {
        let _ = fs::remove_file(ruta);
    }
}

fn pedir_generacion_con_lock(
    raiz: &Path,
    enlace_temporal: &Path,
    programa: &OsStr,
    lock_referencia: Option<&Path>,
) -> Result<PathBuf, String> {
    limpiar_enlace(enlace_temporal);

    let mut comando = Command::new(programa);
    comando.arg("build").arg("--out-link").arg(enlace_temporal);

    if let Some(lock) = lock_referencia {
        comando
            .arg("--reference-lock-file")
            .arg(lock)
            .arg("--no-write-lock-file");
    }

    let resultado = comando
        .arg(GENERACION_NIXOS)
        .current_dir(raiz)
        .status()
        .map_err(|error| format!("No pude pedirle el preview a Nix.\nDetalle: {error}"))?;

    if !resultado.success() {
        limpiar_enlace(enlace_temporal);
        return Err("Nix no pudo construir el preview.".to_string());
    }

    let generacion = fs::read_link(enlace_temporal)
        .map_err(|error| format!("No pude leer la generación del preview.\nDetalle: {error}"))?;

    if !generacion.is_absolute() || !generacion.starts_with("/nix/store") {
        limpiar_enlace(enlace_temporal);
        return Err(format!(
            "Nix dejó el preview en una ruta que no esperaba: {}",
            generacion.display()
        ));
    }

    Ok(generacion)
}

fn pedir_generacion(
    raiz: &Path,
    enlace_temporal: &Path,
    programa: &OsStr,
) -> Result<PathBuf, String> {
    pedir_generacion_con_lock(raiz, enlace_temporal, programa, None)
}

fn registrar_enlace(
    estado: &Path,
    generacion: &Path,
    nix_store: &OsStr,
) -> Result<PathBuf, String> {
    fs::create_dir_all(estado)
        .map_err(|error| format!("No pude preparar la carpeta del preview.\nDetalle: {error}"))?;

    let enlace = estado.join("preview");
    let anterior = estado_enlace(&enlace)?;

    if anterior.is_some() {
        fs::remove_file(&enlace).map_err(|error| {
            format!(
                "La generación nueva está construida, pero no pude preparar el enlace de preview.\n\
                 El preview anterior sigue siendo el válido.\nDetalle: {error}"
            )
        })?;
    }

    let resultado = Command::new(nix_store)
        .arg("--add-root")
        .arg(&enlace)
        .arg("--indirect")
        .arg("--realise")
        .arg(generacion)
        .status();

    let fallo = match resultado {
        Ok(estado) if estado.success() => None,
        Ok(_) => Some("Nix devolvió un error al registrar la raíz de GC.".to_string()),
        Err(error) => Some(format!(
            "No pude proteger el preview frente a GC.\nDetalle: {error}"
        )),
    };

    if let Some(fallo) = fallo {
        if let Some(anterior) = anterior {
            let _ = fs::remove_file(&enlace);
            let _ = Command::new(nix_store)
                .arg("--add-root")
                .arg(&enlace)
                .arg("--indirect")
                .arg("--realise")
                .arg(anterior)
                .status();
        }

        return Err(format!(
            "La generación nueva está construida, pero Nix no pudo guardarla como preview protegido.\n{fallo}"
        ));
    }

    let guardada = fs::read_link(&enlace)
        .map_err(|error| format!("No pude leer el preview recién guardado.\nDetalle: {error}"))?;

    if guardada != generacion {
        return Err(format!(
            "El preview guardado no apunta a la generación recién construida.\n\
             Esperada: {}\nEncontrada: {}",
            generacion.display(),
            guardada.display()
        ));
    }

    Ok(enlace)
}

fn construir_con_lock_en(
    raiz: &Path,
    estado: &Path,
    nix: &OsStr,
    nix_store: &OsStr,
    lock_referencia: Option<&Path>,
) -> Result<Preview, String> {
    fs::create_dir_all(estado)
        .map_err(|error| format!("No pude preparar la carpeta del preview.\nDetalle: {error}"))?;

    // El preview estable no se toca mientras Nix construye. El enlace temporal
    // mantiene viva la generación nueva hasta registrar la raíz de GC definitiva.
    let temporal = estado.join(format!(".preview-construyendo-{}", process::id()));
    let generacion = pedir_generacion_con_lock(raiz, &temporal, nix, lock_referencia)?;
    let resultado = registrar_enlace(estado, &generacion, nix_store);
    limpiar_enlace(&temporal);
    let enlace = resultado?;

    Ok(Preview { generacion, enlace })
}

fn construir_en(
    raiz: &Path,
    estado: &Path,
    nix: &OsStr,
    nix_store: &OsStr,
) -> Result<Preview, String> {
    construir_con_lock_en(raiz, estado, nix, nix_store, None)
}

fn escribir_atomico(ruta: &Path, contenido: &[u8]) -> Result<(), String> {
    let nombre = ruta
        .file_name()
        .ok_or_else(|| format!("No pude preparar {}.", ruta.display()))?
        .to_string_lossy();
    let temporal = ruta.with_file_name(format!(".{nombre}-{}", process::id()));

    fs::write(&temporal, contenido).map_err(|error| {
        format!(
            "No pude guardar los datos del preview en {}.\nDetalle: {error}",
            temporal.display()
        )
    })?;

    fs::rename(&temporal, ruta).map_err(|error| {
        let _ = fs::remove_file(&temporal);
        format!(
            "No pude terminar de guardar {}.\nDetalle: {error}",
            ruta.display()
        )
    })
}

fn guardar_datos(
    raiz: &Path,
    estado: &Path,
    preview: &Preview,
    lock_base: &[u8],
    lock_usado: &[u8],
) -> Result<(), String> {
    let configuracion = fs::read(raiz.join("configuracion.toml")).map_err(|error| {
        format!(
            "El preview se construyó, pero no pude guardar con qué configuración se hizo.\nDetalle: {error}"
        )
    })?;

    let generacion = format!("{}\n", preview.generacion.display());

    // La generación se escribe al final. Si aparece, todos los datos necesarios
    // para saber exactamente qué se revisó ya quedaron guardados.
    escribir_atomico(&estado.join(ARCHIVO_CONFIGURACION), &configuracion)?;
    escribir_atomico(&estado.join(ARCHIVO_LOCK_BASE), lock_base)?;
    escribir_atomico(&estado.join(ARCHIVO_LOCK_USADO), lock_usado)?;
    escribir_atomico(&estado.join(ARCHIVO_GENERACION), generacion.as_bytes())?;
    Ok(())
}

fn crear_con_lock_en(
    raiz: &Path,
    estado: &Path,
    nix: &OsStr,
    nix_store: &OsStr,
    lock_usado: Option<&[u8]>,
) -> Result<Preview, String> {
    let lock_base = fs::read(raiz.join("flake.lock"))
        .map_err(|error| format!("No pude leer flake.lock antes del preview.\nDetalle: {error}"))?;
    let lock_usado = lock_usado.unwrap_or(lock_base.as_slice());

    let lock_temporal = if lock_usado != lock_base.as_slice() {
        fs::create_dir_all(estado).map_err(|error| {
            format!("No pude preparar la carpeta del preview.\nDetalle: {error}")
        })?;
        let ruta = estado.join(format!(".preview-lock-usado-{}", process::id()));
        let _ = fs::remove_file(&ruta);
        escribir_atomico(&ruta, lock_usado)?;
        Some(ruta)
    } else {
        None
    };

    let preview = construir_con_lock_en(raiz, estado, nix, nix_store, lock_temporal.as_deref());

    if let Some(ruta) = &lock_temporal {
        let _ = fs::remove_file(ruta);
    }

    let preview = preview?;

    if let Err(error) = guardar_datos(raiz, estado, &preview, &lock_base, lock_usado) {
        return Err(format!(
            "{error}\n\
             La generación construida no se considera aplicable. Ejecuta el preview otra vez."
        ));
    }

    Ok(preview)
}

fn crear_en(raiz: &Path, estado: &Path, nix: &OsStr, nix_store: &OsStr) -> Result<Preview, String> {
    crear_con_lock_en(raiz, estado, nix, nix_store, None)
}

pub(crate) fn datos_guardados(estado: &Path) -> Result<Option<(PathBuf, Vec<u8>)>, String> {
    let enlace = estado.join("preview");

    let datos = match fs::symlink_metadata(&enlace) {
        Ok(datos) => datos,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "No pude revisar el preview anterior antes de crear el nuevo.\nDetalle: {error}"
            ));
        }
    };

    if !datos.file_type().is_symlink() {
        return Err(format!(
            "{} no es un enlace de preview válido.",
            enlace.display()
        ));
    }

    let generacion = fs::read_link(&enlace)
        .map_err(|error| format!("No pude leer el preview anterior.\nDetalle: {error}"))?;

    if !generacion.is_absolute() || !generacion.starts_with("/nix/store") {
        return Err(format!(
            "El preview anterior apunta fuera de /nix/store: {}",
            generacion.display()
        ));
    }

    let generacion_guardada =
        fs::read_to_string(estado.join(ARCHIVO_GENERACION)).map_err(|_| {
            "El preview anterior no tiene sus datos de generación completos.".to_string()
        })?;

    if generacion_guardada.trim() != generacion.to_string_lossy() {
        return Err("El enlace y los datos del preview anterior no coinciden.".to_string());
    }

    let configuracion = fs::read(estado.join(ARCHIVO_CONFIGURACION)).map_err(|_| {
        "El preview anterior no tiene guardada su configuración humana.".to_string()
    })?;

    Ok(Some((generacion, configuracion)))
}

pub(crate) fn leer_en(raiz: &Path, estado: &Path) -> Result<Preview, String> {
    let enlace = estado.join("preview");
    let datos = fs::symlink_metadata(&enlace).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            "No hay un preview guardado. Ejecuta primero «korunix preview».".to_string()
        } else {
            format!("No pude revisar el preview guardado.\nDetalle: {error}")
        }
    })?;

    if !datos.file_type().is_symlink() {
        return Err(format!(
            "{} no es un enlace de preview válido.",
            enlace.display()
        ));
    }

    let generacion = fs::read_link(&enlace)
        .map_err(|error| format!("No pude leer el preview guardado.\nDetalle: {error}"))?;

    if !generacion.is_absolute() || !generacion.starts_with("/nix/store") {
        return Err(format!(
            "El preview guardado apunta fuera de /nix/store: {}",
            generacion.display()
        ));
    }

    let generacion_guardada = fs::read_to_string(estado.join(ARCHIVO_GENERACION)).map_err(|_| {
        "Este preview es anterior a la comprobación de cambios. Crea uno nuevo con «korunix preview»."
            .to_string()
    })?;

    if generacion_guardada.trim() != generacion.to_string_lossy() {
        return Err(
            "Los datos guardados del preview no coinciden con su generación. Crea un preview nuevo."
                .to_string(),
        );
    }

    let configuracion_guardada = fs::read(estado.join(ARCHIVO_CONFIGURACION)).map_err(|_| {
        "No encontré la configuración con la que se construyó el preview. Crea un preview nuevo."
            .to_string()
    })?;
    let configuracion_actual = fs::read(raiz.join("configuracion.toml"))
        .map_err(|error| format!("No pude leer configuracion.toml.\nDetalle: {error}"))?;

    if configuracion_actual != configuracion_guardada {
        return Err(
            "configuracion.toml cambió después del preview. Crea un preview nuevo antes de aplicar."
                .to_string(),
        );
    }

    let lock_base = fs::read(estado.join(ARCHIVO_LOCK_BASE)).map_err(|_| {
        "Este preview no recuerda el flake.lock del que partió. Crea un preview nuevo.".to_string()
    })?;
    let lock_usado = fs::read(estado.join(ARCHIVO_LOCK_USADO)).map_err(|_| {
        "Este preview no recuerda el flake.lock que usó. Crea un preview nuevo.".to_string()
    })?;
    let lock_actual = fs::read(raiz.join("flake.lock"))
        .map_err(|error| format!("No pude leer flake.lock.\nDetalle: {error}"))?;

    if lock_actual != lock_base {
        return Err(
            "flake.lock cambió después del preview. Crea un preview nuevo antes de aplicar."
                .to_string(),
        );
    }

    if lock_usado.is_empty() {
        return Err("El flake.lock guardado por el preview está vacío.".to_string());
    }

    let activador = generacion.join("bin/switch-to-configuration");
    let activador_datos = fs::metadata(&activador).map_err(|error| {
        format!(
            "La generación guardada no parece una generación NixOS aplicable.\nDetalle: {error}"
        )
    })?;

    if !activador_datos.is_file() || activador_datos.permissions().mode() & 0o111 == 0 {
        return Err("La generación guardada no tiene un activador NixOS ejecutable.".to_string());
    }

    Ok(Preview { generacion, enlace })
}

pub(crate) fn locks_guardados(estado: &Path) -> Result<(Vec<u8>, Vec<u8>), String> {
    let base = fs::read(estado.join(ARCHIVO_LOCK_BASE))
        .map_err(|_| "El preview no tiene guardado el flake.lock del que partió.".to_string())?;
    let usado = fs::read(estado.join(ARCHIVO_LOCK_USADO))
        .map_err(|_| "El preview no tiene guardado el flake.lock que usó.".to_string())?;
    Ok((base, usado))
}

pub(crate) fn usa_lock_distinto(raiz: &Path) -> Result<bool, String> {
    let estado = carpeta_estado()?;
    let (base, usado) = locks_guardados(&estado)?;
    let actual = fs::read(raiz.join("flake.lock"))
        .map_err(|error| format!("No pude leer flake.lock.\nDetalle: {error}"))?;

    if actual != base {
        return Err(
            "flake.lock cambió después del preview. Crea un preview nuevo antes de aplicar."
                .to_string(),
        );
    }

    Ok(base != usado)
}

pub fn leer(raiz: &Path) -> Result<Preview, String> {
    let estado = carpeta_estado()?;
    leer_en(raiz, &estado)
}

pub fn crear(raiz: &Path) -> Result<Preview, String> {
    let estado = carpeta_estado()?;
    let nix = env::var_os("KORUNIX_NIX_BIN").unwrap_or_else(|| "nix".into());
    let nix_store = env::var_os("KORUNIX_NIX_STORE_BIN").unwrap_or_else(|| "nix-store".into());

    crear_en(raiz, &estado, nix.as_os_str(), nix_store.as_os_str())
}

pub fn crear_con_lock(raiz: &Path, lock: &[u8]) -> Result<Preview, String> {
    let estado = carpeta_estado()?;
    let nix = env::var_os("KORUNIX_NIX_BIN").unwrap_or_else(|| "nix".into());
    let nix_store = env::var_os("KORUNIX_NIX_STORE_BIN").unwrap_or_else(|| "nix-store".into());

    crear_con_lock_en(
        raiz,
        &estado,
        nix.as_os_str(),
        nix_store.as_os_str(),
        Some(lock),
    )
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use std::os::unix::fs::{symlink, PermissionsExt};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporal(nombre: &str) -> PathBuf {
        let momento = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("el reloj debería funcionar")
            .as_nanos();

        env::temp_dir().join(format!(
            "korunix-preview-{nombre}-{}-{momento}",
            process::id()
        ))
    }

    fn programa(carpeta: &Path, nombre: &str, cuerpo: &str) -> PathBuf {
        let ruta = carpeta.join(nombre);
        let temporal = carpeta.join(format!(".{nombre}.escribiendo-{}", process::id()));

        fs::write(&temporal, cuerpo).expect("debería escribir el programa de prueba");

        let mut permisos = fs::metadata(&temporal)
            .expect("debería leer los permisos")
            .permissions();
        permisos.set_mode(0o755);
        fs::set_permissions(&temporal, permisos).expect("debería hacer ejecutable el programa");

        fs::rename(&temporal, &ruta)
            .expect("debería publicar el programa después de cerrar su escritura");
        ruta
    }

    fn nix_que_crea(carpeta: &Path, generacion: &str) -> PathBuf {
        programa(
            carpeta,
            "nix-de-prueba.sh",
            &format!(
                r#"#!/bin/sh
set -eu
enlace=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --out-link)
      enlace="$2"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done
ln -s {generacion} "$enlace"
"#
            ),
        )
    }

    fn nix_store_que_enlaza(carpeta: &Path) -> PathBuf {
        programa(
            carpeta,
            "nix-store-de-prueba.sh",
            r#"#!/bin/sh
set -eu
enlace="$2"
destino="$5"
ln -s "$destino" "$enlace"
"#,
        )
    }

    #[test]
    fn el_primer_preview_conserva_la_generacion_exacta() {
        let carpeta = temporal("primero");
        let raiz = carpeta.join("repo");
        let estado = carpeta.join("estado");
        fs::create_dir_all(&raiz).expect("debería crear la prueba");
        fs::write(raiz.join("configuracion.toml"), "nombre = \"prueba\"\n")
            .expect("debería crear la configuración");
        fs::write(
            raiz.join("flake.lock"),
            b"{\"nodes\":{},\"root\":\"root\",\"version\":7}\n",
        )
        .expect("debería crear el lock");

        let esperada = "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-nixos-system-korunix";
        let nix = nix_que_crea(&carpeta, esperada);
        let nix_store = nix_store_que_enlaza(&carpeta);

        let preview = crear_en(&raiz, &estado, nix.as_os_str(), nix_store.as_os_str())
            .expect("el preview debería construirse");

        assert_eq!(preview.enlace, estado.join("preview"));
        assert_eq!(preview.generacion, PathBuf::from(esperada));
        assert_eq!(
            fs::read_link(&preview.enlace).expect("debería leer el enlace"),
            preview.generacion
        );
        assert_eq!(
            fs::read(estado.join(ARCHIVO_CONFIGURACION)).unwrap(),
            b"nombre = \"prueba\"\n"
        );
        assert_eq!(
            fs::read_to_string(estado.join(ARCHIVO_GENERACION)).unwrap(),
            format!("{esperada}\n")
        );
        assert_eq!(
            fs::read(estado.join(ARCHIVO_LOCK_BASE)).unwrap(),
            fs::read(raiz.join("flake.lock")).unwrap()
        );
        assert_eq!(
            fs::read(estado.join(ARCHIVO_LOCK_USADO)).unwrap(),
            fs::read(raiz.join("flake.lock")).unwrap()
        );

        let _ = fs::remove_dir_all(&carpeta);
    }

    #[test]
    fn un_fallo_conserva_el_preview_anterior() {
        let carpeta = temporal("fallo");
        let raiz = carpeta.join("repo");
        let estado = carpeta.join("estado");
        fs::create_dir_all(&raiz).expect("debería crear la prueba");
        fs::create_dir_all(&estado).expect("debería crear el estado");

        let anterior = "/nix/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-preview-anterior";
        let enlace = estado.join("preview");
        symlink(anterior, &enlace).expect("debería crear el preview anterior");

        let nix = programa(&carpeta, "nix-falla.sh", "#!/bin/sh\nexit 1\n");
        let nix_store = nix_store_que_enlaza(&carpeta);

        let error = construir_en(&raiz, &estado, nix.as_os_str(), nix_store.as_os_str())
            .expect_err("el preview nuevo debería fallar");

        assert!(error.contains("no pudo construir"));
        assert_eq!(
            fs::read_link(&enlace).expect("el preview anterior debería seguir"),
            PathBuf::from(anterior)
        );

        let _ = fs::remove_dir_all(&carpeta);
    }

    #[test]
    fn el_nuevo_preview_reemplaza_al_anterior_solo_si_termina_bien() {
        let carpeta = temporal("reemplazo");
        let raiz = carpeta.join("repo");
        let estado = carpeta.join("estado");
        fs::create_dir_all(&raiz).expect("debería crear la prueba");
        fs::create_dir_all(&estado).expect("debería crear el estado");

        let anterior = "/nix/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-preview-anterior";
        let nuevo = "/nix/store/cccccccccccccccccccccccccccccccc-preview-nuevo";
        let enlace = estado.join("preview");
        symlink(anterior, &enlace).expect("debería crear el preview anterior");

        let nix = nix_que_crea(&carpeta, nuevo);
        let nix_store = nix_store_que_enlaza(&carpeta);
        let preview = construir_en(&raiz, &estado, nix.as_os_str(), nix_store.as_os_str())
            .expect("el preview nuevo debería quedar");

        assert_eq!(preview.generacion, PathBuf::from(nuevo));
        assert_eq!(
            fs::read_link(&enlace).expect("debería apuntar al nuevo preview"),
            preview.generacion
        );

        let _ = fs::remove_dir_all(&carpeta);
    }

    #[test]
    fn una_configuracion_cambiada_invalida_el_preview() {
        let carpeta = temporal("cambio-configuracion");
        let raiz = carpeta.join("repo");
        let estado = carpeta.join("estado");
        fs::create_dir_all(&raiz).expect("debería crear la prueba");
        fs::create_dir_all(&estado).expect("debería crear el estado");
        fs::write(raiz.join("configuracion.toml"), "nombre = \"antes\"\n")
            .expect("debería crear la configuración");

        let generacion = "/nix/store/dddddddddddddddddddddddddddddddd-nixos-system-korunix";
        symlink(generacion, estado.join("preview")).expect("debería crear el enlace");
        fs::write(estado.join(ARCHIVO_GENERACION), format!("{generacion}\n"))
            .expect("debería guardar la generación");
        fs::write(estado.join(ARCHIVO_CONFIGURACION), b"nombre = \"antes\"\n")
            .expect("debería guardar la configuración");

        fs::write(raiz.join("configuracion.toml"), "nombre = \"después\"\n")
            .expect("debería cambiar la configuración");

        let error = leer_en(&raiz, &estado).expect_err("el preview debería quedar viejo");
        assert!(error.contains("cambió después del preview"));

        let _ = fs::remove_dir_all(&carpeta);
    }

    #[test]
    fn un_fallo_al_proteger_el_nuevo_restaura_el_preview_anterior() {
        let carpeta = temporal("restaura-root");
        let raiz = carpeta.join("repo");
        let estado = carpeta.join("estado");
        fs::create_dir_all(&raiz).expect("debería crear la prueba");
        fs::create_dir_all(&estado).expect("debería crear el estado");

        let anterior = "/nix/store/eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee-preview-anterior";
        let nuevo = "/nix/store/ffffffffffffffffffffffffffffffff-preview-nuevo";
        symlink(anterior, estado.join("preview")).expect("debería crear el preview anterior");

        let nix = nix_que_crea(&carpeta, nuevo);
        let contador = carpeta.join("contador");
        let nix_store = programa(
            &carpeta,
            "nix-store-falla-una-vez.sh",
            &format!(
                "#!/bin/sh\nset -eu\ncontador='{}'\nif [ ! -e \"$contador\" ]; then touch \"$contador\"; exit 1; fi\nenlace=\"$2\"\ndestino=\"$5\"\nln -s \"$destino\" \"$enlace\"\n",
                contador.display()
            ),
        );

        let _error = construir_en(&raiz, &estado, nix.as_os_str(), nix_store.as_os_str())
            .expect_err("la raíz nueva debería fallar");

        assert!(
            contador.exists(),
            "nix-store debía haber llegado a intentar registrar la raíz nueva"
        );
        assert_eq!(
            fs::read_link(estado.join("preview")).unwrap(),
            PathBuf::from(anterior)
        );

        let _ = fs::remove_dir_all(&carpeta);
    }

    #[test]
    fn un_lock_cambiado_invalida_el_preview_antes_de_aplicar() {
        let carpeta = temporal("cambio-lock");
        let raiz = carpeta.join("repo");
        let estado = carpeta.join("estado");
        fs::create_dir_all(&raiz).unwrap();
        fs::create_dir_all(&estado).unwrap();

        let generacion = "/nix/store/99999999999999999999999999999999-nixos-system-korunix";
        symlink(generacion, estado.join("preview")).unwrap();
        fs::write(estado.join(ARCHIVO_GENERACION), format!("{generacion}\n")).unwrap();
        fs::write(raiz.join("configuracion.toml"), b"nombre = \"prueba\"\n").unwrap();
        fs::write(estado.join(ARCHIVO_CONFIGURACION), b"nombre = \"prueba\"\n").unwrap();
        fs::write(estado.join(ARCHIVO_LOCK_BASE), b"lock-base").unwrap();
        fs::write(estado.join(ARCHIVO_LOCK_USADO), b"lock-base").unwrap();
        fs::write(raiz.join("flake.lock"), b"lock-distinto").unwrap();

        let error = leer_en(&raiz, &estado).expect_err("el lock cambiado debe invalidarlo");
        assert!(error.contains("flake.lock cambió después del preview"));

        let _ = fs::remove_dir_all(&carpeta);
    }

    #[test]
    fn distingue_un_preview_que_usa_un_lock_candidato() {
        let carpeta = temporal("lock-candidato");
        let estado = carpeta.join("estado");
        fs::create_dir_all(&estado).unwrap();

        fs::write(estado.join(ARCHIVO_LOCK_BASE), b"base").unwrap();
        fs::write(estado.join(ARCHIVO_LOCK_USADO), b"candidato").unwrap();

        let (base, usado) = locks_guardados(&estado).unwrap();
        assert_ne!(base, usado);

        let _ = fs::remove_dir_all(&carpeta);
    }
}

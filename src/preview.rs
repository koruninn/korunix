use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

const GENERACION_NIXOS: &str = ".#nixosConfigurations.korunix.config.system.build.toplevel";

#[derive(Debug)]
pub struct Preview {
    pub generacion: PathBuf,
    pub enlace: PathBuf,
}

fn carpeta_estado() -> Result<PathBuf, String> {
    if let Some(ruta) = env::var_os("XDG_STATE_HOME") {
        return Ok(PathBuf::from(ruta).join("korunix"));
    }

    if let Some(home) = env::var_os("HOME") {
        return Ok(PathBuf::from(home).join(".local/state/korunix"));
    }

    Err("No pude saber dónde guardar el preview.".to_string())
}

fn estado_enlace(ruta: &Path) -> Result<bool, String> {
    match fs::symlink_metadata(ruta) {
        Ok(datos) if datos.file_type().is_symlink() || datos.is_file() => Ok(true),
        Ok(_) => Err(format!(
            "No voy a reemplazar {} porque no es un enlace de preview.",
            ruta.display()
        )),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(format!(
            "No pude revisar el preview anterior.\nDetalle: {error}"
        )),
    }
}

fn limpiar_temporal(ruta: &Path) {
    if matches!(fs::symlink_metadata(ruta), Ok(datos) if datos.file_type().is_symlink() || datos.is_file())
    {
        let _ = fs::remove_file(ruta);
    }
}

fn pedir_generacion(raiz: &Path, enlace: &Path, programa: &OsStr) -> Result<PathBuf, String> {
    let enlace_texto = enlace.to_string_lossy().into_owned();
    let resultado = Command::new(programa)
        .args([
            "build",
            "--out-link",
            enlace_texto.as_str(),
            GENERACION_NIXOS,
        ])
        .current_dir(raiz)
        .status()
        .map_err(|error| format!("No pude pedirle el preview a Nix.\nDetalle: {error}"))?;

    if !resultado.success() {
        return Err("Nix no pudo construir el preview.".to_string());
    }

    let generacion = fs::read_link(enlace)
        .map_err(|error| format!("No pude leer la generación del preview.\nDetalle: {error}"))?;

    if !generacion.is_absolute() || !generacion.starts_with("/nix/store") {
        return Err(format!(
            "Nix dejó el preview en una ruta que no esperaba: {}",
            generacion.display()
        ));
    }

    Ok(generacion)
}

fn construir_en(raiz: &Path, estado: &Path, programa: &OsStr) -> Result<Preview, String> {
    fs::create_dir_all(estado)
        .map_err(|error| format!("No pude preparar la carpeta del preview.\nDetalle: {error}"))?;

    let enlace = estado.join("preview");
    let habia_preview = estado_enlace(&enlace)?;

    if !habia_preview {
        let generacion = match pedir_generacion(raiz, &enlace, programa) {
            Ok(generacion) => generacion,
            Err(error) => {
                limpiar_temporal(&enlace);
                return Err(error);
            }
        };

        return Ok(Preview { generacion, enlace });
    }

    // El preview anterior sigue en su sitio mientras Nix prepara el nuevo.
    let temporal = estado.join(format!(".preview-nuevo-{}", process::id()));
    limpiar_temporal(&temporal);

    let generacion = match pedir_generacion(raiz, &temporal, programa) {
        Ok(generacion) => generacion,
        Err(error) => {
            limpiar_temporal(&temporal);
            return Err(error);
        }
    };

    // En Linux, rename reemplaza el enlace anterior de una sola vez. La raíz de
    // GC de "preview" sigue apuntando a ese nombre y pasa a proteger lo nuevo.
    if let Err(error) = fs::rename(&temporal, &enlace) {
        limpiar_temporal(&temporal);
        return Err(format!(
            "La generación nueva está construida, pero no pude reemplazar el preview anterior.\n\
             El preview anterior sigue siendo el válido.\nDetalle: {error}"
        ));
    }

    Ok(Preview { generacion, enlace })
}

pub fn crear(raiz: &Path) -> Result<Preview, String> {
    let estado = carpeta_estado()?;
    let programa = env::var_os("KORUNIX_NIX_BIN").unwrap_or_else(|| "nix".into());

    construir_en(raiz, &estado, programa.as_os_str())
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

    fn programa(carpeta: &Path, cuerpo: &str) -> PathBuf {
        let ruta = carpeta.join("nix-de-prueba.sh");

        fs::write(&ruta, cuerpo).expect("debería escribir el programa de prueba");

        let mut permisos = fs::metadata(&ruta)
            .expect("debería leer los permisos")
            .permissions();
        permisos.set_mode(0o755);
        fs::set_permissions(&ruta, permisos).expect("debería hacer ejecutable el programa");

        ruta
    }

    fn nix_que_crea(carpeta: &Path, generacion: &str) -> PathBuf {
        programa(
            carpeta,
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

    #[test]
    fn el_primer_preview_conserva_la_generacion_exacta() {
        let carpeta = temporal("primero");
        let raiz = carpeta.join("repo");
        let estado = carpeta.join("estado");
        fs::create_dir_all(&raiz).expect("debería crear la prueba");

        let esperada = "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-nixos-system-korunix";
        let nix = nix_que_crea(&carpeta, esperada);

        let preview =
            construir_en(&raiz, &estado, nix.as_os_str()).expect("el preview debería construirse");

        assert_eq!(preview.enlace, estado.join("preview"));
        assert_eq!(preview.generacion, PathBuf::from(esperada));
        assert_eq!(
            fs::read_link(&preview.enlace).expect("debería leer el enlace"),
            preview.generacion
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

        let nix = programa(
            &carpeta,
            r#"#!/bin/sh
exit 1
"#,
        );

        let error = construir_en(&raiz, &estado, nix.as_os_str())
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

        let preview =
            construir_en(&raiz, &estado, nix.as_os_str()).expect("el preview nuevo debería quedar");

        assert_eq!(preview.generacion, PathBuf::from(nuevo));
        assert_eq!(
            fs::read_link(&enlace).expect("debería apuntar al nuevo preview"),
            PathBuf::from(nuevo)
        );
        assert!(!estado
            .join(format!(".preview-nuevo-{}", process::id()))
            .exists());

        let _ = fs::remove_dir_all(&carpeta);
    }
}

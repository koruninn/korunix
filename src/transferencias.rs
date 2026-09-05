use crate::{almacenamiento, configuracion};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const BLOQUE: usize = 8 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct Progreso {
    pub copiados: u64,
    pub total: u64,
    pub bytes_por_segundo: f64,
    pub faltan: Option<Duration>,
}

fn cantidad_humana(bytes: f64) -> String {
    const KB: f64 = 1_000.0;
    const MB: f64 = 1_000_000.0;
    const GB: f64 = 1_000_000_000.0;

    if bytes >= GB {
        format!("{:.1} GB", bytes / GB)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes / MB)
    } else if bytes >= KB {
        format!("{:.1} kB", bytes / KB)
    } else {
        format!("{:.0} B", bytes)
    }
}

fn tiempo_humano(segundos: u64) -> String {
    if segundos < 60 {
        format!("{segundos} s")
    } else if segundos < 3600 {
        format!("{} min {} s", segundos / 60, segundos % 60)
    } else {
        format!("{} h {} min", segundos / 3600, (segundos % 3600) / 60)
    }
}

impl Progreso {
    pub fn linea(&self) -> String {
        let porcentaje = if self.total == 0 {
            100
        } else {
            self.copiados.saturating_mul(100) / self.total
        };

        let velocidad = if self.bytes_por_segundo > 0.0 {
            format!(" · {}/s", cantidad_humana(self.bytes_por_segundo))
        } else {
            String::new()
        };

        let eta = self
            .faltan
            .map(|duracion| format!(" · faltan {}", tiempo_humano(duracion.as_secs())))
            .unwrap_or_default();

        format!(
            "{porcentaje}% · {} de {}{velocidad}{eta}",
            cantidad_humana(self.copiados as f64),
            cantidad_humana(self.total as f64),
        )
    }
}

fn temporal_para(carpeta: &Path, nombre: &str) -> PathBuf {
    let momento = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    carpeta.join(format!(
        ".{nombre}.korunix-parcial-{}-{momento}",
        process::id()
    ))
}

fn copiar_a_carpeta<F>(origen: &Path, carpeta: &Path, mut progreso: F) -> Result<PathBuf, String>
where
    F: FnMut(Progreso),
{
    let datos = fs::metadata(origen).map_err(|error| {
        format!(
            "No pude leer el archivo «{}».\nDetalle: {error}",
            origen.display()
        )
    })?;

    if !datos.is_file() {
        return Err(format!(
            "«{}» no es un archivo normal. En este primer corte copia un archivo a la vez.",
            origen.display()
        ));
    }

    let nombre_os = origen.file_name().ok_or_else(|| {
        format!(
            "No pude obtener el nombre del archivo «{}».",
            origen.display()
        )
    })?;
    let nombre = nombre_os.to_string_lossy();

    if nombre.is_empty() || nombre == "." || nombre == ".." {
        return Err("El archivo no tiene un nombre que pueda copiar con seguridad.".to_string());
    }

    let destino = carpeta.join(nombre_os);

    if destino.exists() {
        return Err(format!(
            "Ya existe «{}» en la unidad. No voy a sobrescribirlo.",
            nombre
        ));
    }

    let origen_canonico = fs::canonicalize(origen).map_err(|error| {
        format!(
            "No pude comprobar la ruta de «{}».\nDetalle: {error}",
            origen.display()
        )
    })?;

    if let Ok(destino_canonico) = fs::canonicalize(&destino) {
        if destino_canonico == origen_canonico {
            return Err("El origen y el destino son el mismo archivo.".to_string());
        }
    }

    let total = datos.len();
    let temporal = temporal_para(carpeta, &nombre);
    let resultado = (|| {
        let mut entrada = File::open(origen).map_err(|error| {
            format!(
                "No pude abrir «{}» para leerlo.\nDetalle: {error}",
                origen.display()
            )
        })?;
        let mut salida = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporal)
            .map_err(|error| {
                format!("No pude preparar una copia temporal en la unidad.\nDetalle: {error}")
            })?;

        let inicio = Instant::now();
        let mut ultimo_aviso = inicio.checked_sub(Duration::from_secs(1)).unwrap_or(inicio);
        let mut copiados = 0u64;
        let mut buffer = vec![0u8; BLOQUE];

        progreso(Progreso {
            copiados: 0,
            total,
            bytes_por_segundo: 0.0,
            faltan: None,
        });

        loop {
            let leidos = entrada.read(&mut buffer).map_err(|error| {
                format!(
                    "La lectura de «{}» se interrumpió.\nDetalle: {error}",
                    origen.display()
                )
            })?;

            if leidos == 0 {
                break;
            }

            salida.write_all(&buffer[..leidos]).map_err(|error| {
                format!("La escritura en la unidad se interrumpió.\nDetalle: {error}")
            })?;
            copiados = copiados.saturating_add(leidos as u64);

            let ahora = Instant::now();

            if copiados < total && ahora.duration_since(ultimo_aviso) >= Duration::from_millis(250)
            {
                let transcurrido = ahora.duration_since(inicio).as_secs_f64();
                let velocidad = if transcurrido > 0.0 {
                    copiados as f64 / transcurrido
                } else {
                    0.0
                };
                let faltan = if velocidad > 0.0 && transcurrido >= 0.5 {
                    Some(Duration::from_secs_f64(
                        (total - copiados) as f64 / velocidad,
                    ))
                } else {
                    None
                };

                progreso(Progreso {
                    copiados,
                    total,
                    bytes_por_segundo: velocidad,
                    faltan,
                });
                ultimo_aviso = ahora;
            }
        }

        if copiados != total {
            return Err(format!(
                "La copia quedó incompleta: escribí {copiados} bytes de {total}."
            ));
        }

        salida.sync_all().map_err(|error| {
            format!("No pude terminar de guardar el archivo.\nDetalle: {error}")
        })?;
        drop(salida);

        let escritos = fs::metadata(&temporal)
            .map_err(|error| format!("No pude verificar la copia temporal.\nDetalle: {error}"))?
            .len();

        if escritos != total {
            return Err(format!(
                "La copia temporal mide {escritos} bytes, pero el origen mide {total}."
            ));
        }

        if destino.exists() {
            return Err(format!(
                "Apareció otro archivo llamado «{}» mientras copiaba. No voy a sobrescribirlo.",
                nombre
            ));
        }

        let reserva = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&destino)
            .map_err(|error| {
                format!(
                    "No pude reservar el nombre final «{}» sin sobrescribir nada.\nDetalle: {error}",
                    nombre
                )
            })?;
        drop(reserva);

        if let Err(error) = fs::rename(&temporal, &destino) {
            let _ = fs::remove_file(&destino);
            return Err(format!(
                "La copia se escribió, pero no pude darle su nombre final.\nDetalle: {error}"
            ));
        }

        let final_tamano = fs::metadata(&destino)
            .map_err(|error| format!("No pude verificar el archivo terminado.\nDetalle: {error}"))?
            .len();

        if final_tamano != total {
            let _ = fs::remove_file(&destino);
            return Err(format!(
                "El archivo final mide {final_tamano} bytes, pero debía medir {total}."
            ));
        }

        let transcurrido = inicio.elapsed().as_secs_f64();
        let velocidad = if transcurrido > 0.0 {
            total as f64 / transcurrido
        } else {
            0.0
        };

        progreso(Progreso {
            copiados: total,
            total,
            bytes_por_segundo: velocidad,
            faltan: None,
        });

        Ok(destino.clone())
    })();

    if resultado.is_err() {
        let _ = fs::remove_file(&temporal);
    }

    resultado
}

fn nombre_unidad_systemd(ruta: &Path, tipo: &str) -> Option<String> {
    let texto = ruta.to_str()?.trim_matches('/');

    if texto.is_empty()
        || !texto.bytes().all(|caracter| {
            caracter.is_ascii_alphanumeric() || caracter == b'/' || caracter == b'_'
        })
    {
        return None;
    }

    Some(format!("{}.{}", texto.replace('/', "-"), tipo))
}

fn ruta_aplicada(ruta: &Path) -> Result<bool, String> {
    let montajes = fs::read_to_string("/proc/self/mountinfo")
        .map_err(|error| format!("No pude comprobar los montajes activos.\nDetalle: {error}"))?;
    let buscada = ruta.to_string_lossy();

    if montajes.lines().any(|linea| {
        linea
            .split_whitespace()
            .nth(4)
            .is_some_and(|montaje| montaje == buscada)
    }) {
        return Ok(true);
    }

    let Some(montaje) = nombre_unidad_systemd(ruta, "mount") else {
        return Ok(false);
    };
    let Some(automontaje) = nombre_unidad_systemd(ruta, "automount") else {
        return Ok(false);
    };

    for carpeta in [
        "/run/systemd/generator",
        "/run/systemd/generator.early",
        "/run/systemd/generator.late",
    ] {
        let carpeta = Path::new(carpeta);

        if carpeta.join(&montaje).exists() || carpeta.join(&automontaje).exists() {
            return Ok(true);
        }
    }

    Ok(false)
}

pub fn transferir<F>(
    raiz: &Path,
    unidad: &str,
    origen: &Path,
    progreso: F,
) -> Result<PathBuf, String>
where
    F: FnMut(Progreso),
{
    let configuracion = configuracion::leer(&raiz.join("configuracion.toml"))?;

    if !configuracion
        .almacenamiento
        .disponibles
        .iter()
        .any(|nombre| nombre == unidad)
    {
        return Err(format!(
            "«{unidad}» no está disponible en Korunix. Actívala y aplica ese cambio antes de copiar."
        ));
    }

    let carpeta = almacenamiento::ruta_administrada(raiz, unidad)?;

    if !ruta_aplicada(&carpeta)? {
        return Err(format!(
            "«{unidad}» está elegida, pero todavía no está activa en NixOS. Crea un preview y aplica ese cambio antes de transferir archivos."
        ));
    }

    copiar_a_carpeta(origen, &carpeta, progreso)
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use std::env;

    fn carpeta_prueba(nombre: &str) -> PathBuf {
        let momento = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        env::temp_dir().join(format!(
            "korunix-transferencia-{nombre}-{}-{momento}",
            process::id()
        ))
    }

    #[test]
    fn convierte_la_ruta_interna_en_unidad_systemd() {
        assert_eq!(
            nombre_unidad_systemd(Path::new("/mnt/korunix/baf1579a"), "automount").as_deref(),
            Some("mnt-korunix-baf1579a.automount")
        );
        assert_eq!(
            nombre_unidad_systemd(Path::new("/mnt/datos"), "mount").as_deref(),
            Some("mnt-datos.mount")
        );
    }

    #[test]
    fn copia_y_solo_publica_el_nombre_final_al_terminar() {
        let raiz = carpeta_prueba("copia");
        let origen_dir = raiz.join("origen");
        let destino_dir = raiz.join("destino");
        fs::create_dir_all(&origen_dir).expect("debería crear origen");
        fs::create_dir_all(&destino_dir).expect("debería crear destino");

        let origen = origen_dir.join("imagen.iso");
        let contenido = vec![0x5a; BLOQUE + 4096];
        fs::write(&origen, &contenido).expect("debería crear el archivo");

        let mut avances = Vec::new();
        let destino = copiar_a_carpeta(&origen, &destino_dir, |avance| {
            avances.push(avance.linea());
        })
        .expect("debería copiar");

        assert_eq!(destino, destino_dir.join("imagen.iso"));
        assert_eq!(fs::read(&destino).expect("debería leer destino"), contenido);
        assert!(avances
            .last()
            .is_some_and(|linea| linea.starts_with("100%")));
        assert!(avances[..avances.len() - 1]
            .iter()
            .all(|linea| !linea.starts_with("100%")));
        assert!(!destino_dir
            .read_dir()
            .expect("debería listar destino")
            .any(|entrada| entrada
                .expect("entrada válida")
                .file_name()
                .to_string_lossy()
                .contains("korunix-parcial")));

        fs::remove_dir_all(&raiz).expect("debería limpiar la prueba");
    }

    #[test]
    fn no_sobrescribe_un_archivo_existente() {
        let raiz = carpeta_prueba("sobrescribir");
        let origen_dir = raiz.join("origen");
        let destino_dir = raiz.join("destino");
        fs::create_dir_all(&origen_dir).expect("debería crear origen");
        fs::create_dir_all(&destino_dir).expect("debería crear destino");

        let origen = origen_dir.join("mismo.bin");
        let destino = destino_dir.join("mismo.bin");
        fs::write(&origen, b"nuevo").expect("debería crear origen");
        fs::write(&destino, b"existente").expect("debería crear destino");

        let error = copiar_a_carpeta(&origen, &destino_dir, |_| {})
            .expect_err("debería negarse a sobrescribir");

        assert!(error.contains("No voy a sobrescribirlo"));
        assert_eq!(
            fs::read(&destino).expect("debería conservar destino"),
            b"existente"
        );

        fs::remove_dir_all(&raiz).expect("debería limpiar la prueba");
    }
}

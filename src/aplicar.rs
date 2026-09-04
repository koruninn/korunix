use crate::preview;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{self, Command};

const SISTEMA_ACTIVO: &str = "/run/current-system";
const PERFIL_SISTEMA: &str = "/nix/var/nix/profiles/system";
const SUDO_NIXOS: &str = "/run/wrappers/bin/sudo";

#[derive(Debug, PartialEq, Eq)]
enum Situacion {
    YaAplicado,
    Listo,
}

#[derive(Debug)]
struct Preparado {
    preview: PathBuf,
    anterior: PathBuf,
    enlace_anterior: PathBuf,
    situacion: Situacion,
}

fn destino(ruta: &Path) -> Result<PathBuf, String> {
    fs::canonicalize(ruta).map_err(|error| {
        format!(
            "No pude saber a qué generación apunta {}.\nDetalle: {error}",
            ruta.display()
        )
    })
}

fn clasificar_estado(
    activa: &Path,
    persistente: &Path,
    preview: &Path,
) -> Result<Situacion, String> {
    if activa != persistente {
        return Err(format!(
            "La generación activa y la persistente no coinciden.\n\
             Activa: {}\nPersistente: {}\n\
             No voy a aplicar nada hasta resolver ese estado parcial.",
            activa.display(),
            persistente.display()
        ));
    }

    if activa == preview {
        Ok(Situacion::YaAplicado)
    } else {
        Ok(Situacion::Listo)
    }
}

fn revisar_programa(ruta: PathBuf) -> Result<PathBuf, String> {
    let datos = fs::metadata(&ruta).map_err(|error| {
        format!(
            "No encontré el programa necesario {}.\nDetalle: {error}",
            ruta.display()
        )
    })?;

    if !datos.is_file() || datos.permissions().mode() & 0o111 == 0 {
        return Err(format!("{} no es un programa ejecutable.", ruta.display()));
    }

    if ruta.is_absolute() {
        Ok(ruta)
    } else {
        env::current_dir()
            .map(|actual| actual.join(ruta))
            .map_err(|error| format!("No pude resolver la carpeta actual.\nDetalle: {error}"))
    }
}

fn programa_variable(nombre: &str, normal: &str) -> Result<PathBuf, String> {
    let ruta = env::var_os(nombre)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(normal));

    // No se usa canonicalize aquí. Programas como nix-store y nix-env pueden ser
    // enlaces a un ejecutable multicall y el nombre con el que se invocan importa.
    revisar_programa(ruta)
}

fn sudo_nixos() -> Result<PathBuf, String> {
    if let Some(ruta) = env::var_os("KORUNIX_SUDO_BIN") {
        return Ok(PathBuf::from(ruta));
    }

    let ruta = PathBuf::from(SUDO_NIXOS);
    let datos = fs::metadata(&ruta).map_err(|error| {
        format!("No encontré la autorización normal de NixOS en {SUDO_NIXOS}.\nDetalle: {error}")
    })?;

    if !datos.is_file()
        || datos.permissions().mode() & 0o111 == 0
        || datos.permissions().mode() & 0o4000 == 0
        || datos.uid() != 0
    {
        return Err(format!(
            "{SUDO_NIXOS} no tiene los permisos de autorización que esperaba."
        ));
    }

    Ok(ruta)
}

fn salida(programa: &Path, argumentos: &[&str]) -> Result<String, String> {
    let resultado = Command::new(programa)
        .args(argumentos)
        .output()
        .map_err(|error| format!("No pude ejecutar {}.\nDetalle: {error}", programa.display()))?;

    if !resultado.status.success() {
        return Err(format!("{} devolvió un error.", programa.display()));
    }

    String::from_utf8(resultado.stdout).map_err(|error| {
        format!(
            "{} devolvió texto que no pude leer: {error}",
            programa.display()
        )
    })
}

fn esta_protegida(nix_store: &Path, generacion: &Path, enlace: &Path) -> Result<bool, String> {
    let generacion_texto = generacion.to_string_lossy();
    let texto = salida(
        nix_store,
        &["--query", "--roots", generacion_texto.as_ref()],
    )?;
    let enlace_texto = enlace.to_string_lossy();

    Ok(texto
        .lines()
        .any(|linea| linea.contains(enlace_texto.as_ref())))
}

fn registrar_raiz(nix_store: &Path, enlace: &Path, generacion: &Path) -> Result<(), String> {
    let anterior = match fs::symlink_metadata(enlace) {
        Ok(datos) if datos.file_type().is_symlink() || datos.is_file() => {
            fs::read_link(enlace).ok()
        }
        Ok(_) => {
            return Err(format!(
                "No voy a reemplazar {} porque no es un enlace de Korunix.",
                enlace.display()
            ));
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => {
            return Err(format!(
                "No pude revisar {}.\nDetalle: {error}",
                enlace.display()
            ));
        }
    };

    if enlace.exists() || fs::symlink_metadata(enlace).is_ok() {
        fs::remove_file(enlace).map_err(|error| {
            format!(
                "No pude preparar {} para guardar el punto de regreso.\nDetalle: {error}",
                enlace.display()
            )
        })?;
    }

    let resultado = Command::new(nix_store)
        .arg("--add-root")
        .arg(enlace)
        .arg("--indirect")
        .arg("--realise")
        .arg(generacion)
        .status();

    let fallo = match resultado {
        Ok(estado) if estado.success() => None,
        Ok(_) => Some("Nix devolvió un error al registrar el punto de regreso.".to_string()),
        Err(error) => Some(format!(
            "No pude proteger la generación anterior.\nDetalle: {error}"
        )),
    };

    if let Some(fallo) = fallo {
        if let Some(anterior) = anterior {
            let _ = fs::remove_file(enlace);
            let _ = Command::new(nix_store)
                .arg("--add-root")
                .arg(enlace)
                .arg("--indirect")
                .arg("--realise")
                .arg(anterior)
                .status();
        }

        return Err(format!(
            "Nix no pudo proteger la generación anterior.\n{fallo}"
        ));
    }

    let guardada = fs::read_link(enlace).map_err(|error| {
        format!("Nix creó el punto de regreso, pero no pude leerlo.\nDetalle: {error}")
    })?;

    if guardada != generacion {
        return Err(format!(
            "El punto de regreso no apunta a la generación esperada.\n\
             Esperada: {}\nEncontrada: {}",
            generacion.display(),
            guardada.display()
        ));
    }

    if !esta_protegida(nix_store, generacion, enlace)? {
        return Err(
            "El punto de regreso existe, pero Nix no lo reconoce como raíz de GC.".to_string(),
        );
    }

    Ok(())
}

fn preparar(raiz: &Path) -> Result<Preparado, String> {
    let preview = preview::leer(raiz)?;
    let nix_store = programa_variable(
        "KORUNIX_NIX_STORE_BIN",
        "/run/current-system/sw/bin/nix-store",
    )?;

    if !esta_protegida(&nix_store, &preview.generacion, &preview.enlace)? {
        return Err(format!(
            "El preview {} no está protegido frente al recolector de basura.\n\
             Crea de nuevo el preview antes de aplicar.",
            preview.generacion.display()
        ));
    }

    let activa = destino(Path::new(SISTEMA_ACTIVO))?;
    let persistente = destino(Path::new(PERFIL_SISTEMA))?;
    let situacion = clasificar_estado(&activa, &persistente, &preview.generacion)?;
    let estado = preview::carpeta_estado()?;
    let enlace_anterior = estado.join("anterior");

    Ok(Preparado {
        preview: preview.generacion,
        anterior: activa,
        enlace_anterior,
        situacion,
    })
}

fn unidades_fallidas(systemctl: &Path) -> Result<BTreeSet<String>, String> {
    let resultado = Command::new(systemctl)
        .args(["--failed", "--no-legend", "--plain", "--no-pager"])
        .output()
        .map_err(|error| format!("No pude revisar las unidades fallidas.\nDetalle: {error}"))?;

    if !resultado.status.success() {
        return Err("systemctl no pudo revisar las unidades fallidas.".to_string());
    }

    let texto = String::from_utf8(resultado.stdout)
        .map_err(|error| format!("No pude leer la lista de unidades fallidas: {error}"))?;

    Ok(texto
        .lines()
        .filter_map(|linea| linea.split_whitespace().next())
        .map(str::to_string)
        .collect())
}

fn kernel_actual() -> String {
    fs::read_to_string("/proc/sys/kernel/osrelease")
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn kernel_generacion(generacion: &Path) -> Option<String> {
    let carpeta = generacion.join("kernel-modules/lib/modules");
    let entradas = fs::read_dir(carpeta).ok()?;

    for entrada in entradas.flatten() {
        if entrada.file_type().ok()?.is_dir() {
            return Some(entrada.file_name().to_string_lossy().into_owned());
        }
    }

    None
}

fn marcar(ruta: &Path, valor: &str) {
    let _ = fs::write(ruta, format!("{valor}\n"));
}

fn es_root() -> bool {
    let Ok(texto) = fs::read_to_string("/proc/self/status") else {
        return false;
    };

    texto
        .lines()
        .find(|linea| linea.starts_with("Uid:"))
        .is_some_and(|linea| {
            linea
                .split_whitespace()
                .nth(2)
                .is_some_and(|uid| uid == "0")
        })
}

fn ejecutar_accion(generacion: &Path, accion: &str) -> Result<(), String> {
    let programa = generacion.join("bin/switch-to-configuration");
    let resultado = Command::new(&programa)
        .arg(accion)
        .status()
        .map_err(|error| {
            format!(
                "No pude ejecutar {} {accion}.\nDetalle: {error}",
                programa.display()
            )
        })?;

    if resultado.success() {
        Ok(())
    } else {
        Err(format!("NixOS no pudo completar «{accion}»."))
    }
}

fn fijar_perfil(nix_env: &Path, perfil: &Path, generacion: &Path) -> Result<(), String> {
    let resultado = Command::new(nix_env)
        .arg("-p")
        .arg(perfil)
        .arg("--set")
        .arg(generacion)
        .status()
        .map_err(|error| format!("No pude cambiar el perfil persistente.\nDetalle: {error}"))?;

    if resultado.success() {
        Ok(())
    } else {
        Err("Nix no pudo cambiar el perfil persistente.".to_string())
    }
}

fn recuperar(
    anterior: &Path,
    nix_env: &Path,
    perfil: &Path,
    sistema_activo: &Path,
) -> Result<(), String> {
    eprintln!();
    eprintln!("El cambio no terminó bien. Recuperando la generación anterior...");

    fijar_perfil(nix_env, perfil, anterior)?;
    ejecutar_accion(anterior, "switch")?;

    let activa = destino(sistema_activo)?;
    let persistente = destino(perfil)?;

    if activa != anterior || persistente != anterior {
        return Err(format!(
            "La recuperación no terminó en la generación anterior.\n\
             Activa: {}\nPersistente: {}",
            activa.display(),
            persistente.display()
        ));
    }

    eprintln!("✓ La generación anterior volvió a quedar activa y persistente.");
    Ok(())
}

fn aplicar_generacion(
    preview: &Path,
    anterior: &Path,
    nix_env: &Path,
    perfil: &Path,
    sistema_activo: &Path,
) -> Result<(), String> {
    println!("Fase 1/2 · dejando exactamente el preview como generación persistente...");

    if let Err(error) = fijar_perfil(nix_env, perfil, preview) {
        let recuperacion = recuperar(anterior, nix_env, perfil, sistema_activo);
        return match recuperacion {
            Ok(()) => Err(format!("{error}\nEl sistema quedó recuperado en la generación anterior.")),
            Err(error_recuperacion) => Err(format!(
                "{error}\nERROR CRÍTICO: tampoco pude completar la recuperación.\n{error_recuperacion}"
            )),
        };
    }

    match destino(perfil) {
        Ok(persistente) if persistente == preview => {}
        Ok(persistente) => {
            let error = format!(
                "El perfil persistente no quedó en el preview.\nEncontrado: {}",
                persistente.display()
            );
            let recuperacion = recuperar(anterior, nix_env, perfil, sistema_activo);
            return match recuperacion {
                Ok(()) => Err(format!(
                    "{error}\nEl sistema quedó recuperado en la generación anterior."
                )),
                Err(error_recuperacion) => Err(format!(
                    "{error}\nERROR CRÍTICO: la recuperación también falló.\n{error_recuperacion}"
                )),
            };
        }
        Err(error) => {
            let recuperacion = recuperar(anterior, nix_env, perfil, sistema_activo);
            return match recuperacion {
                Ok(()) => Err(format!(
                    "{error}\nEl sistema quedó recuperado en la generación anterior."
                )),
                Err(error_recuperacion) => Err(format!(
                    "{error}\nERROR CRÍTICO: la recuperación también falló.\n{error_recuperacion}"
                )),
            };
        }
    }

    println!("Fase 2/2 · activando esa misma generación...");

    if let Err(error) = ejecutar_accion(preview, "switch") {
        let recuperacion = recuperar(anterior, nix_env, perfil, sistema_activo);

        return match recuperacion {
            Ok(()) => Err(format!("{error}\nEl sistema quedó recuperado en la generación anterior.")),
            Err(error_recuperacion) => Err(format!(
                "{error}\nERROR CRÍTICO: tampoco pude completar la recuperación.\n{error_recuperacion}"
            )),
        };
    }

    let activa = destino(sistema_activo);
    let persistente = destino(perfil);

    if activa.as_deref() != Ok(preview) || persistente.as_deref() != Ok(preview) {
        let detalle = format!(
            "Apply no terminó con activa = persistente = preview.\nActiva: {}\nPersistente: {}",
            activa
                .as_ref()
                .map(|ruta| ruta.display().to_string())
                .unwrap_or_else(|error| format!("<error: {error}>")),
            persistente
                .as_ref()
                .map(|ruta| ruta.display().to_string())
                .unwrap_or_else(|error| format!("<error: {error}>"))
        );
        let recuperacion = recuperar(anterior, nix_env, perfil, sistema_activo);

        return match recuperacion {
            Ok(()) => Err(format!(
                "{detalle}\nEl sistema quedó recuperado en la generación anterior."
            )),
            Err(error_recuperacion) => Err(format!(
                "{detalle}\nERROR CRÍTICO: la recuperación también falló.\n{error_recuperacion}"
            )),
        };
    }

    Ok(())
}

pub fn ejecutar_como_root(argumentos: &[String]) -> i32 {
    if !es_root() {
        eprintln!("Esta parte de apply solo puede ejecutarla root.");
        return 90;
    }

    if argumentos.len() != 4 {
        eprintln!("La llamada interna de apply está incompleta.");
        return 91;
    }

    let preview = PathBuf::from(argumentos[0].as_str());
    let anterior = PathBuf::from(argumentos[1].as_str());
    let nix_env = PathBuf::from(argumentos[2].as_str());
    let resultado = PathBuf::from(argumentos[3].as_str());

    let activa = match destino(Path::new(SISTEMA_ACTIVO)) {
        Ok(ruta) => ruta,
        Err(error) => {
            eprintln!("{error}");
            marcar(&resultado, "sin-cambio");
            return 10;
        }
    };
    let persistente = match destino(Path::new(PERFIL_SISTEMA)) {
        Ok(ruta) => ruta,
        Err(error) => {
            eprintln!("{error}");
            marcar(&resultado, "sin-cambio");
            return 10;
        }
    };

    if activa != anterior || persistente != anterior {
        eprintln!("El sistema cambió antes de empezar la revisión privilegiada.");
        eprintln!("Activa:      {}", activa.display());
        eprintln!("Persistente: {}", persistente.display());
        marcar(&resultado, "sin-cambio");
        return 10;
    }

    println!("Comprobando si NixOS permite este cambio...");
    if let Err(error) = ejecutar_accion(&preview, "check") {
        eprintln!("{error}");
        marcar(&resultado, "sin-cambio");
        return 11;
    }

    println!();
    println!("Simulando la activación...");
    if let Err(error) = ejecutar_accion(&preview, "dry-activate") {
        eprintln!("{error}");
        marcar(&resultado, "sin-cambio");
        return 12;
    }

    println!();
    println!("Efecto del cambio");
    println!("Ahora:       {}", anterior.display());
    println!("Se aplicará: {}", preview.display());

    let kernel_ahora = kernel_actual();
    let kernel_nuevo = kernel_generacion(&preview);

    if !kernel_ahora.is_empty() {
        println!("Kernel en ejecución: {kernel_ahora}");
    }

    if let Some(kernel_nuevo) = &kernel_nuevo {
        println!("Kernel del preview:  {kernel_nuevo}");

        if *kernel_nuevo != kernel_ahora {
            println!("Reinicio: sí, para empezar a usar el kernel {kernel_nuevo}.");
        } else {
            println!("Reinicio: no por cambio de kernel.");
        }
    }

    println!(
        "Sesión: algunos ajustes de escritorio pueden completarse al volver a iniciar sesión."
    );
    println!("Persistencia: esta misma generación quedará elegida para los próximos arranques.");
    println!("Rollback: la generación actual ya está protegida como «anterior».");
    println!("NixOS todavía no cambió.");
    println!();
    print!("Escribe exactamente APLICAR para activar esta generación: ");
    let _ = io::stdout().flush();

    let mut respuesta = String::new();
    if io::stdin().read_line(&mut respuesta).is_err() || respuesta.trim() != "APLICAR" {
        println!("Cancelado. NixOS no cambió.");
        marcar(&resultado, "cancelado");
        return 0;
    }

    let activa = match destino(Path::new(SISTEMA_ACTIVO)) {
        Ok(ruta) => ruta,
        Err(error) => {
            eprintln!("{error}");
            marcar(&resultado, "sin-cambio");
            return 13;
        }
    };
    let persistente = match destino(Path::new(PERFIL_SISTEMA)) {
        Ok(ruta) => ruta,
        Err(error) => {
            eprintln!("{error}");
            marcar(&resultado, "sin-cambio");
            return 13;
        }
    };

    if activa != anterior || persistente != anterior {
        eprintln!("El sistema cambió entre la revisión y tu autorización.");
        marcar(&resultado, "sin-cambio");
        return 13;
    }

    match aplicar_generacion(
        &preview,
        &anterior,
        &nix_env,
        Path::new(PERFIL_SISTEMA),
        Path::new(SISTEMA_ACTIVO),
    ) {
        Ok(()) => {
            println!("✓ Apply terminó con activa = persistente = preview.");
            marcar(&resultado, "aplicado");
            0
        }
        Err(error) => {
            eprintln!("{error}");

            let activa = destino(Path::new(SISTEMA_ACTIVO)).ok();
            let persistente = destino(Path::new(PERFIL_SISTEMA)).ok();

            if activa.as_deref() == Some(anterior.as_path())
                && persistente.as_deref() == Some(anterior.as_path())
            {
                marcar(&resultado, "recuperado");
                14
            } else {
                marcar(&resultado, "critico");
                15
            }
        }
    }
}

pub fn ejecutar(raiz: &Path) -> Result<(), String> {
    let preparado = preparar(raiz)?;

    if preparado.situacion == Situacion::YaAplicado {
        println!("✓ Este preview ya está activo y persistente.");
        println!("Generación: {}", preparado.preview.display());
        println!("No cambié nada.");
        return Ok(());
    }

    let nix_store = programa_variable(
        "KORUNIX_NIX_STORE_BIN",
        "/run/current-system/sw/bin/nix-store",
    )?;

    println!("Protegiendo la generación actual para poder volver atrás...");
    registrar_raiz(&nix_store, &preparado.enlace_anterior, &preparado.anterior)?;
    println!(
        "✓ Regreso: {} → {}",
        preparado.enlace_anterior.display(),
        preparado.anterior.display()
    );

    let sudo = sudo_nixos()?;
    let systemd_run = programa_variable(
        "KORUNIX_SYSTEMD_RUN_BIN",
        "/run/current-system/sw/bin/systemd-run",
    )?;
    let systemctl = programa_variable(
        "KORUNIX_SYSTEMCTL_BIN",
        "/run/current-system/sw/bin/systemctl",
    )?;
    let nix_env = programa_variable("KORUNIX_NIX_ENV_BIN", "/run/current-system/sw/bin/nix-env")?;
    let ejecutable = env::current_exe().map_err(|error| {
        format!("No pude localizar el Korunix que estás usando.\nDetalle: {error}")
    })?;

    let fallidas_antes = unidades_fallidas(&systemctl)?;
    let estado = preview::carpeta_estado()?;
    let resultado = estado.join(format!(".aplicar-resultado-{}", process::id()));
    fs::write(&resultado, "iniciado\n")
        .map_err(|error| format!("No pude preparar el resultado de apply.\nDetalle: {error}"))?;

    println!();
    println!("NixOS necesita autorización para revisar el cambio.");
    println!("La contraseña, si aparece, se pide antes de dry-activate porque NixOS exige root incluso para esa simulación.");
    println!("La misma autorización se mantiene para apply; no se ejecutará nix build.");

    let unidad = format!("korunix-aplicar-{}", process::id());
    let estado_ejecucion = Command::new(&sudo)
        .arg(&systemd_run)
        .arg("--wait")
        .arg("--collect")
        .arg("--pipe")
        .arg("--quiet")
        .arg("--service-type=exec")
        .arg(format!("--unit={unidad}"))
        .arg(&ejecutable)
        .arg("__aplicar-raiz")
        .arg(&preparado.preview)
        .arg(&preparado.anterior)
        .arg(&nix_env)
        .arg(&resultado)
        .status()
        .map_err(|error| format!("No pude iniciar apply con autorización.\nDetalle: {error}"))?;

    let marca = fs::read_to_string(&resultado)
        .unwrap_or_else(|_| "desconocido".to_string())
        .trim()
        .to_string();
    let _ = fs::remove_file(&resultado);

    let activa = destino(Path::new(SISTEMA_ACTIVO))?;
    let persistente = destino(Path::new(PERFIL_SISTEMA))?;

    if marca == "cancelado" && activa == preparado.anterior && persistente == preparado.anterior {
        println!("No se aplicó nada.");
        return Ok(());
    }

    if activa == preparado.preview && persistente == preparado.preview {
        if !estado_ejecucion.success() {
            eprintln!(
                "Aviso: la conexión con la unidad devolvió {}, pero el estado final confirma que apply terminó.",
                estado_ejecucion
            );
        }
    } else if activa == preparado.anterior && persistente == preparado.anterior {
        return Err(format!(
            "Apply no se completó. El sistema quedó en la generación anterior.\n\
             Estado interno: {marca}\nUnidad: {unidad}"
        ));
    } else {
        return Err(format!(
            "ERROR CRÍTICO: activa y persistente quedaron en un estado parcial.\n\
             Activa: {}\nPersistente: {}\nUnidad: {unidad}",
            activa.display(),
            persistente.display()
        ));
    }

    if !esta_protegida(&nix_store, &preparado.anterior, &preparado.enlace_anterior)? {
        return Err(
            "Apply terminó, pero la generación anterior dejó de estar protegida para rollback."
                .to_string(),
        );
    }

    let fallidas_despues = unidades_fallidas(&systemctl)?;
    let nuevas: Vec<_> = fallidas_despues
        .difference(&fallidas_antes)
        .cloned()
        .collect();

    if !nuevas.is_empty() {
        return Err(format!(
            "El preview quedó activo y persistente, pero aparecieron unidades fallidas nuevas:\n  - {}\n\
             Esto debe revisarse antes de cerrar apply.",
            nuevas.join("\n  - ")
        ));
    }

    println!();
    println!("✓ Se aplicó exactamente el preview revisado.");
    println!("✓ No se reconstruyó NixOS.");
    println!("✓ Activa = persistente = {}", preparado.preview.display());
    println!("✓ La generación anterior sigue protegida para rollback.");
    println!("✓ No aparecieron nuevas unidades systemd fallidas.");

    let kernel_ahora = kernel_actual();
    if let Some(kernel_nuevo) = kernel_generacion(&preparado.preview) {
        if kernel_nuevo != kernel_ahora {
            println!("Reinicio pendiente: kernel actual {kernel_ahora} → {kernel_nuevo}.");
        }
    }

    Ok(())
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
            "korunix-aplicar-{nombre}-{}-{momento}",
            process::id()
        ))
    }

    fn programa(ruta: &Path, cuerpo: &str) {
        fs::write(ruta, cuerpo).expect("debería escribir el programa de prueba");
        let mut permisos = fs::metadata(ruta)
            .expect("debería leer permisos")
            .permissions();
        permisos.set_mode(0o755);
        fs::set_permissions(ruta, permisos).expect("debería hacerlo ejecutable");
    }

    #[test]
    fn conserva_el_nombre_de_un_programa_multicall() {
        let carpeta = temporal("multicall");
        fs::create_dir_all(&carpeta).expect("debería crear la prueba");

        let programa_real = carpeta.join("nix");
        programa(&programa_real, "#!/bin/sh\nexit 0\n");

        let enlace = carpeta.join("nix-store");
        symlink(&programa_real, &enlace).expect("debería crear el enlace multicall");

        let revisada =
            revisar_programa(enlace.clone()).expect("debería conservar el programa ejecutable");

        assert_eq!(revisada, enlace);
        let _ = fs::remove_dir_all(carpeta);
    }

    #[test]
    fn un_estado_dividido_se_rechaza() {
        let activa = Path::new("/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-activa");
        let persistente = Path::new("/nix/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-persistente");
        let preview = Path::new("/nix/store/cccccccccccccccccccccccccccccccc-preview");

        let error = clasificar_estado(activa, persistente, preview)
            .expect_err("el estado dividido debería rechazarse");

        assert!(error.contains("no coinciden"));
    }

    #[test]
    fn un_preview_ya_aplicado_es_idempotente() {
        let preview = Path::new("/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-preview");

        assert_eq!(
            clasificar_estado(preview, preview, preview).expect("debería reconocerlo"),
            Situacion::YaAplicado
        );
    }

    #[test]
    fn aplica_exactamente_la_generacion_indicada() {
        let carpeta = temporal("exacto");
        let anterior = carpeta.join("anterior");
        let preview = carpeta.join("preview");
        let activa = carpeta.join("activa");
        let perfil = carpeta.join("perfil");
        fs::create_dir_all(anterior.join("bin")).expect("debería crear anterior");
        fs::create_dir_all(preview.join("bin")).expect("debería crear preview");
        symlink(&anterior, &activa).expect("debería crear activa");
        symlink(&anterior, &perfil).expect("debería crear perfil");

        let nix_env = carpeta.join("nix-env");
        programa(
            &nix_env,
            &format!(
                "#!/bin/sh\nset -eu\nperfil=\"$2\"\ndestino=\"$4\"\nln -sfn \"$destino\" \"$perfil\"\n",
            ),
        );

        programa(
            &anterior.join("bin/switch-to-configuration"),
            &format!(
                "#!/bin/sh\nset -eu\n[ \"$1\" = switch ] && ln -sfn '{}' '{}'\n",
                anterior.display(),
                activa.display()
            ),
        );
        programa(
            &preview.join("bin/switch-to-configuration"),
            &format!(
                "#!/bin/sh\nset -eu\n[ \"$1\" = switch ] && ln -sfn '{}' '{}'\n",
                preview.display(),
                activa.display()
            ),
        );

        aplicar_generacion(&preview, &anterior, &nix_env, &perfil, &activa)
            .expect("apply debería funcionar");

        assert_eq!(destino(&activa).unwrap(), preview);
        assert_eq!(destino(&perfil).unwrap(), preview);
        let _ = fs::remove_dir_all(carpeta);
    }

    #[test]
    fn un_fallo_de_activacion_recupera_la_anterior() {
        let carpeta = temporal("recupera");
        let anterior = carpeta.join("anterior");
        let preview = carpeta.join("preview");
        let activa = carpeta.join("activa");
        let perfil = carpeta.join("perfil");
        fs::create_dir_all(anterior.join("bin")).expect("debería crear anterior");
        fs::create_dir_all(preview.join("bin")).expect("debería crear preview");
        symlink(&anterior, &activa).expect("debería crear activa");
        symlink(&anterior, &perfil).expect("debería crear perfil");

        let nix_env = carpeta.join("nix-env");
        programa(
            &nix_env,
            "#!/bin/sh\nset -eu\nperfil=\"$2\"\ndestino=\"$4\"\nln -sfn \"$destino\" \"$perfil\"\n",
        );
        programa(
            &anterior.join("bin/switch-to-configuration"),
            &format!(
                "#!/bin/sh\nset -eu\n[ \"$1\" = switch ] && ln -sfn '{}' '{}'\n",
                anterior.display(),
                activa.display()
            ),
        );
        programa(
            &preview.join("bin/switch-to-configuration"),
            "#!/bin/sh\nexit 1\n",
        );

        let error = aplicar_generacion(&preview, &anterior, &nix_env, &perfil, &activa)
            .expect_err("la activación debería fallar");

        assert!(error.contains("recuperado"));
        assert_eq!(destino(&activa).unwrap(), anterior);
        assert_eq!(destino(&perfil).unwrap(), anterior);
        let _ = fs::remove_dir_all(carpeta);
    }
}

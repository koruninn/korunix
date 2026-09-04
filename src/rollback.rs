use crate::aplicar;
use crate::preview;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

const SISTEMA_ACTIVO: &str = "/run/current-system";
const PERFIL_SISTEMA: &str = "/nix/var/nix/profiles/system";

#[derive(Debug, PartialEq, Eq)]
enum Situacion {
    YaVuelto,
    Listo,
}

#[derive(Debug)]
struct Preparado {
    actual: PathBuf,
    anterior: PathBuf,
    enlace_anterior: PathBuf,
    configuracion_actual: aplicar::ConfiguracionGeneracion,
    configuracion_anterior: aplicar::ConfiguracionGeneracion,
    situacion: Situacion,
}

fn clasificar_estado(
    activa: &Path,
    persistente: &Path,
    anterior: &Path,
) -> Result<Situacion, String> {
    if activa != persistente {
        return Err(format!(
            "La generación activa y la persistente no coinciden.\n\
             Activa: {}\nPersistente: {}\n\
             No voy a hacer rollback desde un estado parcial.",
            activa.display(),
            persistente.display()
        ));
    }

    if activa == anterior {
        Ok(Situacion::YaVuelto)
    } else {
        Ok(Situacion::Listo)
    }
}

fn revisar_activador(generacion: &Path) -> Result<(), String> {
    if !generacion.is_absolute() || !generacion.starts_with("/nix/store") {
        return Err(format!(
            "La generación anterior no apunta a /nix/store: {}",
            generacion.display()
        ));
    }

    let activador = generacion.join("bin/switch-to-configuration");
    let datos = fs::metadata(&activador).map_err(|error| {
        format!(
            "La generación anterior ya no parece una generación NixOS aplicable.\n\
             Detalle: {error}"
        )
    })?;

    if !datos.is_file() || datos.permissions().mode() & 0o111 == 0 {
        return Err("La generación anterior no tiene un activador NixOS ejecutable.".to_string());
    }

    Ok(())
}

fn revisar_registros(
    actual: &Path,
    anterior: &Path,
    configuracion_actual: &aplicar::ConfiguracionGeneracion,
    configuracion_anterior: &aplicar::ConfiguracionGeneracion,
) -> Result<(), String> {
    if configuracion_actual.generacion != actual {
        return Err(format!(
            "La configuración humana aplicada no corresponde a la generación activa.\n\
             Configuración: {}\nActiva:        {}",
            configuracion_actual.generacion.display(),
            actual.display()
        ));
    }

    if configuracion_anterior.generacion != anterior {
        return Err(format!(
            "La configuración humana anterior no corresponde al punto de rollback.\n\
             Configuración: {}\nRollback:      {}",
            configuracion_anterior.generacion.display(),
            anterior.display()
        ));
    }

    Ok(())
}

fn preparar(_raiz: &Path) -> Result<Preparado, String> {
    let estado = preview::carpeta_estado()?;
    let enlace_anterior = estado.join("anterior");

    let anterior = aplicar::destino(&enlace_anterior).map_err(|error| {
        format!(
            "No encontré un punto de rollback válido en {}.\n{error}",
            enlace_anterior.display()
        )
    })?;
    revisar_activador(&anterior)?;

    let activa = aplicar::destino(Path::new(SISTEMA_ACTIVO))?;
    let persistente = aplicar::destino(Path::new(PERFIL_SISTEMA))?;
    let situacion = clasificar_estado(&activa, &persistente, &anterior)?;

    if situacion == Situacion::YaVuelto {
        return Ok(Preparado {
            actual: activa.clone(),
            anterior: activa.clone(),
            enlace_anterior,
            configuracion_actual: aplicar::ConfiguracionGeneracion {
                generacion: activa.clone(),
                configuracion: Vec::new(),
            },
            configuracion_anterior: aplicar::ConfiguracionGeneracion {
                generacion: activa,
                configuracion: Vec::new(),
            },
            situacion,
        });
    }

    let nix_store = aplicar::programa_variable(
        "KORUNIX_NIX_STORE_BIN",
        "/run/current-system/sw/bin/nix-store",
    )?;

    if !aplicar::esta_protegida(&nix_store, &anterior, &enlace_anterior)? {
        return Err(
            "La generación anterior existe, pero Nix no la reconoce como raíz de GC. \
             No voy a hacer rollback hacia un estado que puede desaparecer."
                .to_string(),
        );
    }

    let configuracion_actual = aplicar::leer_aplicada(&estado)?.ok_or_else(|| {
        "No tengo registrada la configuración humana de la generación activa. \
         No voy a hacer un rollback que deje TOML y NixOS diciendo cosas distintas."
            .to_string()
    })?;

    let configuracion_anterior = aplicar::leer_anterior(&estado)?.ok_or_else(|| {
        "Tengo la generación anterior, pero no la configuración humana que le correspondía. \
         Este punto es anterior al rollback completo; no voy a volver a medias."
            .to_string()
    })?;

    revisar_registros(
        &activa,
        &anterior,
        &configuracion_actual,
        &configuracion_anterior,
    )?;

    Ok(Preparado {
        actual: activa,
        anterior,
        enlace_anterior,
        configuracion_actual,
        configuracion_anterior,
        situacion,
    })
}

fn guardar_borrador_si_hace_falta(
    raiz: &Path,
    estado: &Path,
    aplicada: &aplicar::ConfiguracionGeneracion,
) -> Result<Option<PathBuf>, String> {
    let ruta = raiz.join("configuracion.toml");
    let actual = fs::read(&ruta)
        .map_err(|error| format!("No pude leer configuracion.toml.\nDetalle: {error}"))?;

    if actual == aplicada.configuracion {
        return Ok(None);
    }

    let borrador = estado.join("configuracion-antes-de-rollback.toml");
    fs::create_dir_all(estado)
        .map_err(|error| format!("No pude preparar el estado de Korunix.\nDetalle: {error}"))?;
    fs::write(&borrador, &actual).map_err(|error| {
        format!("No pude conservar tus cambios actuales antes de rollback.\nDetalle: {error}")
    })?;

    Ok(Some(borrador))
}

fn preparar_configuracion_temporal(raiz: &Path, configuracion: &[u8]) -> Result<PathBuf, String> {
    let destino = raiz.join("configuracion.toml");
    let temporal = raiz.join(format!(".configuracion-rollback-{}.toml", process::id()));

    let _ = fs::remove_file(&temporal);
    fs::write(&temporal, configuracion).map_err(|error| {
        format!("No pude preparar la configuración anterior antes de rollback.\nDetalle: {error}")
    })?;

    if let Ok(datos) = fs::metadata(&destino) {
        fs::set_permissions(&temporal, datos.permissions()).map_err(|error| {
            format!("No pude conservar los permisos de configuracion.toml.\nDetalle: {error}")
        })?;
    }

    Ok(temporal)
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

fn recuperar_origen(origen: &Path, nix_env: &Path) -> Result<(), String> {
    aplicar::recuperar(
        origen,
        nix_env,
        Path::new(PERFIL_SISTEMA),
        Path::new(SISTEMA_ACTIVO),
    )
}

fn volver_generacion(anterior: &Path, origen: &Path, nix_env: &Path) -> Result<(), String> {
    println!("Fase 1/2 · dejando la generación anterior como persistente...");

    if let Err(error) = aplicar::fijar_perfil(nix_env, Path::new(PERFIL_SISTEMA), anterior) {
        let recuperacion = recuperar_origen(origen, nix_env);
        return match recuperacion {
            Ok(()) => Err(format!(
                "{error}\nEl sistema quedó recuperado en la generación de origen."
            )),
            Err(error_recuperacion) => Err(format!(
                "{error}\nERROR CRÍTICO: tampoco pude recuperar la generación de origen.\n\
                 {error_recuperacion}"
            )),
        };
    }

    let persistente = aplicar::destino(Path::new(PERFIL_SISTEMA))?;
    if persistente != anterior {
        let error = format!(
            "El perfil persistente no quedó en la generación anterior.\nEncontrado: {}",
            persistente.display()
        );
        let recuperacion = recuperar_origen(origen, nix_env);

        return match recuperacion {
            Ok(()) => Err(format!(
                "{error}\nEl sistema quedó recuperado en la generación de origen."
            )),
            Err(error_recuperacion) => Err(format!(
                "{error}\nERROR CRÍTICO: la recuperación también falló.\n\
                 {error_recuperacion}"
            )),
        };
    }

    println!("Fase 2/2 · activando esa misma generación...");

    if let Err(error) = aplicar::ejecutar_accion(anterior, "switch") {
        let recuperacion = recuperar_origen(origen, nix_env);

        return match recuperacion {
            Ok(()) => Err(format!(
                "{error}\nEl sistema quedó recuperado en la generación de origen."
            )),
            Err(error_recuperacion) => Err(format!(
                "{error}\nERROR CRÍTICO: la recuperación también falló.\n\
                 {error_recuperacion}"
            )),
        };
    }

    let activa = aplicar::destino(Path::new(SISTEMA_ACTIVO))?;
    let persistente = aplicar::destino(Path::new(PERFIL_SISTEMA))?;

    if activa != anterior || persistente != anterior {
        let error = format!(
            "Rollback no terminó con activa = persistente = anterior.\n\
             Activa: {}\nPersistente: {}",
            activa.display(),
            persistente.display()
        );
        let recuperacion = recuperar_origen(origen, nix_env);

        return match recuperacion {
            Ok(()) => Err(format!(
                "{error}\nEl sistema quedó recuperado en la generación de origen."
            )),
            Err(error_recuperacion) => Err(format!(
                "{error}\nERROR CRÍTICO: la recuperación también falló.\n\
                 {error_recuperacion}"
            )),
        };
    }

    Ok(())
}

pub fn ejecutar_como_root(argumentos: &[String]) -> i32 {
    if !es_root() {
        eprintln!("Esta parte de rollback solo puede ejecutarla root.");
        return 90;
    }

    if argumentos.len() != 6 {
        eprintln!("La llamada interna de rollback está incompleta.");
        return 91;
    }

    let anterior = PathBuf::from(&argumentos[0]);
    let origen = PathBuf::from(&argumentos[1]);
    let nix_env = PathBuf::from(&argumentos[2]);
    let resultado = PathBuf::from(&argumentos[3]);
    let configuracion_temporal = PathBuf::from(&argumentos[4]);
    let configuracion_destino = PathBuf::from(&argumentos[5]);

    let activa = match aplicar::destino(Path::new(SISTEMA_ACTIVO)) {
        Ok(ruta) => ruta,
        Err(error) => {
            eprintln!("{error}");
            marcar(&resultado, "sin-cambio");
            return 10;
        }
    };
    let persistente = match aplicar::destino(Path::new(PERFIL_SISTEMA)) {
        Ok(ruta) => ruta,
        Err(error) => {
            eprintln!("{error}");
            marcar(&resultado, "sin-cambio");
            return 10;
        }
    };

    if activa != origen || persistente != origen {
        eprintln!("El sistema cambió antes de empezar la revisión de rollback.");
        marcar(&resultado, "sin-cambio");
        return 10;
    }

    println!("Comprobando si NixOS permite volver...");
    if let Err(error) = aplicar::ejecutar_accion(&anterior, "check") {
        eprintln!("{error}");
        marcar(&resultado, "sin-cambio");
        return 11;
    }

    println!();
    println!("Simulando rollback...");
    if let Err(error) = aplicar::ejecutar_accion(&anterior, "dry-activate") {
        eprintln!("{error}");
        marcar(&resultado, "sin-cambio");
        return 12;
    }

    println!();
    println!("Efecto de rollback");
    println!("Ahora:      {}", origen.display());
    println!("Volverá a:  {}", anterior.display());

    let kernel_ahora = aplicar::kernel_actual();
    let kernel_anterior = aplicar::kernel_generacion(&anterior);

    if !kernel_ahora.is_empty() {
        println!("Kernel en ejecución: {kernel_ahora}");
    }

    if let Some(kernel_anterior) = &kernel_anterior {
        println!("Kernel al volver:    {kernel_anterior}");

        if *kernel_anterior != kernel_ahora {
            println!("Reinicio: sí, para empezar a usar el kernel {kernel_anterior}.");
        } else {
            println!("Reinicio: no por cambio de kernel.");
        }
    }

    println!(
        "Configuración: configuracion.toml volverá a la copia humana asociada a esa generación."
    );
    println!(
        "Sesión: algunos ajustes de escritorio pueden completarse al volver a iniciar sesión."
    );
    println!("Persistencia: la generación anterior quedará elegida para los próximos arranques.");
    println!("NixOS todavía no cambió.");
    println!();
    print!("Escribe exactamente VOLVER para hacer rollback: ");
    let _ = io::stdout().flush();

    let mut respuesta = String::new();
    if io::stdin().read_line(&mut respuesta).is_err() || respuesta.trim() != "VOLVER" {
        println!("Cancelado. NixOS no cambió.");
        marcar(&resultado, "cancelado");
        return 0;
    }

    let activa = match aplicar::destino(Path::new(SISTEMA_ACTIVO)) {
        Ok(ruta) => ruta,
        Err(error) => {
            eprintln!("{error}");
            marcar(&resultado, "sin-cambio");
            return 13;
        }
    };
    let persistente = match aplicar::destino(Path::new(PERFIL_SISTEMA)) {
        Ok(ruta) => ruta,
        Err(error) => {
            eprintln!("{error}");
            marcar(&resultado, "sin-cambio");
            return 13;
        }
    };

    if activa != origen || persistente != origen {
        eprintln!("El sistema cambió entre la revisión y tu autorización.");
        marcar(&resultado, "sin-cambio");
        return 13;
    }

    if let Err(error) = volver_generacion(&anterior, &origen, &nix_env) {
        eprintln!("{error}");

        let activa = aplicar::destino(Path::new(SISTEMA_ACTIVO)).ok();
        let persistente = aplicar::destino(Path::new(PERFIL_SISTEMA)).ok();

        if activa.as_deref() == Some(origen.as_path())
            && persistente.as_deref() == Some(origen.as_path())
        {
            marcar(&resultado, "recuperado");
            return 14;
        }

        marcar(&resultado, "critico");
        return 15;
    }

    if let Err(error) = fs::rename(&configuracion_temporal, &configuracion_destino) {
        eprintln!(
            "NixOS volvió a la generación anterior, pero no pude restaurar configuracion.toml.\n\
             Detalle: {error}"
        );

        match recuperar_origen(&origen, &nix_env) {
            Ok(()) => {
                marcar(&resultado, "recuperado");
                return 16;
            }
            Err(error_recuperacion) => {
                eprintln!(
                    "ERROR CRÍTICO: tampoco pude recuperar la generación de origen.\n\
                     {error_recuperacion}"
                );
                marcar(&resultado, "critico");
                return 17;
            }
        }
    }

    println!("✓ Rollback terminó con activa = persistente = anterior.");
    marcar(&resultado, "vuelto");
    0
}

pub fn ejecutar(raiz: &Path) -> Result<(), String> {
    let preparado = preparar(raiz)?;

    if preparado.situacion == Situacion::YaVuelto {
        println!("✓ El sistema ya está en el punto de rollback.");
        println!("Generación: {}", preparado.anterior.display());
        println!("No cambié nada.");
        return Ok(());
    }

    let estado = preview::carpeta_estado()?;
    let nix_store = aplicar::programa_variable(
        "KORUNIX_NIX_STORE_BIN",
        "/run/current-system/sw/bin/nix-store",
    )?;

    if !aplicar::esta_protegida(&nix_store, &preparado.anterior, &preparado.enlace_anterior)? {
        return Err(
            "La generación anterior dejó de estar protegida antes de rollback.".to_string(),
        );
    }

    let borrador = guardar_borrador_si_hace_falta(raiz, &estado, &preparado.configuracion_actual)?;

    if let Some(ruta) = &borrador {
        println!(
            "✓ Tus cambios actuales sin aplicar quedaron guardados en {}.",
            ruta.display()
        );
    }

    let configuracion_destino = raiz.join("configuracion.toml");
    let configuracion_temporal =
        preparar_configuracion_temporal(raiz, &preparado.configuracion_anterior.configuracion)?;

    let sudo = aplicar::sudo_nixos()?;
    let systemd_run = aplicar::programa_variable(
        "KORUNIX_SYSTEMD_RUN_BIN",
        "/run/current-system/sw/bin/systemd-run",
    )?;
    let systemctl = aplicar::programa_variable(
        "KORUNIX_SYSTEMCTL_BIN",
        "/run/current-system/sw/bin/systemctl",
    )?;
    let nix_env =
        aplicar::programa_variable("KORUNIX_NIX_ENV_BIN", "/run/current-system/sw/bin/nix-env")?;
    let ejecutable = env::current_exe().map_err(|error| {
        format!("No pude localizar el Korunix que estás usando.\nDetalle: {error}")
    })?;

    let fallidas_antes = aplicar::unidades_fallidas(&systemctl)?;
    let resultado = estado.join(format!(".rollback-resultado-{}", process::id()));
    fs::write(&resultado, "iniciado\n")
        .map_err(|error| format!("No pude preparar el resultado de rollback.\nDetalle: {error}"))?;

    println!();
    println!("NixOS necesita autorización para revisar rollback.");
    println!(
        "La misma autorización se mantiene desde dry-activate hasta el cambio; no se ejecutará nix build."
    );

    let unidad = format!("korunix-rollback-{}", process::id());
    let estado_ejecucion = Command::new(&sudo)
        .arg(&systemd_run)
        .arg("--wait")
        .arg("--collect")
        .arg("--pipe")
        .arg("--quiet")
        .arg("--service-type=exec")
        .arg(format!("--unit={unidad}"))
        .arg(&ejecutable)
        .arg("__rollback-raiz")
        .arg(&preparado.anterior)
        .arg(&preparado.actual)
        .arg(&nix_env)
        .arg(&resultado)
        .arg(&configuracion_temporal)
        .arg(&configuracion_destino)
        .status()
        .map_err(|error| format!("No pude iniciar rollback con autorización.\nDetalle: {error}"))?;

    let marca = fs::read_to_string(&resultado)
        .unwrap_or_else(|_| "desconocido".to_string())
        .trim()
        .to_string();
    let _ = fs::remove_file(&resultado);
    let _ = fs::remove_file(&configuracion_temporal);

    let activa = aplicar::destino(Path::new(SISTEMA_ACTIVO))?;
    let persistente = aplicar::destino(Path::new(PERFIL_SISTEMA))?;
    let configuracion_final = fs::read(&configuracion_destino)
        .map_err(|error| format!("No pude comprobar configuracion.toml.\nDetalle: {error}"))?;

    if marca == "cancelado" && activa == preparado.actual && persistente == preparado.actual {
        println!("No se hizo rollback.");
        return Ok(());
    }

    if activa == preparado.anterior
        && persistente == preparado.anterior
        && configuracion_final == preparado.configuracion_anterior.configuracion
    {
        if !estado_ejecucion.success() {
            eprintln!(
                "Aviso: la conexión con la unidad devolvió {}, pero el estado final confirma rollback.",
                estado_ejecucion
            );
        }
    } else if activa == preparado.actual && persistente == preparado.actual {
        return Err(format!(
            "Rollback no se completó. El sistema quedó recuperado en la generación de origen.\n\
             Estado interno: {marca}\nUnidad: {unidad}"
        ));
    } else {
        return Err(format!(
            "ERROR CRÍTICO: rollback dejó un estado parcial.\n\
             Activa: {}\nPersistente: {}\n\
             Configuración anterior restaurada: {}\nUnidad: {unidad}",
            activa.display(),
            persistente.display(),
            configuracion_final == preparado.configuracion_anterior.configuracion
        ));
    }

    aplicar::guardar_aplicada(&estado, &preparado.configuracion_anterior)?;

    let fallidas_despues = aplicar::unidades_fallidas(&systemctl)?;
    let nuevas: Vec<_> = fallidas_despues
        .difference(&fallidas_antes)
        .cloned()
        .collect();

    if !nuevas.is_empty() {
        return Err(format!(
            "Rollback quedó activo y persistente, pero aparecieron unidades fallidas nuevas:\n  - {}\n\
             Esto debe revisarse antes de cerrar rollback.",
            nuevas.join("\n  - ")
        ));
    }

    println!();
    println!("✓ Volví exactamente a la generación anterior.");
    println!("✓ No se reconstruyó NixOS.");
    println!("✓ Activa = persistente = {}", preparado.anterior.display());
    println!("✓ configuracion.toml volvió a la copia humana de esa generación.");
    println!("✓ No aparecieron nuevas unidades systemd fallidas.");

    let kernel_ahora = aplicar::kernel_actual();
    if let Some(kernel_anterior) = aplicar::kernel_generacion(&preparado.anterior) {
        if kernel_anterior != kernel_ahora {
            println!("Reinicio pendiente: kernel actual {kernel_ahora} → {kernel_anterior}.");
        }
    }

    Ok(())
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporal(nombre: &str) -> PathBuf {
        let momento = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("el reloj debería funcionar")
            .as_nanos();

        env::temp_dir().join(format!(
            "korunix-rollback-{nombre}-{}-{momento}",
            process::id()
        ))
    }

    #[test]
    fn un_estado_dividido_se_rechaza() {
        let activa = Path::new("/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-activa");
        let persistente = Path::new("/nix/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-persistente");
        let anterior = Path::new("/nix/store/cccccccccccccccccccccccccccccccc-anterior");

        let error = clasificar_estado(activa, persistente, anterior)
            .expect_err("debería rechazar un estado parcial");

        assert!(error.contains("no coinciden"));
    }

    #[test]
    fn volver_dos_veces_es_inocuo() {
        let anterior = Path::new("/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-anterior");

        assert_eq!(
            clasificar_estado(anterior, anterior, anterior).expect("debería reconocer el estado"),
            Situacion::YaVuelto
        );
    }

    #[test]
    fn los_toml_tienen_que_corresponder_a_sus_generaciones() {
        let actual = PathBuf::from("/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-actual");
        let anterior = PathBuf::from("/nix/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-anterior");

        let aplicada = aplicar::ConfiguracionGeneracion {
            generacion: actual.clone(),
            configuracion: b"nombre = \"actual\"\n".to_vec(),
        };
        let previa = aplicar::ConfiguracionGeneracion {
            generacion: anterior.clone(),
            configuracion: b"nombre = \"anterior\"\n".to_vec(),
        };

        revisar_registros(&actual, &anterior, &aplicada, &previa).expect("deberían coincidir");

        let equivocada = aplicar::ConfiguracionGeneracion {
            generacion: actual.clone(),
            configuracion: previa.configuracion.clone(),
        };

        let error = revisar_registros(&actual, &anterior, &aplicada, &equivocada)
            .expect_err("debería detectar la mezcla");

        assert!(error.contains("no corresponde"));
    }

    #[test]
    fn un_borrador_conserva_cambios_sin_aplicar() {
        let carpeta = temporal("borrador");
        let raiz = carpeta.join("repo");
        let estado = carpeta.join("estado");
        fs::create_dir_all(&raiz).expect("debería crear repo");

        fs::write(raiz.join("configuracion.toml"), b"nombre = \"editado\"\n")
            .expect("debería escribir configuración");

        let aplicada = aplicar::ConfiguracionGeneracion {
            generacion: PathBuf::from("/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-actual"),
            configuracion: b"nombre = \"aplicado\"\n".to_vec(),
        };

        let borrador = guardar_borrador_si_hace_falta(&raiz, &estado, &aplicada)
            .expect("debería guardar el borrador")
            .expect("debería existir");

        assert_eq!(
            fs::read(borrador).expect("debería leer borrador"),
            b"nombre = \"editado\"\n"
        );

        let _ = fs::remove_dir_all(carpeta);
    }
}

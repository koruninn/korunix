use crate::configuracion::Configuracion;
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use toml_edit::{value, DocumentMut, Item, Table};

const PATRON_CAPTURA: &str = "Captura de pantalla del %Y-%m-%d %H-%M-%S";

#[derive(Debug, Deserialize)]
pub struct Plan {
    pub nombre: String,
    pub canal: String,
    pub escritorio: String,
    pub personas: Vec<PersonaPlan>,
    pub revision: String,
    pub aplicaciones: Vec<Aplicacion>,
    pub noctalia: bool,
    pub noctalia_version: String,
}

#[derive(Debug, Deserialize)]
pub struct PersonaPlan {
    pub cuenta: String,
    pub administrador: bool,
}

#[derive(Debug, Deserialize)]
pub struct Aplicacion {
    pub elegida: String,
    pub nombre: String,
    pub version: String,
}

pub struct SesionPreparada {
    pub configuracion_noctalia: PathBuf,
    pub capturas: PathBuf,
}

fn sistema_nix() -> Result<&'static str, String> {
    match env::consts::ARCH {
        "x86_64" => Ok("x86_64-linux"),
        "aarch64" => Ok("aarch64-linux"),
        otro => Err(format!(
            "Korunix todavía no sabe preparar un plan para «{otro}»."
        )),
    }
}

fn leer_plan(texto: &[u8]) -> Result<Plan, String> {
    serde_json::from_slice(texto)
        .map_err(|error| format!("Nix devolvió un plan que no pude entender.\nDetalle: {error}"))
}

fn comprobar_plan(configuracion: &Configuracion, plan: &Plan) -> Result<(), String> {
    if plan.nombre != configuracion.nombre {
        return Err(
            "El nombre que resolvió Nix no coincide con configuracion.toml. No voy a usar ese plan."
                .to_string(),
        );
    }

    if plan.canal != configuracion.canal {
        return Err(
            "El canal que resolvió Nix no coincide con configuracion.toml. No voy a usar ese plan."
                .to_string(),
        );
    }

    if plan.escritorio != configuracion.escritorio.principal {
        return Err(
            "El escritorio que resolvió Nix no coincide con configuracion.toml. No voy a usar ese plan."
                .to_string(),
        );
    }

    let noctalia_esperado = matches!(
        configuracion.escritorio.principal.as_str(),
        "niri" | "hyprland"
    );

    if plan.noctalia != noctalia_esperado {
        return Err(
            "Noctalia no coincide con el escritorio elegido. No voy a usar ese plan.".to_string(),
        );
    }

    let personas: Vec<(&str, bool)> = plan
        .personas
        .iter()
        .map(|persona| (persona.cuenta.as_str(), persona.administrador))
        .collect();

    let esperadas: Vec<(&str, bool)> = configuracion
        .personas
        .iter()
        .map(|persona| (persona.cuenta.as_str(), persona.administrador))
        .collect();

    if personas != esperadas {
        return Err(
            "Las cuentas que resolvió Nix no coinciden con configuracion.toml. No voy a usar ese plan."
                .to_string(),
        );
    }

    let elegidas: Vec<&str> = plan
        .aplicaciones
        .iter()
        .map(|aplicacion| aplicacion.elegida.as_str())
        .collect();

    let esperadas: Vec<&str> = configuracion
        .aplicaciones
        .instaladas
        .iter()
        .map(String::as_str)
        .collect();

    if elegidas != esperadas {
        return Err(
            "Las aplicaciones que resolvió Nix no coinciden con configuracion.toml. No voy a usar ese plan."
                .to_string(),
        );
    }

    Ok(())
}

pub fn preparar_plan(raiz: &Path, configuracion: &Configuracion) -> Result<Plan, String> {
    let sistema = sistema_nix()?;
    let atributo = format!(".#packages.{sistema}.korunix.plan");
    let programa = env::var_os("KORUNIX_NIX_BIN").unwrap_or_else(|| "nix".into());

    let salida = Command::new(programa)
        .args(["eval", "--json", &atributo])
        .current_dir(raiz)
        .output()
        .map_err(|error| format!("No pude pedirle el plan a Nix.\nDetalle: {error}"))?;

    if !salida.status.success() {
        let detalle = String::from_utf8_lossy(&salida.stderr).trim().to_string();

        if detalle.is_empty() {
            return Err("Nix no pudo preparar el plan.".to_string());
        }

        return Err(format!("Nix no pudo preparar el plan.\nDetalle: {detalle}"));
    }

    let plan = leer_plan(&salida.stdout)?;
    comprobar_plan(configuracion, &plan)?;

    Ok(plan)
}

fn revisar_ruta_generacion(ruta: PathBuf) -> Result<PathBuf, String> {
    if !ruta.is_absolute() || !ruta.starts_with("/nix/store") {
        return Err(format!(
            "Nix construyó algo en una ruta que no esperaba: {}",
            ruta.display()
        ));
    }

    Ok(ruta)
}

pub fn construir_generacion(raiz: &Path) -> Result<PathBuf, String> {
    let programa = env::var_os("KORUNIX_NIX_BIN").unwrap_or_else(|| "nix".into());
    let enlace = env::temp_dir().join(format!("korunix-generacion-{}", process::id()));

    if enlace.exists() {
        fs::remove_file(&enlace).map_err(|error| {
            format!("No pude limpiar una generación temporal.\nDetalle: {error}")
        })?;
    }

    let enlace_texto = enlace.to_string_lossy().into_owned();

    let estado = Command::new(programa)
        .args([
            "build",
            "--out-link",
            &enlace_texto,
            ".#nixosConfigurations.korunix.config.system.build.toplevel",
        ])
        .current_dir(raiz)
        .status()
        .map_err(|error| format!("No pude pedirle la generación a Nix.\nDetalle: {error}"))?;

    if !estado.success() {
        let _ = fs::remove_file(&enlace);
        return Err("Nix no pudo construir la generación.".to_string());
    }

    let ruta = fs::read_link(&enlace)
        .map_err(|error| format!("No pude leer la generación construida.\nDetalle: {error}"))?;

    fs::remove_file(&enlace)
        .map_err(|error| format!("No pude limpiar el enlace temporal.\nDetalle: {error}"))?;

    revisar_ruta_generacion(ruta)
}

fn carpeta_imagenes(home: &Path, config_home: &Path) -> PathBuf {
    let user_dirs = config_home.join("user-dirs.dirs");

    if let Ok(texto) = fs::read_to_string(user_dirs) {
        for linea in texto.lines() {
            let linea = linea.trim();

            let Some(valor) = linea.strip_prefix("XDG_PICTURES_DIR=") else {
                continue;
            };

            let valor = valor.trim().trim_matches('"');

            if let Some(resto) = valor.strip_prefix("$HOME") {
                return home.join(resto.trim_start_matches('/'));
            }

            let ruta = PathBuf::from(valor);

            if ruta.is_absolute() {
                return ruta;
            }
        }
    }

    home.join("Pictures")
}

fn tabla<'a>(tabla: &'a mut Table, nombre: &str, contexto: &str) -> Result<&'a mut Table, String> {
    tabla
        .entry(nombre)
        .or_insert(Item::Table(Table::new()))
        .as_table_mut()
        .ok_or_else(|| format!("No pude usar [{contexto}]. Esa parte tiene que ser una sección."))
}

fn guardar_toml(ruta: &Path, texto: &str) -> Result<(), String> {
    let carpeta = ruta.parent().unwrap_or_else(|| Path::new("."));
    let nombre = ruta
        .file_name()
        .and_then(|nombre| nombre.to_str())
        .unwrap_or("config.toml");

    fs::create_dir_all(carpeta).map_err(|error| {
        format!(
            "No pude preparar la carpeta {}.\nDetalle: {error}",
            carpeta.display()
        )
    })?;

    let temporal = carpeta.join(format!(".{nombre}.korunix-{}.tmp", process::id()));

    let resultado = (|| {
        fs::write(&temporal, texto)
            .map_err(|error| format!("No pude guardar la sesión.\nDetalle: {error}"))?;

        if let Ok(datos) = fs::metadata(ruta) {
            fs::set_permissions(&temporal, datos.permissions())
                .map_err(|error| format!("No pude conservar los permisos.\nDetalle: {error}"))?;
        }

        fs::rename(&temporal, ruta)
            .map_err(|error| format!("No pude terminar de guardar la sesión.\nDetalle: {error}"))?;

        Ok::<(), String>(())
    })();

    if resultado.is_err() {
        let _ = fs::remove_file(&temporal);
    }

    resultado
}

fn fusionar_capturas(ruta: &Path, capturas: &Path) -> Result<(), String> {
    let texto = fs::read_to_string(ruta)
        .map_err(|error| format!("No pude leer {}.\nDetalle: {error}", ruta.display()))?;

    let mut documento = texto.parse::<DocumentMut>().map_err(|error| {
        format!(
            "No pude entender {}.\nNo se cambió ese archivo.\nDetalle: {error}",
            ruta.display()
        )
    })?;

    let shell = tabla(documento.as_table_mut(), "shell", "shell")?;
    let screenshot = tabla(shell, "screenshot", "shell.screenshot")?;

    screenshot["directory"] = value(capturas.to_string_lossy().to_string());
    screenshot["filename_pattern"] = value(PATRON_CAPTURA);

    guardar_toml(ruta, &documento.to_string())
}

fn preparar_sesion_en(
    base: &Path,
    home: &Path,
    config_home: &Path,
    state_home: &Path,
) -> Result<SesionPreparada, String> {
    if !base.is_file() {
        return Err(format!(
            "No encontré la configuración base de Noctalia en {}.",
            base.display()
        ));
    }

    let capturas = carpeta_imagenes(home, config_home).join("Capturas de pantalla");
    fs::create_dir_all(&capturas).map_err(|error| {
        format!(
            "No pude preparar la carpeta de capturas {}.\nDetalle: {error}",
            capturas.display()
        )
    })?;

    let noctalia_dir = config_home.join("noctalia");
    let configuracion_noctalia = noctalia_dir.join("config.toml");

    if !configuracion_noctalia.exists() {
        fs::create_dir_all(&noctalia_dir).map_err(|error| {
            format!(
                "No pude preparar {}.\nDetalle: {error}",
                noctalia_dir.display()
            )
        })?;

        fs::copy(base, &configuracion_noctalia).map_err(|error| {
            format!("No pude crear la configuración inicial de Noctalia.\nDetalle: {error}")
        })?;
    }

    fusionar_capturas(&configuracion_noctalia, &capturas)?;

    let settings = state_home.join("noctalia/settings.toml");

    if settings.is_file() {
        fusionar_capturas(&settings, &capturas)?;
    }

    Ok(SesionPreparada {
        configuracion_noctalia,
        capturas,
    })
}

pub fn preparar_sesion() -> Result<SesionPreparada, String> {
    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "No pude saber cuál es tu carpeta personal.".to_string())?;

    let config_home = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".config"));

    let state_home = env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".local/state"));

    let base = env::var_os("KORUNIX_NOCTALIA_BASE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/etc/korunix/noctalia.toml"));

    preparar_sesion_en(&base, &home, &config_home, &state_home)
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::configuracion::{Aplicaciones, Escritorio, Persona};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn configuracion() -> Configuracion {
        Configuracion {
            nombre: "korunix".to_string(),
            canal: "inestable".to_string(),
            personas: vec![Persona {
                cuenta: "koru".to_string(),
                nombre: "André".to_string(),
                administrador: true,
            }],
            escritorio: Escritorio {
                principal: "niri".to_string(),
            },
            aplicaciones: Aplicaciones {
                instaladas: vec!["firefox".to_string(), "karere".to_string()],
            },
        }
    }

    fn plan() -> Plan {
        Plan {
            nombre: "korunix".to_string(),
            canal: "inestable".to_string(),
            escritorio: "niri".to_string(),
            personas: vec![PersonaPlan {
                cuenta: "koru".to_string(),
                administrador: true,
            }],
            revision: "abc123".to_string(),
            aplicaciones: vec![
                Aplicacion {
                    elegida: "firefox".to_string(),
                    nombre: "firefox".to_string(),
                    version: "1".to_string(),
                },
                Aplicacion {
                    elegida: "karere".to_string(),
                    nombre: "karere".to_string(),
                    version: "2".to_string(),
                },
            ],
            noctalia: true,
            noctalia_version: "5".to_string(),
        }
    }

    fn temporal(nombre: &str) -> PathBuf {
        let momento = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("el reloj debería funcionar")
            .as_nanos();

        env::temp_dir().join(format!("korunix-{nombre}-{}-{momento}", process::id()))
    }

    #[test]
    fn entiende_el_plan_de_nix() {
        let texto = br#"{
          "nombre": "korunix",
          "canal": "inestable",
          "escritorio": "niri",
          "personas": [{"cuenta": "koru", "administrador": true}],
          "revision": "abc123",
          "aplicaciones": [
            {"elegida": "firefox", "nombre": "firefox", "version": "1"}
          ],
          "noctalia": true,
          "noctalia_version": "5"
        }"#;

        let plan = leer_plan(texto).expect("el plan debería entenderse");

        assert_eq!(plan.nombre, "korunix");
        assert_eq!(plan.escritorio, "niri");
        assert_eq!(plan.personas[0].cuenta, "koru");
        assert!(plan.noctalia);
    }

    #[test]
    fn rechaza_un_plan_con_otro_nombre() {
        let configuracion = configuracion();
        let mut plan = plan();
        plan.nombre = "otra-pc".to_string();

        let error = comprobar_plan(&configuracion, &plan)
            .expect_err("un nombre distinto debería rechazarse");

        assert!(error.contains("nombre"));
        assert!(error.contains("no coincide"));
    }

    #[test]
    fn rechaza_un_plan_con_otro_canal() {
        let configuracion = configuracion();
        let mut plan = plan();
        plan.canal = "estable".to_string();

        let error = comprobar_plan(&configuracion, &plan)
            .expect_err("un canal distinto debería rechazarse");

        assert!(error.contains("no coincide"));
    }

    #[test]
    fn rechaza_un_plan_con_otro_escritorio() {
        let configuracion = configuracion();
        let mut plan = plan();
        plan.escritorio = "plasma".to_string();

        let error = comprobar_plan(&configuracion, &plan)
            .expect_err("un escritorio distinto debería rechazarse");

        assert!(error.contains("escritorio"));
        assert!(error.contains("no coincide"));
    }

    #[test]
    fn rechaza_un_plan_con_noctalia_equivocada() {
        let configuracion = configuracion();
        let mut plan = plan();
        plan.noctalia = false;

        let error =
            comprobar_plan(&configuracion, &plan).expect_err("Noctalia debería derivarse de Niri");

        assert!(error.contains("Noctalia"));
        assert!(error.contains("escritorio"));
    }

    #[test]
    fn rechaza_un_plan_con_otras_personas() {
        let configuracion = configuracion();
        let mut plan = plan();
        plan.personas[0].administrador = false;

        let error = comprobar_plan(&configuracion, &plan)
            .expect_err("una cuenta distinta debería rechazarse");

        assert!(error.contains("cuentas"));
        assert!(error.contains("no coinciden"));
    }

    #[test]
    fn rechaza_un_plan_con_otras_aplicaciones() {
        let configuracion = configuracion();
        let mut plan = plan();
        plan.aplicaciones.pop();

        let error = comprobar_plan(&configuracion, &plan)
            .expect_err("una lista distinta debería rechazarse");

        assert!(error.contains("no coinciden"));
    }

    #[test]
    fn una_generacion_tiene_que_estar_en_el_store() {
        let ruta = revisar_ruta_generacion(PathBuf::from(
            "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-nixos-system-korunix",
        ))
        .expect("una ruta del store debería aceptarse");

        assert!(ruta.starts_with("/nix/store"));
    }

    #[test]
    fn una_ruta_fuera_del_store_se_rechaza() {
        let error = revisar_ruta_generacion(PathBuf::from("/tmp/korunix"))
            .expect_err("una ruta fuera del store debería rechazarse");

        assert!(error.contains("no esperaba"));
    }

    #[test]
    fn capturas_conserva_las_preferencias_de_noctalia() {
        let carpeta = temporal("capturas");
        fs::create_dir_all(&carpeta).expect("debería crear la prueba");

        let ruta = carpeta.join("config.toml");
        fs::write(
            &ruta,
            r#"# Este comentario tiene que quedarse.
[shell]
offline_mode = true

[shell.screenshot]
copy_to_clipboard = false
directory = "/viejo"
filename_pattern = "Viejo %Y"
"#,
        )
        .expect("debería escribir la prueba");

        let capturas = carpeta.join("Imágenes/Capturas de pantalla");
        fusionar_capturas(&ruta, &capturas).expect("debería fusionar la política");

        let despues = fs::read_to_string(&ruta).expect("debería releer el TOML");
        let _ = fs::remove_dir_all(&carpeta);

        assert!(despues.contains("# Este comentario tiene que quedarse."));
        assert!(despues.contains("offline_mode = true"));
        assert!(despues.contains("copy_to_clipboard = false"));
        assert!(despues.contains(PATRON_CAPTURA));
        assert!(despues.contains("Capturas de pantalla"));
    }

    #[test]
    fn preparar_sesion_respeta_la_carpeta_xdg_de_imagenes() {
        let carpeta = temporal("xdg");
        let home = carpeta.join("home");
        let config_home = carpeta.join("config");
        let state_home = carpeta.join("state");
        let base = carpeta.join("base.toml");

        fs::create_dir_all(&config_home).expect("debería crear XDG_CONFIG_HOME");
        fs::write(
            config_home.join("user-dirs.dirs"),
            "XDG_PICTURES_DIR=\"$HOME/Imágenes\"\n",
        )
        .expect("debería escribir user-dirs.dirs");
        fs::write(
            &base,
            r#"# Base de prueba.
[shell.screenshot]
directory = ""
filename_pattern = ""
"#,
        )
        .expect("debería escribir la base");

        let preparada = preparar_sesion_en(&base, &home, &config_home, &state_home)
            .expect("debería preparar la sesión");

        let texto = fs::read_to_string(&preparada.configuracion_noctalia)
            .expect("debería leer la configuración");
        let capturas = preparada.capturas.clone();
        let _ = fs::remove_dir_all(&carpeta);

        assert!(capturas.ends_with("Imágenes/Capturas de pantalla"));
        assert!(texto.contains("# Base de prueba."));
        assert!(texto.contains(PATRON_CAPTURA));
    }

    #[test]
    fn preparar_sesion_conserva_el_settings_de_noctalia() {
        let carpeta = temporal("settings");
        let home = carpeta.join("home");
        let config_home = carpeta.join("config");
        let state_home = carpeta.join("state");
        let base = carpeta.join("base.toml");
        let settings_dir = state_home.join("noctalia");

        fs::create_dir_all(&settings_dir).expect("debería crear el estado");
        fs::write(
            &base,
            r#"[shell.screenshot]
directory = ""
filename_pattern = ""
"#,
        )
        .expect("debería escribir la base");
        fs::write(
            settings_dir.join("settings.toml"),
            r#"[dock]
enabled = false

[shell.screenshot]
directory = "/viejo"
filename_pattern = "viejo"
"#,
        )
        .expect("debería escribir settings.toml");

        preparar_sesion_en(&base, &home, &config_home, &state_home)
            .expect("debería preparar la sesión");

        let despues = fs::read_to_string(settings_dir.join("settings.toml"))
            .expect("debería releer settings.toml");
        let _ = fs::remove_dir_all(&carpeta);

        assert!(despues.contains("[dock]"));
        assert!(despues.contains("enabled = false"));
        assert!(despues.contains(PATRON_CAPTURA));
    }
}

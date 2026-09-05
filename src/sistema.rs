use crate::configuracion::Configuracion;
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use toml_edit::{value, Array, DocumentMut, Item, Table};

const PATRON_CAPTURA: &str = "Captura de pantalla del %Y-%m-%d %H-%M-%S";

#[derive(Debug, Deserialize)]
pub struct Plan {
    pub nombre: String,
    pub canal: String,
    pub escritorio: String,
    pub escritorios: Vec<String>,
    pub personas: Vec<PersonaPlan>,
    pub revision: String,
    pub aplicaciones: Vec<Aplicacion>,
    pub noctalia: bool,
    pub noctalia_version: String,
    pub apariencia: AparienciaPlan,
    pub idioma: IdiomaPlan,
    pub teclado: TecladoPlan,
    pub monitor: MonitorPlan,
    pub entrada: EntradaPlan,
    pub almacenamiento: Vec<UnidadPlan>,
    pub bluetooth: bool,
    pub sunshine: SunshinePlan,
    pub steam: SteamPlan,
    pub impresion: ImpresionPlan,
    pub virtualizacion: bool,
}

#[derive(Debug, Deserialize)]
pub struct AparienciaPlan {
    pub estilo: String,
    pub modo: String,
    pub noctalia_source: String,
    pub noctalia_mode: String,
}

#[derive(Debug, Deserialize)]
pub struct IdiomaPlan {
    pub sistema: String,
    pub region: String,
    pub locale: String,
    pub zona_horaria: String,
}

#[derive(Debug, Deserialize)]
pub struct TecladoPlan {
    pub distribuciones: Vec<String>,
    pub cambio: String,
    pub xkb: Vec<String>,
    pub variantes: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct MonitorPlan {
    pub resolucion: String,
    pub hz: u32,
}

#[derive(Debug, Deserialize)]
pub struct EntradaPlan {
    pub backend: String,
    pub wayland: bool,
}

#[derive(Debug, Deserialize)]
pub struct UnidadPlan {
    pub nombre: String,
    pub ruta: String,
}

#[derive(Debug, Deserialize)]
pub struct SunshinePlan {
    pub activo: bool,
    pub autoinicio: bool,
}

#[derive(Debug, Deserialize)]
pub struct SteamPlan {
    pub activo: bool,
    pub remote_play: bool,
    pub servidor_dedicado: bool,
}

#[derive(Debug, Deserialize)]
pub struct ImpresionPlan {
    pub activa: bool,
    pub controlador: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PersonaPlan {
    pub cuenta: String,
    pub administrador: bool,
    pub avatar: Option<String>,
    pub clave_github: Option<String>,
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

    let escritorios_esperados = configuracion.escritorio.instalados_efectivos();
    let escritorios_resueltos: Vec<&str> = plan.escritorios.iter().map(String::as_str).collect();

    if escritorios_resueltos != escritorios_esperados {
        return Err(
            "Los escritorios instalados que resolvió Nix no coinciden con configuracion.toml."
                .to_string(),
        );
    }

    let noctalia_esperado = escritorios_esperados
        .iter()
        .any(|escritorio| matches!(*escritorio, "niri" | "hyprland"));

    if plan.noctalia != noctalia_esperado {
        return Err(
            "Noctalia no coincide con el escritorio elegido. No voy a usar ese plan.".to_string(),
        );
    }

    let source_esperado = match configuracion.apariencia.estilo.as_str() {
        "dinamico" => "wallpaper",
        "everforest" => "community",
        _ => "builtin",
    };

    let modo_esperado = match configuracion.apariencia.modo.as_str() {
        "claro" => "light",
        "oscuro" => "dark",
        _ => "auto",
    };

    if plan.apariencia.estilo != configuracion.apariencia.estilo
        || plan.apariencia.modo != configuracion.apariencia.modo
        || plan.apariencia.noctalia_source != source_esperado
        || plan.apariencia.noctalia_mode != modo_esperado
    {
        return Err(
            "La apariencia que resolvió Nix no coincide con configuracion.toml.".to_string(),
        );
    }

    if plan.idioma.sistema != configuracion.idioma.sistema
        || plan.idioma.region != configuracion.idioma.region
    {
        return Err(
            "El idioma que resolvió Nix no coincide con configuracion.toml. No voy a usar ese plan."
                .to_string(),
        );
    }

    if plan.teclado.distribuciones != configuracion.teclado.distribuciones
        || plan.teclado.cambio != configuracion.teclado.cambio
    {
        return Err(
            "El teclado que resolvió Nix no coincide con configuracion.toml. No voy a usar ese plan."
                .to_string(),
        );
    }

    let mut xkb_esperado = Vec::new();
    let mut variantes_esperadas = Vec::new();

    for distribucion in &configuracion.teclado.distribuciones {
        match distribucion.as_str() {
            "españa" => {
                xkb_esperado.push("es".to_string());
                variantes_esperadas.push("deadtilde".to_string());
            }
            "latinoamérica" => {
                xkb_esperado.push("latam".to_string());
                variantes_esperadas.push(String::new());
            }
            _ => {}
        }
    }

    if plan.teclado.xkb != xkb_esperado || plan.teclado.variantes != variantes_esperadas {
        return Err(
            "La traducción técnica del teclado no coincide con las distribuciones elegidas."
                .to_string(),
        );
    }

    if plan.monitor.resolucion != configuracion.monitor.resolucion
        || plan.monitor.hz != configuracion.monitor.hz
    {
        return Err(
            "El monitor que resolvió Nix no coincide con configuracion.toml. No voy a usar ese plan."
                .to_string(),
        );
    }

    if plan.entrada.backend != "ibus" || !plan.entrada.wayland {
        return Err("El método de entrada no conserva IBus con su frontend Wayland.".to_string());
    }

    let unidades: Vec<&str> = plan
        .almacenamiento
        .iter()
        .map(|unidad| unidad.nombre.as_str())
        .collect();

    let unidades_esperadas: Vec<&str> = configuracion
        .almacenamiento
        .disponibles
        .iter()
        .map(String::as_str)
        .collect();

    if unidades != unidades_esperadas {
        return Err(
            "Las unidades disponibles que resolvió Nix no coinciden con configuracion.toml."
                .to_string(),
        );
    }

    if plan.bluetooth != configuracion.bluetooth.activo {
        return Err("Bluetooth no coincide con configuracion.toml.".to_string());
    }

    if plan.sunshine.activo != configuracion.sunshine.activo
        || plan.sunshine.autoinicio != configuracion.sunshine.autoinicio
    {
        return Err("Sunshine no coincide con configuracion.toml.".to_string());
    }

    if plan.steam.activo != configuracion.steam.activo
        || plan.steam.remote_play != configuracion.steam.remote_play
        || plan.steam.servidor_dedicado != configuracion.steam.servidor_dedicado
    {
        return Err("Steam no coincide con configuracion.toml.".to_string());
    }

    if plan.impresion.activa != configuracion.impresion.activa
        || plan.impresion.controlador != configuracion.impresion.controlador
    {
        return Err("La impresión no coincide con configuracion.toml.".to_string());
    }

    if plan.virtualizacion != configuracion.virtualizacion.activa {
        return Err("La virtualización no coincide con configuracion.toml.".to_string());
    }

    let personas: Vec<(&str, bool, Option<&str>, Option<&str>)> = plan
        .personas
        .iter()
        .map(|persona| {
            (
                persona.cuenta.as_str(),
                persona.administrador,
                persona.avatar.as_deref(),
                persona.clave_github.as_deref(),
            )
        })
        .collect();

    let esperadas: Vec<(&str, bool, Option<&str>, Option<&str>)> = configuracion
        .personas
        .iter()
        .map(|persona| {
            (
                persona.cuenta.as_str(),
                persona.administrador,
                persona.avatar.as_deref(),
                persona.clave_github.as_deref(),
            )
        })
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

fn fusionar_noctalia(
    ruta: &Path,
    capturas: &Path,
    tema: Option<(&str, &str)>,
    tiene_avatar: bool,
    spicetify: Option<bool>,
) -> Result<(), String> {
    let texto = fs::read_to_string(ruta)
        .map_err(|error| format!("No pude leer {}.\nDetalle: {error}", ruta.display()))?;

    let mut documento = texto.parse::<DocumentMut>().map_err(|error| {
        format!(
            "No pude entender {}.\nNo se cambió ese archivo.\nDetalle: {error}",
            ruta.display()
        )
    })?;

    {
        let shell = tabla(documento.as_table_mut(), "shell", "shell")?;
        let screenshot = tabla(shell, "screenshot", "shell.screenshot")?;

        screenshot["directory"] = value(capturas.to_string_lossy().to_string());
        screenshot["filename_pattern"] = value(PATRON_CAPTURA);

        if tiene_avatar {
            shell["avatar_path"] = value("~/.face");
        }
    }

    if let Some((source, mode)) = tema {
        let theme = tabla(documento.as_table_mut(), "theme", "theme")?;
        theme["source"] = value(source);
        theme["mode"] = value(mode);
    }

    if let Some(activo) = spicetify {
        let theme = tabla(documento.as_table_mut(), "theme", "theme")?;
        let templates = tabla(theme, "templates", "theme.templates")?;
        let community_ids = templates
            .entry("community_ids")
            .or_insert(value(Array::new()))
            .as_array_mut()
            .ok_or_else(|| {
                "No pude usar theme.templates.community_ids. Tiene que ser una lista.".to_string()
            })?;

        if activo {
            if !community_ids
                .iter()
                .any(|item| item.as_str() == Some("spicetify"))
            {
                community_ids.push("spicetify");
            }
        } else {
            loop {
                let indice = community_ids
                    .iter()
                    .position(|item| item.as_str() == Some("spicetify"));

                let Some(indice) = indice else {
                    break;
                };

                community_ids.remove(indice);
            }
        }
    }

    guardar_toml(ruta, &documento.to_string())
}

fn fusionar_capturas(ruta: &Path, capturas: &Path) -> Result<(), String> {
    fusionar_noctalia(ruta, capturas, None, false, None)
}

fn escapar_kdl(valor: &str) -> String {
    valor.replace('\\', "\\\\").replace('"', "\\\"")
}

fn preparar_capturas_niri(config_home: &Path, capturas: &Path) -> Result<PathBuf, String> {
    let ruta = config_home.join("niri/korunix-screenshots.kdl");
    let destino = escapar_kdl(&capturas.to_string_lossy());
    let patron = escapar_kdl(PATRON_CAPTURA);
    let texto = format!(
        "// Korunix mantiene Niri y Noctalia en la misma carpeta de capturas.\n\
         screenshot-path \"{destino}/{patron}.png\"\n"
    );

    guardar_toml(&ruta, &texto)?;
    Ok(ruta)
}

fn enlazar_integracion_noctalia(destino: &Path, origen: &Path) -> Result<(), String> {
    if !origen.exists() {
        return Ok(());
    }

    if let Some(carpeta) = destino.parent() {
        fs::create_dir_all(carpeta).map_err(|error| {
            format!("No pude preparar {}.\nDetalle: {error}", carpeta.display())
        })?;
    }

    if destino.is_symlink() {
        let actual = fs::read_link(destino)
            .map_err(|error| format!("No pude revisar {}.\nDetalle: {error}", destino.display()))?;

        if actual.starts_with("/etc/korunix/noctalia") {
            fs::remove_file(destino).map_err(|error| {
                format!(
                    "No pude actualizar {}.\nDetalle: {error}",
                    destino.display()
                )
            })?;
        } else {
            return Ok(());
        }
    } else if destino.exists() {
        return Ok(());
    }

    std::os::unix::fs::symlink(origen, destino)
        .map_err(|error| format!("No pude enlazar {}.\nDetalle: {error}", destino.display()))
}

fn preparar_sesion_en_con_politica(
    base: &Path,
    home: &Path,
    config_home: &Path,
    state_home: &Path,
    tema: Option<(&str, &str)>,
    tiene_avatar: bool,
    spotify_activo: bool,
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

    preparar_capturas_niri(config_home, &capturas)?;

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

    fusionar_noctalia(
        &configuracion_noctalia,
        &capturas,
        tema,
        tiene_avatar,
        Some(spotify_activo),
    )?;

    let settings = state_home.join("noctalia/settings.toml");

    if settings.is_file() {
        fusionar_noctalia(&settings, &capturas, tema, tiene_avatar, None)?;
    }

    enlazar_integracion_noctalia(
        &noctalia_dir.join("30-korunix-gtk4-live.toml"),
        Path::new("/etc/korunix/noctalia/gtk4-live.toml"),
    )?;

    enlazar_integracion_noctalia(
        &home.join(".obsidian/snippets/noctalia.css"),
        Path::new("/etc/korunix/noctalia/themes/obsidian/obsidian.css"),
    )?;

    enlazar_integracion_noctalia(
        &config_home.join("heroic/themes/noctalia.css"),
        Path::new("/etc/korunix/noctalia/themes/heroic/heroic.css"),
    )?;

    Ok(SesionPreparada {
        configuracion_noctalia,
        capturas,
    })
}

fn preparar_sesion_en(
    base: &Path,
    home: &Path,
    config_home: &Path,
    state_home: &Path,
) -> Result<SesionPreparada, String> {
    preparar_sesion_en_con_politica(base, home, config_home, state_home, None, false, false)
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

    let tema = match (
        env::var("KORUNIX_NOCTALIA_SOURCE").ok(),
        env::var("KORUNIX_NOCTALIA_MODE").ok(),
    ) {
        (Some(source), Some(mode)) => Some((source, mode)),
        _ => None,
    };

    let tiene_avatar = home.join(".face").exists();
    let spotify_activo = env::var("KORUNIX_SPOTIFY_ACTIVO")
        .map(|valor| valor == "1")
        .unwrap_or(false);

    preparar_sesion_en_con_politica(
        &base,
        &home,
        &config_home,
        &state_home,
        tema.as_ref()
            .map(|(source, mode)| (source.as_str(), mode.as_str())),
        tiene_avatar,
        spotify_activo,
    )
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::configuracion::{
        Almacenamiento, Apariencia, Aplicaciones, Bluetooth, Escritorio, Idioma, Impresion,
        Monitor, Persona, Steam, Sunshine, Teclado, Virtualizacion,
    };
    use std::time::{SystemTime, UNIX_EPOCH};

    fn configuracion() -> Configuracion {
        Configuracion {
            nombre: "korunix".to_string(),
            canal: "inestable".to_string(),
            personas: vec![Persona {
                cuenta: "koru".to_string(),
                nombre: "André".to_string(),
                administrador: true,
                avatar: Some("avatar-koru.jpg".to_string()),
                clave_github: Some(".ssh/blep".to_string()),
            }],
            escritorio: Escritorio {
                principal: "niri".to_string(),
                instalados: vec![
                    "niri".to_string(),
                    "hyprland".to_string(),
                    "plasma".to_string(),
                    "cinnamon".to_string(),
                ],
            },
            apariencia: Apariencia {
                estilo: "dinamico".to_string(),
                modo: "automatico".to_string(),
            },
            idioma: Idioma::default(),
            teclado: Teclado::default(),
            monitor: Monitor::default(),
            almacenamiento: Almacenamiento {
                disponibles: vec!["datos".to_string()],
            },
            bluetooth: Bluetooth { activo: true },
            sunshine: Sunshine {
                activo: true,
                autoinicio: true,
            },
            steam: Steam {
                activo: true,
                remote_play: true,
                servidor_dedicado: true,
            },
            impresion: Impresion {
                activa: true,
                controlador: Some("epson-201207w".to_string()),
            },
            virtualizacion: Virtualizacion { activa: true },
            aplicaciones: Aplicaciones {
                instaladas: vec!["firefox".to_string(), "whatsapp".to_string()],
            },
        }
    }

    fn plan() -> Plan {
        Plan {
            nombre: "korunix".to_string(),
            canal: "inestable".to_string(),
            escritorio: "niri".to_string(),
            escritorios: vec![
                "niri".to_string(),
                "hyprland".to_string(),
                "plasma".to_string(),
                "cinnamon".to_string(),
            ],
            personas: vec![PersonaPlan {
                cuenta: "koru".to_string(),
                administrador: true,
                avatar: Some("avatar-koru.jpg".to_string()),
                clave_github: Some(".ssh/blep".to_string()),
            }],
            revision: "abc123".to_string(),
            aplicaciones: vec![
                Aplicacion {
                    elegida: "firefox".to_string(),
                    nombre: "firefox".to_string(),
                    version: "1".to_string(),
                },
                Aplicacion {
                    elegida: "whatsapp".to_string(),
                    nombre: "whatsapp".to_string(),
                    version: "PWA".to_string(),
                },
            ],
            noctalia: true,
            noctalia_version: "5".to_string(),
            apariencia: AparienciaPlan {
                estilo: "dinamico".to_string(),
                modo: "automatico".to_string(),
                noctalia_source: "wallpaper".to_string(),
                noctalia_mode: "auto".to_string(),
            },
            idioma: IdiomaPlan {
                sistema: "español".to_string(),
                region: "Perú".to_string(),
                locale: "es_PE.UTF-8".to_string(),
                zona_horaria: "America/Lima".to_string(),
            },
            teclado: TecladoPlan {
                distribuciones: vec!["españa".to_string(), "latinoamérica".to_string()],
                cambio: "alt+shift".to_string(),
                xkb: vec!["es".to_string(), "latam".to_string()],
                variantes: vec!["deadtilde".to_string(), "".to_string()],
            },
            monitor: MonitorPlan {
                resolucion: "1920x1080".to_string(),
                hz: 120,
            },
            entrada: EntradaPlan {
                backend: "ibus".to_string(),
                wayland: true,
            },
            almacenamiento: vec![UnidadPlan {
                nombre: "datos".to_string(),
                ruta: "/mnt/datos".to_string(),
            }],
            bluetooth: true,
            sunshine: SunshinePlan {
                activo: true,
                autoinicio: true,
            },
            steam: SteamPlan {
                activo: true,
                remote_play: true,
                servidor_dedicado: true,
            },
            impresion: ImpresionPlan {
                activa: true,
                controlador: Some("epson-201207w".to_string()),
            },
            virtualizacion: true,
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
        let texto = r#"{
          "nombre": "korunix",
          "canal": "inestable",
          "escritorio": "niri",
          "escritorios": ["niri", "hyprland", "plasma", "cinnamon"],
          "personas": [{
            "cuenta": "koru",
            "administrador": true,
            "avatar": "avatar-koru.jpg",
            "clave_github": ".ssh/blep"
          }],
          "revision": "abc123",
          "aplicaciones": [
            {"elegida": "firefox", "nombre": "firefox", "version": "1"}
          ],
          "noctalia": true,
          "noctalia_version": "5",
          "apariencia": {
            "estilo": "dinamico",
            "modo": "automatico",
            "noctalia_source": "wallpaper",
            "noctalia_mode": "auto"
          },
          "idioma": {
            "sistema": "español",
            "region": "Perú",
            "locale": "es_PE.UTF-8",
            "zona_horaria": "America/Lima"
          },
          "teclado": {
            "distribuciones": ["españa", "latinoamérica"],
            "cambio": "alt+shift",
            "xkb": ["es", "latam"],
            "variantes": ["deadtilde", ""]
          },
          "monitor": {
            "resolucion": "1920x1080",
            "hz": 120
          },
          "entrada": {
            "backend": "ibus",
            "wayland": true
          },
          "almacenamiento": [
            {"nombre": "datos", "ruta": "/mnt/datos"}
          ],
          "bluetooth": true,
          "sunshine": {
            "activo": true,
            "autoinicio": true
          },
          "steam": {
            "activo": true,
            "remote_play": true,
            "servidor_dedicado": true
          },
          "impresion": {
            "activa": true,
            "controlador": "epson-201207w"
          },
          "virtualizacion": true
        }"#;

        let plan = leer_plan(texto.as_bytes()).expect("el plan debería entenderse");

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
    fn rechaza_un_plan_con_otro_teclado() {
        let configuracion = configuracion();
        let mut plan = plan();
        plan.teclado.distribuciones = vec!["latinoamérica".to_string()];

        let error = comprobar_plan(&configuracion, &plan)
            .expect_err("un teclado distinto debería rechazarse");

        assert!(error.contains("teclado"));
    }

    #[test]
    fn rechaza_un_plan_sin_ibus_wayland() {
        let configuracion = configuracion();
        let mut plan = plan();
        plan.entrada.wayland = false;

        let error =
            comprobar_plan(&configuracion, &plan).expect_err("IBus Wayland debería conservarse");

        assert!(error.contains("IBus"));
        assert!(error.contains("Wayland"));
    }

    #[test]
    fn rechaza_un_plan_con_otros_escritorios_instalados() {
        let configuracion = configuracion();
        let mut plan = plan();
        plan.escritorios.pop();

        let error = comprobar_plan(&configuracion, &plan)
            .expect_err("una lista distinta debería rechazarse");

        assert!(error.contains("escritorios instalados"));
    }

    #[test]
    fn rechaza_un_plan_con_otras_unidades() {
        let configuracion = configuracion();
        let mut plan = plan();
        plan.almacenamiento.clear();

        let error = comprobar_plan(&configuracion, &plan)
            .expect_err("una unidad perdida debería rechazarse");

        assert!(error.contains("unidades disponibles"));
    }

    #[test]
    fn rechaza_un_plan_con_otro_steam() {
        let configuracion = configuracion();
        let mut plan = plan();
        plan.steam.remote_play = false;

        let error =
            comprobar_plan(&configuracion, &plan).expect_err("Steam distinto debería rechazarse");

        assert!(error.contains("Steam"));
    }

    #[test]
    fn rechaza_un_plan_con_otro_sunshine() {
        let configuracion = configuracion();
        let mut plan = plan();
        plan.sunshine.autoinicio = false;

        let error = comprobar_plan(&configuracion, &plan)
            .expect_err("Sunshine distinto debería rechazarse");

        assert!(error.contains("Sunshine"));
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
        let niri = fs::read_to_string(config_home.join("niri/korunix-screenshots.kdl"))
            .expect("debería generar las capturas de Niri");

        assert!(capturas.ends_with("Imágenes/Capturas de pantalla"));
        assert!(texto.contains("# Base de prueba."));
        assert!(texto.contains(PATRON_CAPTURA));
        assert!(niri.contains("Imágenes/Capturas de pantalla"));
        assert!(niri.contains(PATRON_CAPTURA));
        assert!(niri.contains("screenshot-path"));

        let _ = fs::remove_dir_all(&carpeta);
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

    #[test]
    fn spotify_activo_agrega_spicetify_sin_borrar_otras_plantillas() {
        let carpeta = temporal("spicetify-activo");
        let home = carpeta.join("home");
        let config_home = carpeta.join("config");
        let state_home = carpeta.join("state");
        let base = carpeta.join("base.toml");

        fs::create_dir_all(&config_home).expect("debería crear XDG_CONFIG_HOME");
        fs::write(
            &base,
            r#"[shell.screenshot]
directory = ""
filename_pattern = ""

[theme.templates]
community_ids = ["steam", "vscode"]
"#,
        )
        .expect("debería escribir la base");

        preparar_sesion_en_con_politica(&base, &home, &config_home, &state_home, None, false, true)
            .expect("debería preparar Spicetify");

        let texto = fs::read_to_string(config_home.join("noctalia/config.toml"))
            .expect("debería leer la configuración");
        let documento = texto
            .parse::<DocumentMut>()
            .expect("el TOML debería seguir siendo válido");
        let ids = documento["theme"]["templates"]["community_ids"]
            .as_array()
            .expect("community_ids debería seguir siendo una lista");

        let valores: Vec<&str> = ids.iter().filter_map(|item| item.as_str()).collect();
        let _ = fs::remove_dir_all(&carpeta);

        assert!(valores.contains(&"steam"));
        assert!(valores.contains(&"vscode"));
        assert!(valores.contains(&"spicetify"));
    }

    #[test]
    fn spotify_apagado_retira_solo_spicetify() {
        let carpeta = temporal("spicetify-apagado");
        let ruta = carpeta.join("config.toml");
        fs::create_dir_all(&carpeta).expect("debería crear la prueba");
        fs::write(
            &ruta,
            r#"[shell.screenshot]
directory = ""
filename_pattern = ""

[theme.templates]
community_ids = ["steam", "spicetify", "vscode"]
"#,
        )
        .expect("debería escribir la prueba");

        let capturas = carpeta.join("Capturas");
        fusionar_noctalia(&ruta, &capturas, None, false, Some(false))
            .expect("debería retirar solo Spicetify");

        let texto = fs::read_to_string(&ruta).expect("debería releer la configuración");
        let documento = texto
            .parse::<DocumentMut>()
            .expect("el TOML debería seguir siendo válido");
        let ids = documento["theme"]["templates"]["community_ids"]
            .as_array()
            .expect("community_ids debería seguir siendo una lista");

        let valores: Vec<&str> = ids.iter().filter_map(|item| item.as_str()).collect();
        let _ = fs::remove_dir_all(&carpeta);

        assert!(valores.contains(&"steam"));
        assert!(valores.contains(&"vscode"));
        assert!(!valores.contains(&"spicetify"));
    }

    #[test]
    fn rechaza_un_plan_con_otra_apariencia() {
        let configuracion = configuracion();
        let mut plan = plan();
        plan.apariencia.noctalia_source = "community".to_string();

        let error = comprobar_plan(&configuracion, &plan)
            .expect_err("una apariencia distinta debería rechazarse");

        assert!(error.contains("apariencia"));
    }

    #[test]
    fn rechaza_un_plan_con_otro_bluetooth() {
        let configuracion = configuracion();
        let mut plan = plan();
        plan.bluetooth = false;

        let error = comprobar_plan(&configuracion, &plan)
            .expect_err("Bluetooth distinto debería rechazarse");

        assert!(error.contains("Bluetooth"));
    }

    #[test]
    fn preparar_sesion_aplica_apariencia_y_avatar_sin_borrar_preferencias() {
        let carpeta = temporal("apariencia");
        let home = carpeta.join("home");
        let config_home = carpeta.join("config");
        let state_home = carpeta.join("state");
        let base = carpeta.join("base.toml");
        let settings_dir = state_home.join("noctalia");

        fs::create_dir_all(&home).expect("debería crear HOME");
        fs::create_dir_all(&settings_dir).expect("debería crear el estado");
        fs::write(home.join(".face"), b"avatar").expect("debería crear el avatar");

        fs::write(
            &base,
            r#"[shell.screenshot]
directory = ""
filename_pattern = ""
"#,
        )
        .expect("debería escribir la base");

        fs::create_dir_all(config_home.join("noctalia")).expect("debería crear Noctalia");
        fs::write(
            config_home.join("noctalia/config.toml"),
            r#"[dock]
enabled = false

[theme]
source = "community"
mode = "light"
"#,
        )
        .expect("debería escribir config.toml");

        fs::write(
            settings_dir.join("settings.toml"),
            r#"[bar]
enabled = true

[theme]
source = "community"
mode = "light"
"#,
        )
        .expect("debería escribir settings.toml");

        preparar_sesion_en_con_politica(
            &base,
            &home,
            &config_home,
            &state_home,
            Some(("wallpaper", "auto")),
            true,
            false,
        )
        .expect("debería preparar la sesión");

        let config = fs::read_to_string(config_home.join("noctalia/config.toml"))
            .expect("debería leer config.toml");
        let settings = fs::read_to_string(settings_dir.join("settings.toml"))
            .expect("debería leer settings.toml");
        let _ = fs::remove_dir_all(&carpeta);

        for texto in [&config, &settings] {
            assert!(texto.contains("source = \"wallpaper\""));
            assert!(texto.contains("mode = \"auto\""));
            assert!(texto.contains("avatar_path = \"~/.face\""));
        }

        assert!(config.contains("[dock]"));
        assert!(config.contains("enabled = false"));
        assert!(settings.contains("[bar]"));
        assert!(settings.contains("enabled = true"));
    }
}

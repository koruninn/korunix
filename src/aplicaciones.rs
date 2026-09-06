use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ficha {
    pub id: &'static str,
    pub nombre: &'static str,
    pub descripcion: &'static str,
    pub categoria: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VistaAplicacion {
    pub id: String,
    pub nombre: String,
    pub descripcion: String,
    pub categoria: String,
    pub instalada: bool,
    pub curada: bool,
    pub icono: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PresentacionLocal {
    pub nombre: Option<String>,
    pub descripcion: Option<String>,
    pub icono: Option<String>,
}

thread_local! {
    static PRESENTACIONES_LOCALES: RefCell<HashMap<String, PresentacionLocal>> =
        RefCell::new(HashMap::new());
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AplicacionResuelta {
    pub id: String,
    pub nombre: String,
}

const CATEGORIAS: &[&str] = &[
    "Internet y comunicación",
    "Oficina y estudio",
    "Diseño",
    "Multimedia",
    "Juegos",
    "Dispositivos",
    "Desarrollo",
    "Archivos y utilidades",
    "Utilidades",
    "Otras elegidas",
];

// Esta lista rescata la presentación humana comprobada en la rama pruebas.
// No es una lista blanca: cualquier elección que no esté aquí se conserva y
// puede resolverse explícitamente con Nixpkgs.
const CATALOGO: &[Ficha] = &[
    Ficha {
        id: "firefox",
        nombre: "Firefox",
        descripcion: "Navegador web para páginas, enlaces y aplicaciones web.",
        categoria: "Internet y comunicación",
    },
    Ficha {
        id: "google-chrome",
        nombre: "Google Chrome",
        descripcion: "Navegador web alternativo para páginas, enlaces y servicios web.",
        categoria: "Internet y comunicación",
    },
    Ficha {
        id: "vesktop",
        nombre: "Vesktop",
        descripcion: "Cliente de Discord para conversaciones, llamadas y comunidades.",
        categoria: "Internet y comunicación",
    },
    Ficha {
        id: "localsend",
        nombre: "LocalSend",
        descripcion: "Envía archivos entre dispositivos cercanos mediante la red local.",
        categoria: "Internet y comunicación",
    },
    Ficha {
        id: "whatsapp",
        nombre: "WhatsApp",
        descripcion: "Abre WhatsApp como aplicación web integrada en el escritorio.",
        categoria: "Internet y comunicación",
    },
    Ficha {
        id: "onlyoffice-desktopeditors",
        nombre: "ONLYOFFICE",
        descripcion: "Suite para crear y editar documentos, hojas de cálculo y presentaciones.",
        categoria: "Oficina y estudio",
    },
    Ficha {
        id: "libreoffice",
        nombre: "LibreOffice",
        descripcion:
            "Suite ofimática compatible para documentos, hojas de cálculo y presentaciones.",
        categoria: "Oficina y estudio",
    },
    Ficha {
        id: "obsidian",
        nombre: "Obsidian",
        descripcion: "Organiza notas en Markdown y bases personales de conocimiento.",
        categoria: "Oficina y estudio",
    },
    Ficha {
        id: "polyglot",
        nombre: "Polyglot",
        descripcion: "Herramientas para diseñar, documentar y mantener lenguas construidas.",
        categoria: "Oficina y estudio",
    },
    Ficha {
        id: "cohesion",
        nombre: "Cohesion",
        descripcion: "Cliente de escritorio no oficial para trabajar con Notion en Linux.",
        categoria: "Oficina y estudio",
    },
    Ficha {
        id: "birdfont",
        nombre: "Birdfont",
        descripcion: "Crea y edita fuentes tipográficas.",
        categoria: "Diseño",
    },
    Ficha {
        id: "darktable",
        nombre: "Darktable",
        descripcion: "Organiza y revela fotografías, especialmente archivos RAW.",
        categoria: "Diseño",
    },
    Ficha {
        id: "figma-linux-next",
        nombre: "Figma",
        descripcion:
            "Cliente de escritorio no oficial para trabajar con diseños y prototipos de Figma.",
        categoria: "Diseño",
    },
    Ficha {
        id: "fontforge",
        nombre: "FontForge",
        descripcion: "Crea, inspecciona y modifica tipografías.",
        categoria: "Diseño",
    },
    Ficha {
        id: "inkscape",
        nombre: "Inkscape",
        descripcion: "Crea ilustraciones, diagramas y gráficos vectoriales.",
        categoria: "Diseño",
    },
    Ficha {
        id: "krita",
        nombre: "Krita",
        descripcion: "Aplicación de dibujo, ilustración y pintura digital.",
        categoria: "Diseño",
    },
    Ficha {
        id: "rapidraw",
        nombre: "RapidRAW",
        descripcion: "Revela y ajusta fotografías RAW mediante un flujo de edición visual.",
        categoria: "Diseño",
    },
    Ficha {
        id: "kdenlive",
        nombre: "Kdenlive",
        descripcion: "Editor de vídeo multipista para montar y producir proyectos audiovisuales.",
        categoria: "Multimedia",
    },
    Ficha {
        id: "vlc",
        nombre: "VLC",
        descripcion: "Reproductor multimedia opcional para una amplia variedad de formatos.",
        categoria: "Multimedia",
    },
    Ficha {
        id: "obs-studio",
        nombre: "OBS Studio",
        descripcion:
            "Graba la pantalla, cámaras y otras fuentes, y permite realizar transmisiones.",
        categoria: "Multimedia",
    },
    Ficha {
        id: "spotify",
        nombre: "Spotify",
        descripcion: "Cliente de Spotify con la integración funcional administrada por Korunix.",
        categoria: "Multimedia",
    },
    Ficha {
        id: "heroic",
        nombre: "Heroic Games Launcher",
        descripcion:
            "Administra bibliotecas de juegos compatibles desde un lanzador de escritorio.",
        categoria: "Juegos",
    },
    Ficha {
        id: "lutris",
        nombre: "Lutris",
        descripcion: "Organiza y ejecuta juegos de distintas fuentes y entornos de compatibilidad.",
        categoria: "Juegos",
    },
    Ficha {
        id: "prismlauncher",
        nombre: "Prism Launcher",
        descripcion: "Administra instalaciones, perfiles e instancias de Minecraft.",
        categoria: "Juegos",
    },
    Ficha {
        id: "protonplus",
        nombre: "ProtonPlus",
        descripcion:
            "Administra herramientas de compatibilidad como Proton para juegos y aplicaciones.",
        categoria: "Juegos",
    },
    Ficha {
        id: "genshin-impact",
        nombre: "Genshin Impact",
        descripcion: "Instala el lanzador compatible; Korunix gestiona AAGL por debajo.",
        categoria: "Juegos",
    },
    Ficha {
        id: "honkai-star-rail",
        nombre: "Honkai: Star Rail",
        descripcion: "Instala el lanzador compatible; Korunix gestiona AAGL por debajo.",
        categoria: "Juegos",
    },
    Ficha {
        id: "scrcpy",
        nombre: "Controlar Android",
        descripcion: "Muestra y controla un dispositivo Android conectado desde el equipo.",
        categoria: "Dispositivos",
    },
    Ficha {
        id: "valent",
        nombre: "Valent",
        descripcion: "Conecta el teléfono con el escritorio para compartir funciones compatibles.",
        categoria: "Dispositivos",
    },
    Ficha {
        id: "vscode",
        nombre: "Visual Studio Code",
        descripcion:
            "Editor de código con extensiones, terminal integrada y herramientas de desarrollo.",
        categoria: "Desarrollo",
    },
    Ficha {
        id: "peazip",
        nombre: "PeaZip",
        descripcion: "Gestor avanzado para crear, abrir y convertir archivos comprimidos.",
        categoria: "Archivos y utilidades",
    },
    Ficha {
        id: "baobab",
        nombre: "Uso del disco",
        descripcion: "Muestra qué carpetas y archivos están ocupando espacio en el equipo.",
        categoria: "Archivos y utilidades",
    },
    Ficha {
        id: "gnome-characters",
        nombre: "Caracteres",
        descripcion: "Busca y copia símbolos, caracteres especiales y emoji.",
        categoria: "Utilidades",
    },
    Ficha {
        id: "gnome-clocks",
        nombre: "Relojes",
        descripcion: "Ofrece relojes, alarmas, temporizadores y cronómetro.",
        categoria: "Utilidades",
    },
    Ficha {
        id: "gnome-disk-utility",
        nombre: "Discos",
        descripcion: "Inspecciona unidades y permite tareas avanzadas sobre discos y particiones.",
        categoria: "Utilidades",
    },
    Ficha {
        id: "gnome-font-viewer",
        nombre: "Tipografías",
        descripcion: "Previsualiza archivos tipográficos antes de instalarlos o utilizarlos.",
        categoria: "Utilidades",
    },
    Ficha {
        id: "gnome-weather",
        nombre: "Tiempo",
        descripcion: "Consulta el tiempo actual y el pronóstico de las ubicaciones elegidas.",
        categoria: "Utilidades",
    },
    Ficha {
        id: "simple-scan",
        nombre: "Escáner",
        descripcion: "Digitaliza documentos e imágenes con escáneres compatibles.",
        categoria: "Utilidades",
    },
];

pub fn categorias() -> &'static [&'static str] {
    CATEGORIAS
}

pub fn catalogo() -> &'static [Ficha] {
    CATALOGO
}

const TERMINOS_SUNSHINE: &str =
    "sunshine transmisión remoto remota acceso autoinicio iniciar automáticamente";
const TERMINOS_STEAM: &str = "steam juegos remote play servidor dedicado gamemode millennium";

pub fn coincide_especial(id: &str, consulta: &str) -> bool {
    let consulta = consulta.trim().to_lowercase();
    if consulta.is_empty() {
        return true;
    }

    let terminos = match id {
        "sunshine" => TERMINOS_SUNSHINE,
        "steam" => TERMINOS_STEAM,
        _ => return false,
    };

    terminos.contains(&consulta)
}

pub fn hay_especial(consulta: &str) -> bool {
    coincide_especial("sunshine", consulta) || coincide_especial("steam", consulta)
}

pub fn terminos_busqueda() -> String {
    let mut terminos = CATALOGO
        .iter()
        .flat_map(|ficha| [ficha.id, ficha.nombre, ficha.descripcion, ficha.categoria])
        .collect::<Vec<_>>()
        .join(" ");

    terminos.push(' ');
    terminos.push_str(TERMINOS_SUNSHINE);
    terminos.push(' ');
    terminos.push_str(TERMINOS_STEAM);

    terminos.to_lowercase()
}

pub fn guardar_presentaciones_locales(presentaciones: Vec<(String, PresentacionLocal)>) {
    PRESENTACIONES_LOCALES.with(|guardadas| {
        let mut guardadas = guardadas.borrow_mut();
        guardadas.clear();
        guardadas.extend(presentaciones);
    });
}

fn presentacion_local(id: &str) -> Option<PresentacionLocal> {
    PRESENTACIONES_LOCALES.with(|guardadas| guardadas.borrow().get(id).cloned())
}

#[cfg(feature = "interfaz")]
pub fn leer_appstream_local(ids: &[String]) -> Result<Vec<(String, PresentacionLocal)>, String> {
    use libappstream::prelude::*;

    let pool = libappstream::Pool::new();
    pool.set_load_std_data_locations(true);
    pool.load(None::<&libappstream::gio::Cancellable>)
        .map_err(|error| format!("No pude leer AppStream local.\nDetalle: {error}"))?;

    let mut resultado = Vec::new();

    for id in ids {
        let Some(componentes) = pool.search(id) else {
            continue;
        };

        let id_desktop = format!("{id}.desktop");
        let mut mejor: Option<(bool, PresentacionLocal)> = None;

        for indice in 0..componentes.size() {
            let Some(componente) = componentes.index_safe(indice) else {
                continue;
            };

            let paquete_exacto = componente
                .pkgnames()
                .iter()
                .any(|paquete| paquete.as_str() == id);
            let id_exacto = componente
                .id()
                .as_deref()
                .map(|actual| actual == id || actual == id_desktop)
                .unwrap_or(false);

            if !paquete_exacto && !id_exacto {
                continue;
            }

            let icono = componente
                .icon_stock()
                .and_then(|icono| icono.name())
                .map(|nombre| nombre.to_string());

            // Los textos del catálogo curado ya están escritos en español.
            // Nombre y descripción de AppStream solo se guardan cuando el
            // paquete coincide exactamente; una coincidencia por .desktop
            // puede aportar icono, pero no reemplaza texto humano fiable.
            let presentacion = PresentacionLocal {
                nombre: paquete_exacto
                    .then(|| componente.name().map(|valor| valor.to_string()))
                    .flatten(),
                descripcion: paquete_exacto
                    .then(|| componente.summary().map(|valor| valor.to_string()))
                    .flatten(),
                icono,
            };

            let reemplazar = mejor
                .as_ref()
                .map(|(era_paquete, _)| paquete_exacto && !*era_paquete)
                .unwrap_or(true);

            if reemplazar {
                mejor = Some((paquete_exacto, presentacion));
            }

            if paquete_exacto {
                break;
            }
        }

        if let Some((_, presentacion)) = mejor {
            resultado.push((id.clone(), presentacion));
        }
    }

    Ok(resultado)
}

fn capitalizar(palabra: &str) -> String {
    let mut caracteres = palabra.chars();
    let Some(inicial) = caracteres.next() else {
        return String::new();
    };

    inicial.to_uppercase().collect::<String>() + caracteres.as_str()
}

pub fn nombre_desde_id(id: &str) -> String {
    id.split(['-', '_'])
        .filter(|parte| !parte.is_empty())
        .map(capitalizar)
        .collect::<Vec<_>>()
        .join(" ")
}

fn orden_categoria(categoria: &str) -> usize {
    CATEGORIAS
        .iter()
        .position(|actual| *actual == categoria)
        .unwrap_or(CATEGORIAS.len())
}

pub fn vistas(instaladas: &[String], consulta: &str) -> Vec<VistaAplicacion> {
    let instaladas_set: HashSet<&str> = instaladas.iter().map(String::as_str).collect();
    let ids_curados: HashSet<&str> = CATALOGO.iter().map(|ficha| ficha.id).collect();
    let consulta = consulta.trim().to_lowercase();

    let mut resultado = CATALOGO
        .iter()
        .map(|ficha| {
            let local = presentacion_local(ficha.id);
            VistaAplicacion {
                id: ficha.id.to_string(),
                nombre: ficha.nombre.to_string(),
                descripcion: ficha.descripcion.to_string(),
                categoria: ficha.categoria.to_string(),
                instalada: instaladas_set.contains(ficha.id),
                curada: true,
                icono: local.and_then(|presentacion| presentacion.icono),
            }
        })
        .collect::<Vec<_>>();

    for id in instaladas {
        if ids_curados.contains(id.as_str()) {
            continue;
        }

        let local = presentacion_local(id);
        resultado.push(VistaAplicacion {
            id: id.clone(),
            nombre: local
                .as_ref()
                .and_then(|presentacion| presentacion.nombre.clone())
                .unwrap_or_else(|| nombre_desde_id(id)),
            descripcion: local
                .as_ref()
                .and_then(|presentacion| presentacion.descripcion.clone())
                .unwrap_or_else(|| {
                    "Aplicación elegida. Korunix conserva tu elección aunque no tenga ficha curada."
                        .to_string()
                }),
            categoria: "Otras elegidas".to_string(),
            instalada: true,
            curada: false,
            icono: local.and_then(|presentacion| presentacion.icono),
        });
    }

    if !consulta.is_empty() {
        resultado.retain(|vista| {
            [
                vista.id.as_str(),
                vista.nombre.as_str(),
                vista.descripcion.as_str(),
                vista.categoria.as_str(),
            ]
            .into_iter()
            .any(|texto| texto.to_lowercase().contains(&consulta))
        });
    }

    resultado.sort_by(|a, b| {
        orden_categoria(&a.categoria)
            .cmp(&orden_categoria(&b.categoria))
            .then_with(|| a.nombre.to_lowercase().cmp(&b.nombre.to_lowercase()))
    });

    resultado
}

pub fn nombre_valido_para_resolver(nombre: &str) -> bool {
    let nombre = nombre.trim();
    !nombre.is_empty()
        && nombre.chars().all(|caracter| {
            caracter.is_ascii_alphanumeric() || matches!(caracter, '-' | '_' | '.' | '+')
        })
}

pub fn resolver_nixpkgs(
    raiz: &Path,
    canal: &str,
    nombre: &str,
) -> Result<Option<AplicacionResuelta>, String> {
    let nombre = nombre.trim();

    if !nombre_valido_para_resolver(nombre) {
        return Err(
            "Escribe un nombre de paquete sin espacios, por ejemplo «karere» o «blender»."
                .to_string(),
        );
    }

    if !matches!(canal, "estable" | "inestable") {
        return Err(format!("No conozco el canal «{canal}»."));
    }

    const EXPRESION: &str = r#"
let
  flake = builtins.getFlake (builtins.getEnv "KORUNIX_FLAKE");
  canal = builtins.getEnv "KORUNIX_CANAL";
  nombre = builtins.getEnv "KORUNIX_APLICACION";
  nixpkgs =
    if canal == "estable"
    then flake.inputs.nixpkgs-estable
    else flake.inputs.nixpkgs-inestable;
  pkgs = import nixpkgs {
    system = builtins.currentSystem;
    config.allowUnfree = true;
  };
  partes = pkgs.lib.splitString "." nombre;
  paquete = pkgs.lib.attrByPath partes null pkgs;
in
  if paquete == null || !(pkgs.lib.isDerivation paquete)
  then ""
  else if paquete ? pname && paquete.pname != null
  then builtins.toString paquete.pname
  else nombre
"#;

    let salida = Command::new("nix")
        .args([
            "--extra-experimental-features",
            "nix-command flakes",
            "eval",
            "--raw",
            "--impure",
            "--expr",
            EXPRESION,
        ])
        .env("KORUNIX_FLAKE", format!("path:{}", raiz.display()))
        .env("KORUNIX_CANAL", canal)
        .env("KORUNIX_APLICACION", nombre)
        .current_dir(raiz)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| {
            format!("No pude iniciar Nix para comprobar «{nombre}».\nDetalle: {error}")
        })?;

    if !salida.status.success() {
        let error = String::from_utf8_lossy(&salida.stderr);
        let resumen = error
            .lines()
            .filter(|linea| !linea.trim().is_empty())
            .take(3)
            .collect::<Vec<_>>()
            .join("\n");

        return Err(if resumen.is_empty() {
            format!("Nix no pudo comprobar «{nombre}».")
        } else {
            format!("Nix no pudo comprobar «{nombre}».\n{resumen}")
        });
    }

    let nombre_humano = String::from_utf8_lossy(&salida.stdout).trim().to_string();

    if nombre_humano.is_empty() {
        Ok(None)
    } else {
        Ok(Some(AplicacionResuelta {
            id: nombre.to_string(),
            nombre: nombre_humano,
        }))
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn la_busqueda_global_conoce_aplicaciones_y_opciones_propias() {
        let terminos = terminos_busqueda();

        assert!(terminos.contains("firefox"));
        assert!(terminos.contains("fotografías"));
        assert!(terminos.contains("steam"));
        assert!(terminos.contains("sunshine"));
    }

    #[test]
    fn las_opciones_propias_participan_en_la_busqueda_local() {
        assert!(coincide_especial("steam", "remote play"));
        assert!(coincide_especial("sunshine", "remoto"));
        assert!(!hay_especial("karere"));
    }

    #[test]
    fn buscar_usa_nombre_descripcion_y_categoria() {
        let instaladas = vec!["firefox".to_string()];

        assert_eq!(vistas(&instaladas, "Firefox")[0].id, "firefox");
        assert!(vistas(&instaladas, "fotografías")
            .iter()
            .any(|vista| vista.id == "darktable"));
        assert!(vistas(&instaladas, "Juegos")
            .iter()
            .any(|vista| vista.id == "heroic"));
    }

    #[test]
    fn una_eleccion_libre_no_desaparece_por_no_tener_ficha() {
        let instaladas = vec!["blender".to_string()];
        let resultado = vistas(&instaladas, "blender");

        assert_eq!(resultado.len(), 1);
        assert_eq!(resultado[0].id, "blender");
        assert!(resultado[0].instalada);
        assert!(!resultado[0].curada);
        assert_eq!(resultado[0].categoria, "Otras elegidas");
    }

    #[test]
    fn el_catalogo_no_convierte_dependencias_en_elecciones() {
        let vacias = Vec::<String>::new();

        assert!(vistas(&vacias, "android-tools").is_empty());
        assert!(vistas(&vacias, "kate").is_empty());
    }

    #[test]
    fn un_nombre_libre_se_valida_sin_aceptar_comandos() {
        assert!(nombre_valido_para_resolver("karere"));
        assert!(nombre_valido_para_resolver("kdePackages.kate"));
        assert!(!nombre_valido_para_resolver("foo;rm -rf"));
        assert!(!nombre_valido_para_resolver("dos palabras"));
    }
}

//! PARTE INTERNA DE KORUNIX.
//!
//! Esta interfaz presenta información y solicita acciones al motor público
//! `korunix`. No evalúa Nix ni mantiene una segunda implementación del sistema.

use adw::prelude::*;
use adw::{gio, glib};
use serde_json::Value;
use std::cell::Cell;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::rc::Rc;

const APPLICATION_ID: &str = "io.github.koruninn.Korunix";

#[derive(Clone, Copy)]
enum Idioma {
    Espanol,
    Ingles,
    Hungaro,
}

fn idioma_actual() -> Idioma {
    let valor = env::var("LANG").unwrap_or_default().to_ascii_lowercase();

    if valor.starts_with("hu") {
        Idioma::Hungaro
    } else if valor.starts_with("en") {
        Idioma::Ingles
    } else {
        Idioma::Espanol
    }
}

fn texto(idioma: Idioma, clave: &str) -> &'static str {
    match (idioma, clave) {
        (Idioma::Ingles, "subtitle") => "NixOS control center",
        (Idioma::Ingles, "summary") => "Summary",
        (Idioma::Ingles, "updates") => "Updates",
        (Idioma::Ingles, "localization") => "Language and region",
        (Idioma::Ingles, "hardware") => "Hardware",
        (Idioma::Ingles, "people") => "People",
        (Idioma::Ingles, "refresh") => "Refresh",
        (Idioma::Ingles, "channel") => "System channel",
        (Idioma::Ingles, "stable") => "Stable",
        (Idioma::Ingles, "unstable") => "Unstable",
        (Idioma::Ingles, "prepare") => "Prepare change",
        (Idioma::Ingles, "current") => "Current",
        (Idioma::Ingles, "host") => "Computer",
        (Idioma::Ingles, "model") => "Model",
        (Idioma::Ingles, "cpu") => "Processor",
        (Idioma::Ingles, "memory") => "Memory",
        (Idioma::Ingles, "language") => "Language",
        (Idioma::Ingles, "region") => "Region",
        (Idioma::Ingles, "timezone") => "Time zone",
        (Idioma::Ingles, "keyboard") => "Keyboard",
        (Idioma::Ingles, "status") => "Status",
        (Idioma::Ingles, "loading") => "Reading this computer…",
        (Idioma::Ingles, "ready") => "Ready",
        (Idioma::Ingles, "error") => "Korunix could not read this area.",
        (Idioma::Ingles, "empty") => "No information available.",

        (Idioma::Hungaro, "subtitle") => "NixOS vezérlőközpont",
        (Idioma::Hungaro, "summary") => "Összefoglaló",
        (Idioma::Hungaro, "updates") => "Frissítések",
        (Idioma::Hungaro, "localization") => "Nyelv és régió",
        (Idioma::Hungaro, "hardware") => "Hardver",
        (Idioma::Hungaro, "people") => "Személyek",
        (Idioma::Hungaro, "refresh") => "Frissítés",
        (Idioma::Hungaro, "channel") => "Rendszercsatorna",
        (Idioma::Hungaro, "stable") => "Stabil",
        (Idioma::Hungaro, "unstable") => "Instabil",
        (Idioma::Hungaro, "prepare") => "Változtatás előkészítése",
        (Idioma::Hungaro, "current") => "Jelenlegi",
        (Idioma::Hungaro, "host") => "Számítógép",
        (Idioma::Hungaro, "model") => "Modell",
        (Idioma::Hungaro, "cpu") => "Processzor",
        (Idioma::Hungaro, "memory") => "Memória",
        (Idioma::Hungaro, "language") => "Nyelv",
        (Idioma::Hungaro, "region") => "Régió",
        (Idioma::Hungaro, "timezone") => "Időzóna",
        (Idioma::Hungaro, "keyboard") => "Billentyűzet",
        (Idioma::Hungaro, "status") => "Állapot",
        (Idioma::Hungaro, "loading") => "A számítógép adatainak olvasása…",
        (Idioma::Hungaro, "ready") => "Kész",
        (Idioma::Hungaro, "error") => "A Korunix nem tudta beolvasni ezt a területet.",
        (Idioma::Hungaro, "empty") => "Nincs elérhető információ.",

        (_, "subtitle") => "Centro de control de NixOS",
        (_, "summary") => "Resumen",
        (_, "updates") => "Actualizaciones",
        (_, "localization") => "Idioma y región",
        (_, "hardware") => "Hardware",
        (_, "people") => "Personas",
        (_, "refresh") => "Actualizar",
        (_, "channel") => "Canal del sistema",
        (_, "stable") => "Estable",
        (_, "unstable") => "Inestable",
        (_, "prepare") => "Preparar cambio",
        (_, "current") => "Actual",
        (_, "host") => "Equipo",
        (_, "model") => "Modelo",
        (_, "cpu") => "Procesador",
        (_, "memory") => "Memoria",
        (_, "language") => "Idioma",
        (_, "region") => "Región",
        (_, "timezone") => "Zona horaria",
        (_, "keyboard") => "Teclado",
        (_, "status") => "Estado",
        (_, "loading") => "Leyendo este equipo…",
        (_, "ready") => "Listo",
        (_, "error") => "Korunix no pudo leer esta área.",
        (_, "empty") => "No hay información disponible.",
        _ => "Korunix",
    }
}

fn raiz_proyecto() -> Result<PathBuf, String> {
    if let Some(valor) = env::var_os("KORUNIX_ROOT") {
        let ruta = PathBuf::from(valor);
        if ruta.join("flake.nix").is_file() {
            return Ok(ruta);
        }
    }

    if let Ok(actual) = env::current_dir() {
        if actual.join("flake.nix").is_file() {
            return Ok(actual);
        }
    }

    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME no está disponible.".to_string())?;

    let ruta = home.join(".korunix");

    if ruta.join("flake.nix").is_file() {
        Ok(ruta)
    } else {
        Err("No encuentro el checkout de Korunix.".to_string())
    }
}

fn motor(raiz: &Path) -> Result<PathBuf, String> {
    if let Some(valor) = env::var_os("KORUNIX_MOTOR_BIN") {
        let ruta = PathBuf::from(valor);
        if ruta.is_file() {
            return Ok(ruta);
        }
    }

    let desarrollo = raiz.join("target/debug/korunix");

    if desarrollo.is_file() {
        return Ok(desarrollo);
    }

    Err("No encuentro el motor Rust de Korunix.".to_string())
}

struct Estado {
    raiz: PathBuf,
    motor: PathBuf,
    idioma: Idioma,
    stack: gtk::Stack,
    estado: gtk::Label,
    cargando: Cell<bool>,
}

fn ejecutar_motor(estado: &Estado, argumentos: &[&str]) -> Result<String, String> {
    let salida = Command::new(&estado.motor)
        .args(argumentos)
        .current_dir(&estado.raiz)
        .env("KORUNIX_ROOT", &estado.raiz)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| format!("No pude iniciar el motor: {error}"))?;

    if !salida.status.success() {
        let error = String::from_utf8_lossy(&salida.stderr).trim().to_string();

        return Err(if error.is_empty() {
            "El motor terminó con error.".to_string()
        } else {
            error
        });
    }

    Ok(String::from_utf8_lossy(&salida.stdout).trim().to_string())
}

fn consultar(estado: &Estado, area: &str) -> Result<Value, String> {
    let salida = ejecutar_motor(estado, &[area, "--json"])?;

    serde_json::from_str(&salida)
        .map_err(|error| format!("El motor devolvió JSON inválido para {area}: {error}"))
}

fn valor(datos: &Value, puntero: &str) -> String {
    let Some(valor) = datos.pointer(puntero) else {
        return "—".to_string();
    };

    match valor {
        Value::Null => "—".to_string(),
        Value::String(texto) if texto.is_empty() => "—".to_string(),
        Value::String(texto) => texto.clone(),
        Value::Bool(valor) => valor.to_string(),
        Value::Number(valor) => valor.to_string(),
        Value::Array(valores) => valores
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join(", "),
        otro => otro.to_string(),
    }
}

fn memoria_humana(datos: &Value) -> String {
    let bytes = datos
        .pointer("/memory/bytes")
        .and_then(Value::as_u64)
        .unwrap_or(0);

    if bytes == 0 {
        "—".to_string()
    } else {
        format!("{:.1} GiB", bytes as f64 / 1024_f64.powi(3),)
    }
}

fn fila(titulo: &str, contenido: impl AsRef<str>) -> adw::ActionRow {
    let fila = adw::ActionRow::new();
    fila.set_title(titulo);
    fila.set_subtitle(contenido.as_ref());
    fila
}

fn pagina_error(idioma: Idioma, detalle: &str) -> adw::PreferencesPage {
    let pagina = adw::PreferencesPage::new();
    let grupo = adw::PreferencesGroup::new();

    grupo.set_title(texto(idioma, "error"));
    grupo.add(&fila(texto(idioma, "status"), detalle));

    pagina.add(&grupo);
    pagina
}

fn pagina_resumen(
    estado: &Estado,
    hardware: &Value,
    people: &Value,
    channel: &Value,
) -> adw::PreferencesPage {
    let pagina = adw::PreferencesPage::new();
    let grupo = adw::PreferencesGroup::new();
    grupo.set_title(texto(estado.idioma, "summary"));

    grupo.add(&fila(
        texto(estado.idioma, "host"),
        valor(hardware, "/hostId"),
    ));

    let vendor = valor(hardware, "/machine/vendor");
    let model = valor(hardware, "/machine/model");
    let modelo = format!("{vendor} {model}",).trim().to_string();

    grupo.add(&fila(texto(estado.idioma, "model"), modelo));

    grupo.add(&fila(
        texto(estado.idioma, "channel"),
        valor(channel, "/label"),
    ));

    let personas = people
        .pointer("/accounts")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);

    grupo.add(&fila(texto(estado.idioma, "people"), personas.to_string()));

    pagina.add(&grupo);
    pagina
}

fn pagina_hardware(estado: &Estado, hardware: &Value) -> adw::PreferencesPage {
    let pagina = adw::PreferencesPage::new();
    let grupo = adw::PreferencesGroup::new();
    grupo.set_title(texto(estado.idioma, "hardware"));

    let vendor = valor(hardware, "/machine/vendor");
    let model = valor(hardware, "/machine/model");
    let modelo = format!("{vendor} {model}",).trim().to_string();

    grupo.add(&fila(texto(estado.idioma, "model"), modelo));
    grupo.add(&fila(
        texto(estado.idioma, "cpu"),
        valor(hardware, "/cpu/model"),
    ));
    grupo.add(&fila(
        texto(estado.idioma, "memory"),
        memoria_humana(hardware),
    ));
    grupo.add(&fila("Firmware", valor(hardware, "/firmware/detected")));

    pagina.add(&grupo);
    pagina
}

fn pagina_localizacion(estado: &Estado, datos: &Value) -> adw::PreferencesPage {
    let pagina = adw::PreferencesPage::new();
    let grupo = adw::PreferencesGroup::new();
    grupo.set_title(texto(estado.idioma, "localization"));

    grupo.add(&fila(
        texto(estado.idioma, "language"),
        valor(datos, "/declared/systemLanguage"),
    ));
    grupo.add(&fila(
        texto(estado.idioma, "region"),
        valor(datos, "/declared/region"),
    ));
    grupo.add(&fila(
        texto(estado.idioma, "timezone"),
        valor(datos, "/declared/timeZone"),
    ));
    grupo.add(&fila(
        texto(estado.idioma, "keyboard"),
        valor(datos, "/derived/keyboard/layout"),
    ));

    pagina.add(&grupo);
    pagina
}

fn pagina_personas(estado: &Estado, datos: &Value) -> adw::PreferencesPage {
    let pagina = adw::PreferencesPage::new();
    let grupo = adw::PreferencesGroup::new();
    grupo.set_title(texto(estado.idioma, "people"));

    let Some(cuentas) = datos.pointer("/accounts").and_then(Value::as_array) else {
        grupo.add(&fila(
            texto(estado.idioma, "status"),
            texto(estado.idioma, "empty"),
        ));
        pagina.add(&grupo);
        return pagina;
    };

    if cuentas.is_empty() {
        grupo.add(&fila(
            texto(estado.idioma, "status"),
            texto(estado.idioma, "empty"),
        ));
    } else {
        for cuenta in cuentas {
            let nombre = cuenta
                .get("displayName")
                .and_then(Value::as_str)
                .or_else(|| cuenta.get("accountName").and_then(Value::as_str))
                .unwrap_or("—");

            let cuenta_id = cuenta
                .get("accountName")
                .and_then(Value::as_str)
                .unwrap_or("—");

            let status = cuenta.get("status").and_then(Value::as_str).unwrap_or("");

            let subtitulo = if status.is_empty() {
                cuenta_id.to_string()
            } else {
                format!("{cuenta_id} · {status}",)
            };

            grupo.add(&fila(nombre, subtitulo));
        }
    }

    pagina.add(&grupo);
    pagina
}

fn reemplazar_pagina(
    stack: &gtk::Stack,
    nombre: &str,
    titulo: &str,
    pagina: &adw::PreferencesPage,
) {
    if let Some(anterior) = stack.child_by_name(nombre) {
        stack.remove(&anterior);
    }

    stack.add_titled(pagina, Some(nombre), titulo);
}

fn pagina_actualizaciones(estado: Rc<Estado>, channel: &Value) -> adw::PreferencesPage {
    let pagina = adw::PreferencesPage::new();
    let grupo = adw::PreferencesGroup::new();
    grupo.set_title(texto(estado.idioma, "updates"));

    let actual = channel
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("stable")
        .to_string();

    grupo.add(&fila(
        texto(estado.idioma, "current"),
        channel
            .get("label")
            .and_then(Value::as_str)
            .unwrap_or(&actual),
    ));

    let fila_canal = adw::ActionRow::new();
    fila_canal.set_title(texto(estado.idioma, "channel"));

    let selector = gtk::DropDown::from_strings(&[
        texto(estado.idioma, "stable"),
        texto(estado.idioma, "unstable"),
    ]);

    selector.set_selected(if actual == "unstable" { 1 } else { 0 });

    let boton = gtk::Button::with_label(texto(estado.idioma, "prepare"));
    boton.add_css_class("suggested-action");

    fila_canal.add_suffix(&selector);
    fila_canal.add_suffix(&boton);
    grupo.add(&fila_canal);

    let estado_clon = Rc::clone(&estado);

    boton.connect_clicked(move |boton| {
        boton.set_sensitive(false);

        let destino = if selector.selected() == 0 {
            "stable"
        } else {
            "unstable"
        };

        match ejecutar_motor(&estado_clon, &["channel", destino, "--yes"]) {
            Ok(_) => {
                estado_clon
                    .estado
                    .set_label(texto(estado_clon.idioma, "ready"));

                recargar(Rc::clone(&estado_clon));
            }
            Err(error) => {
                estado_clon.estado.set_label(&error);
                boton.set_sensitive(true);
            }
        }
    });

    pagina.add(&grupo);
    pagina
}

fn recargar(estado: Rc<Estado>) {
    if estado.cargando.get() {
        return;
    }

    estado.cargando.set(true);
    estado.estado.set_label(texto(estado.idioma, "loading"));

    let hardware = consultar(&estado, "hardware");
    let localization = consultar(&estado, "localization");
    let people = consultar(&estado, "users");
    let channel = consultar(&estado, "channel");

    if let (Ok(hardware), Ok(people), Ok(channel)) = (&hardware, &people, &channel) {
        reemplazar_pagina(
            &estado.stack,
            "summary",
            texto(estado.idioma, "summary"),
            &pagina_resumen(&estado, hardware, people, channel),
        );
    } else {
        reemplazar_pagina(
            &estado.stack,
            "summary",
            texto(estado.idioma, "summary"),
            &pagina_error(estado.idioma, texto(estado.idioma, "error")),
        );
    }

    match hardware {
        Ok(datos) => {
            reemplazar_pagina(
                &estado.stack,
                "hardware",
                texto(estado.idioma, "hardware"),
                &pagina_hardware(&estado, &datos),
            );
        }
        Err(error) => {
            reemplazar_pagina(
                &estado.stack,
                "hardware",
                texto(estado.idioma, "hardware"),
                &pagina_error(estado.idioma, &error),
            );
        }
    }

    match localization {
        Ok(datos) => {
            reemplazar_pagina(
                &estado.stack,
                "localization",
                texto(estado.idioma, "localization"),
                &pagina_localizacion(&estado, &datos),
            );
        }
        Err(error) => {
            reemplazar_pagina(
                &estado.stack,
                "localization",
                texto(estado.idioma, "localization"),
                &pagina_error(estado.idioma, &error),
            );
        }
    }

    match people {
        Ok(datos) => {
            reemplazar_pagina(
                &estado.stack,
                "people",
                texto(estado.idioma, "people"),
                &pagina_personas(&estado, &datos),
            );
        }
        Err(error) => {
            reemplazar_pagina(
                &estado.stack,
                "people",
                texto(estado.idioma, "people"),
                &pagina_error(estado.idioma, &error),
            );
        }
    }

    match channel {
        Ok(datos) => {
            reemplazar_pagina(
                &estado.stack,
                "updates",
                texto(estado.idioma, "updates"),
                &pagina_actualizaciones(Rc::clone(&estado), &datos),
            );
        }
        Err(error) => {
            reemplazar_pagina(
                &estado.stack,
                "updates",
                texto(estado.idioma, "updates"),
                &pagina_error(estado.idioma, &error),
            );
        }
    }

    estado.estado.set_label(texto(estado.idioma, "ready"));
    estado.cargando.set(false);
}

fn construir_ventana(app: &adw::Application, raiz: PathBuf, motor: PathBuf) {
    let idioma = idioma_actual();

    let ventana = adw::ApplicationWindow::builder()
        .application(app)
        .title("Korunix")
        .default_width(980)
        .default_height(680)
        .build();

    ventana.set_size_request(360, 520);

    let toolbar = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();

    let titulo = adw::WindowTitle::new("Korunix", texto(idioma, "subtitle"));

    header.set_title_widget(Some(&titulo));

    let refresh = gtk::Button::from_icon_name("view-refresh-symbolic");

    refresh.set_tooltip_text(Some(texto(idioma, "refresh")));

    header.pack_end(&refresh);
    toolbar.add_top_bar(&header);

    let principal = gtk::Box::new(gtk::Orientation::Horizontal, 0);

    let sidebar = gtk::Box::new(gtk::Orientation::Vertical, 6);

    sidebar.set_width_request(250);
    sidebar.set_margin_top(12);
    sidebar.set_margin_bottom(12);
    sidebar.set_margin_start(12);
    sidebar.set_margin_end(12);

    let stack = gtk::Stack::new();
    stack.set_hexpand(true);
    stack.set_vexpand(true);
    stack.set_transition_type(gtk::StackTransitionType::Crossfade);

    for (nombre, clave) in [
        ("summary", "summary"),
        ("updates", "updates"),
        ("localization", "localization"),
        ("hardware", "hardware"),
        ("people", "people"),
    ] {
        let boton = gtk::Button::with_label(texto(idioma, clave));

        boton.add_css_class("flat");
        boton.set_halign(gtk::Align::Fill);

        let stack_clon = stack.clone();

        boton.connect_clicked(move |_| {
            stack_clon.set_visible_child_name(nombre);
        });

        sidebar.append(&boton);

        let pagina = pagina_error(idioma, texto(idioma, "loading"));

        stack.add_titled(&pagina, Some(nombre), texto(idioma, clave));
    }

    principal.append(&sidebar);
    principal.append(&gtk::Separator::new(gtk::Orientation::Vertical));
    principal.append(&stack);

    let exterior = gtk::Box::new(gtk::Orientation::Vertical, 0);

    exterior.append(&principal);

    let estado_label = gtk::Label::new(Some(texto(idioma, "loading")));

    estado_label.add_css_class("dim-label");
    estado_label.set_halign(gtk::Align::Start);
    estado_label.set_margin_start(16);
    estado_label.set_margin_end(16);
    estado_label.set_margin_top(8);
    estado_label.set_margin_bottom(8);

    exterior.append(&estado_label);

    toolbar.set_content(Some(&exterior));
    ventana.set_content(Some(&toolbar));

    let estado = Rc::new(Estado {
        raiz,
        motor,
        idioma,
        stack,
        estado: estado_label,
        cargando: Cell::new(false),
    });

    let estado_clon = Rc::clone(&estado);

    refresh.connect_clicked(move |_| {
        recargar(Rc::clone(&estado_clon));
    });

    recargar(estado);
    ventana.present();
}

fn main() -> glib::ExitCode {
    let app = adw::Application::builder()
        .application_id(APPLICATION_ID)
        .flags(gio::ApplicationFlags::empty())
        .build();

    app.connect_activate(|app| {
        let raiz = match raiz_proyecto() {
            Ok(valor) => valor,
            Err(error) => {
                eprintln!("Korunix: {error}",);
                return;
            }
        };

        let motor = match motor(&raiz) {
            Ok(valor) => valor,
            Err(error) => {
                eprintln!("Korunix: {error}",);
                return;
            }
        };

        construir_ventana(app, raiz, motor);
    });

    app.run()
}

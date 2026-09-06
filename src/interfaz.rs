mod almacenamiento;
#[allow(dead_code)]
mod configuracion;
mod transferencias;

use adw::prelude::*;
use adw::{Application, ApplicationWindow, HeaderBar, ToolbarView};
use gtk::glib;
use std::cell::{Cell, RefCell};
use std::env;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::rc::Rc;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

const ID_APLICACION: &str = "io.github.koruninn.Korunix";

type AlTerminar = Rc<dyn Fn(bool)>;
type AlExpulsar = Rc<dyn Fn(Result<(), String>)>;

enum Mensaje {
    Linea(String),
    Terminado(bool),
}

enum MensajeTransferencia {
    Avance(transferencias::Progreso),
    Terminado(Result<PathBuf, String>),
}

fn raiz_korunix() -> PathBuf {
    if let Some(ruta) = env::var_os("KORUNIX_ROOT") {
        return PathBuf::from(ruta);
    }

    if let Some(home) = env::var_os("HOME") {
        return PathBuf::from(home).join(".korunix");
    }

    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn carpeta_estado() -> PathBuf {
    if let Some(ruta) = env::var_os("XDG_STATE_HOME") {
        return PathBuf::from(ruta).join("korunix");
    }

    if let Some(home) = env::var_os("HOME") {
        return PathBuf::from(home).join(".local/state/korunix");
    }

    PathBuf::from(".").join(".korunix-estado")
}

fn motor_korunix() -> PathBuf {
    if let Some(ruta) = env::var_os("KORUNIX_MOTOR_BIN") {
        return PathBuf::from(ruta);
    }

    PathBuf::from("korunix")
}

fn asegurar_extension_copia(ruta: PathBuf) -> PathBuf {
    let correcta = ruta
        .file_name()
        .map(|nombre| nombre.to_string_lossy().ends_with(".korunix-copia"))
        .unwrap_or(false);

    if correcta {
        return ruta;
    }

    let mut nombre = ruta
        .file_name()
        .map(|valor| valor.to_os_string())
        .unwrap_or_else(|| "Copia de Korunix".into());
    nombre.push(".korunix-copia");
    ruta.with_file_name(nombre)
}

fn nombre_archivo_humano(ruta: &Path) -> String {
    ruta.file_name()
        .map(|nombre| nombre.to_string_lossy().into_owned())
        .unwrap_or_else(|| ruta.display().to_string())
}

fn filtro_copias() -> gtk::FileFilter {
    let filtro = gtk::FileFilter::new();
    filtro.set_name(Some("Copias de Korunix"));
    filtro.add_pattern("*.korunix-copia");
    filtro
}

fn autorizador_grafico() -> Option<&'static str> {
    [
        "/run/wrappers/bin/pkexec",
        "/run/current-system/sw/bin/pkexec",
    ]
    .into_iter()
    .find(|ruta| Path::new(ruta).is_file())
}

fn configuracion_pendiente(raiz: &Path) -> bool {
    let actual = fs::read(raiz.join("configuracion.toml"));
    let preview = fs::read(carpeta_estado().join("preview-configuracion.toml"));

    match (actual, preview) {
        (Ok(actual), Ok(preview)) => actual != preview,
        _ => true,
    }
}

fn preview_actualizacion_vigente(raiz: &Path) -> bool {
    let estado = carpeta_estado();

    let Ok(base) = fs::read(estado.join("preview-flake-base.lock")) else {
        return false;
    };
    let Ok(usado) = fs::read(estado.join("preview-flake-usado.lock")) else {
        return false;
    };
    let Ok(lock_actual) = fs::read(raiz.join("flake.lock")) else {
        return false;
    };
    let Ok(configuracion_actual) = fs::read(raiz.join("configuracion.toml")) else {
        return false;
    };
    let Ok(configuracion_preview) = fs::read(estado.join("preview-configuracion.toml")) else {
        return false;
    };
    let Ok(generacion_texto) = fs::read_to_string(estado.join("preview-generacion")) else {
        return false;
    };
    let Ok(enlace) = fs::read_link(estado.join("preview")) else {
        return false;
    };

    let generacion = PathBuf::from(generacion_texto.trim());

    base != usado
        && (lock_actual == base || lock_actual == usado)
        && configuracion_actual == configuracion_preview
        && generacion.is_absolute()
        && generacion.starts_with("/nix/store")
        && enlace == generacion
}

fn actualizar_boton_aplicar_actualizacion(
    raiz: &Path,
    boton: &gtk::Button,
    ocupado: &Rc<Cell<bool>>,
) {
    boton.set_sensitive(!ocupado.get() && preview_actualizacion_vigente(raiz));
}

fn nombre_escritorio(escritorio: &str) -> &str {
    match escritorio {
        "niri" => "Niri",
        "hyprland" => "Hyprland",
        "cinnamon" => "Cinnamon",
        "plasma" => "Plasma",
        otro => otro,
    }
}

fn indice_canal(canal: &str) -> u32 {
    if canal == "inestable" {
        1
    } else {
        0
    }
}

fn canal_por_indice(indice: u32) -> Option<&'static str> {
    match indice {
        0 => Some("estable"),
        1 => Some("inestable"),
        _ => None,
    }
}

fn indice_escritorio(escritorio: &str) -> u32 {
    match escritorio {
        "niri" => 0,
        "hyprland" => 1,
        "cinnamon" => 2,
        "plasma" => 3,
        _ => 0,
    }
}

fn escritorio_por_indice(indice: u32) -> Option<&'static str> {
    match indice {
        0 => Some("niri"),
        1 => Some("hyprland"),
        2 => Some("cinnamon"),
        3 => Some("plasma"),
        _ => None,
    }
}

fn indice_estilo(estilo: &str) -> u32 {
    match estilo {
        "predeterminado" => 0,
        "dinamico" => 1,
        "everforest" => 2,
        _ => 0,
    }
}

fn estilo_por_indice(indice: u32) -> Option<&'static str> {
    match indice {
        0 => Some("predeterminado"),
        1 => Some("dinamico"),
        2 => Some("everforest"),
        _ => None,
    }
}

fn indice_modo(modo: &str) -> u32 {
    match modo {
        "claro" => 0,
        "oscuro" => 1,
        "automatico" => 2,
        _ => 2,
    }
}

fn modo_por_indice(indice: u32) -> Option<&'static str> {
    match indice {
        0 => Some("claro"),
        1 => Some("oscuro"),
        2 => Some("automatico"),
        _ => None,
    }
}

fn palabra_estado(activo: bool) -> &'static str {
    if activo {
        "activado"
    } else {
        "apagado"
    }
}

fn escritorios_elegidos(
    niri: &adw::SwitchRow,
    hyprland: &adw::SwitchRow,
    plasma: &adw::SwitchRow,
    cinnamon: &adw::SwitchRow,
) -> Vec<String> {
    [
        ("niri", niri.is_active()),
        ("hyprland", hyprland.is_active()),
        ("plasma", plasma.is_active()),
        ("cinnamon", cinnamon.is_active()),
    ]
    .into_iter()
    .filter(|(_, activo)| *activo)
    .map(|(nombre, _)| nombre.to_string())
    .collect()
}

fn teclados_elegidos(espana: &adw::SwitchRow, latinoamerica: &adw::SwitchRow) -> Vec<String> {
    [
        ("españa", espana.is_active()),
        ("latinoamérica", latinoamerica.is_active()),
    ]
    .into_iter()
    .filter(|(_, activo)| *activo)
    .map(|(nombre, _)| nombre.to_string())
    .collect()
}

fn actualizar_destinos_transferencia(
    selector: &gtk::DropDown,
    destinos: &Rc<RefCell<Vec<String>>>,
    raiz: &Path,
) -> Result<(), String> {
    let configuracion = configuracion::leer(&raiz.join("configuracion.toml"))?;
    let nombres = configuracion.almacenamiento.disponibles;
    let referencias: Vec<&str> = nombres.iter().map(String::as_str).collect();
    let modelo = gtk::StringList::new(&referencias);

    selector.set_model(Some(&modelo));

    if !nombres.is_empty() {
        selector.set_selected(0);
    }

    *destinos.borrow_mut() = nombres;
    Ok(())
}

fn iniciar_transferencia_gui(
    origen: PathBuf,
    unidad: String,
    raiz: &Path,
    progreso: &gtk::ProgressBar,
    estado: &gtk::Label,
    controles: &[gtk::Widget],
    ocupado: &Rc<Cell<bool>>,
    aviso: &gtk::Revealer,
    boton_aplicar: &gtk::Button,
) {
    if ocupado.get() {
        return;
    }

    ocupado.set(true);
    sensibilidad(controles, false);
    progreso.set_fraction(0.0);
    progreso.set_show_text(true);
    progreso.set_text(Some("Preparando la transferencia…"));

    let nombre_archivo = origen
        .file_name()
        .map(|nombre| nombre.to_string_lossy().into_owned())
        .unwrap_or_else(|| origen.display().to_string());

    estado.set_text(&format!("Copiando «{nombre_archivo}» a «{unidad}»…"));

    let (envio, recepcion) = mpsc::channel();
    let raiz_trabajo = raiz.to_path_buf();

    thread::spawn(move || {
        let envio_progreso = envio.clone();
        let resultado =
            transferencias::transferir(&raiz_trabajo, &unidad, &origen, move |avance| {
                let _ = envio_progreso.send(MensajeTransferencia::Avance(avance));
            });

        let _ = envio.send(MensajeTransferencia::Terminado(resultado));
    });

    let progreso = progreso.clone();
    let estado = estado.clone();
    let controles = controles.to_vec();
    let ocupado = Rc::clone(ocupado);
    let raiz = raiz.to_path_buf();
    let aviso = aviso.clone();
    let boton_aplicar = boton_aplicar.clone();

    glib::timeout_add_local(Duration::from_millis(80), move || {
        loop {
            match recepcion.try_recv() {
                Ok(MensajeTransferencia::Avance(avance)) => {
                    let fraccion = if avance.total == 0 {
                        1.0
                    } else {
                        avance.copiados as f64 / avance.total as f64
                    };

                    progreso.set_fraction(fraccion.clamp(0.0, 1.0));
                    progreso.set_text(Some(&avance.linea()));
                }
                Ok(MensajeTransferencia::Terminado(resultado)) => {
                    sensibilidad(&controles, true);
                    ocupado.set(false);
                    actualizar_estado_preview(&raiz, &aviso, &boton_aplicar, &ocupado);

                    match resultado {
                        Ok(destino) => {
                            progreso.set_fraction(1.0);
                            let nombre = destino
                                .file_name()
                                .map(|valor| valor.to_string_lossy().into_owned())
                                .unwrap_or_else(|| "archivo".to_string());
                            estado.set_text(&format!(
                                "✓ «{nombre}» terminó de copiarse y quedó verificado."
                            ));
                        }
                        Err(error) => {
                            progreso.set_text(Some("La transferencia no se completó."));
                            estado.set_text(&error);
                        }
                    }

                    return glib::ControlFlow::Break;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    sensibilidad(&controles, true);
                    ocupado.set(false);
                    actualizar_estado_preview(&raiz, &aviso, &boton_aplicar, &ocupado);
                    progreso.set_text(Some("La transferencia terminó sin respuesta."));
                    estado.set_text(
                        "La transferencia terminó sin respuesta. No doy el archivo por terminado.",
                    );
                    return glib::ControlFlow::Break;
                }
            }
        }

        glib::ControlFlow::Continue
    });
}

fn conectar_expulsion(
    boton: &gtk::Button,
    nombre: String,
    raiz: &Path,
    mensaje: &gtk::Label,
    aviso: &gtk::Revealer,
    boton_aplicar: &gtk::Button,
    controles: &[gtk::Widget],
    ocupado: &Rc<Cell<bool>>,
    al_terminar: AlExpulsar,
) {
    let boton = boton.clone();
    let raiz = raiz.to_path_buf();
    let mensaje = mensaje.clone();
    let aviso = aviso.clone();
    let boton_aplicar = boton_aplicar.clone();
    let controles = controles.to_vec();
    let ocupado = Rc::clone(ocupado);

    boton.clone().connect_clicked(move |_| {
        if ocupado.get() {
            return;
        }

        ocupado.set(true);
        sensibilidad(&controles, false);
        boton.set_sensitive(false);
        boton.set_label("Expulsando…");
        mensaje.set_text(&format!("Expulsando «{nombre}» con seguridad…"));

        let (envio, recepcion) = mpsc::channel();
        let nombre_trabajo = nombre.clone();

        thread::spawn(move || {
            let _ = envio.send(almacenamiento::expulsar(&nombre_trabajo));
        });

        let boton = boton.clone();
        let raiz = raiz.clone();
        let mensaje = mensaje.clone();
        let aviso = aviso.clone();
        let boton_aplicar = boton_aplicar.clone();
        let controles = controles.clone();
        let ocupado = Rc::clone(&ocupado);
        let nombre = nombre.clone();
        let al_terminar = Rc::clone(&al_terminar);

        glib::timeout_add_local(Duration::from_millis(80), move || {
            match recepcion.try_recv() {
                Ok(resultado) => {
                    sensibilidad(&controles, true);
                    ocupado.set(false);
                    actualizar_estado_preview(&raiz, &aviso, &boton_aplicar, &ocupado);

                    match &resultado {
                        Ok(()) => {
                            boton.set_label("Expulsada");
                            boton.set_sensitive(false);
                            mensaje.set_text(&format!(
                                "✓ «{nombre}» ya se puede desconectar físicamente."
                            ));
                        }
                        Err(error) => {
                            boton.set_label("Reintentar");
                            boton.set_sensitive(true);
                            mensaje.set_text(error);
                        }
                    }

                    al_terminar(resultado);
                    glib::ControlFlow::Break
                }
                Err(TryRecvError::Empty) => glib::ControlFlow::Continue,
                Err(TryRecvError::Disconnected) => {
                    sensibilidad(&controles, true);
                    ocupado.set(false);
                    actualizar_estado_preview(&raiz, &aviso, &boton_aplicar, &ocupado);
                    boton.set_label("Reintentar");
                    boton.set_sensitive(true);
                    let error =
                        "La expulsión terminó sin respuesta. No doy la unidad por expulsada."
                            .to_string();
                    mensaje.set_text(&error);
                    al_terminar(Err(error));
                    glib::ControlFlow::Break
                }
            }
        });
    });
}

fn guardar_monitor(
    resolucion: &gtk::Entry,
    hz: &gtk::SpinButton,
    raiz: &Path,
    mensaje: &gtk::Label,
    aviso: &gtk::Revealer,
    boton_aplicar: &gtk::Button,
    ocupado: &Rc<Cell<bool>>,
) {
    let resolucion = resolucion.text().to_string();
    let hz = hz.value_as_int().max(0) as u32;

    match configuracion::cambiar_monitor(&raiz.join("configuracion.toml"), &resolucion, hz) {
        Ok(true) => mensaje_guardado(
            mensaje,
            raiz,
            aviso,
            boton_aplicar,
            ocupado,
            &format!("✓ Monitor guardado como {resolucion} @ {hz} Hz. NixOS todavía no cambió."),
        ),
        Ok(false) => mensaje.set_text("El monitor ya tenía esos valores."),
        Err(error) => mensaje.set_text(&error),
    }
}

fn limpiar_lista_almacenamiento(lista: &gtk::ListBox) {
    while let Some(hijo) = lista.first_child() {
        lista.remove(&hijo);
    }
}

fn agregar_fila_almacenamiento(
    lista: &gtk::ListBox,
    filas: &Rc<RefCell<Vec<(String, adw::SwitchRow)>>>,
    nombre: String,
    detalle: String,
    activa: bool,
    expulsable: bool,
    raiz: &Path,
    mensaje: &gtk::Label,
    aviso: &gtk::Revealer,
    boton_aplicar: &gtk::Button,
    ocupado: &Rc<Cell<bool>>,
    actualizando: &Rc<Cell<bool>>,
    selector_destino: &gtk::DropDown,
    destinos_transferencia: &Rc<RefCell<Vec<String>>>,
    controles: &[gtk::Widget],
) {
    let fila = adw::SwitchRow::builder()
        .title(nombre.as_str())
        .subtitle(detalle.as_str())
        .build();
    fila.set_active(activa);

    // La misma fila tiene dos acciones distintas: guardar la preferencia y,
    // cuando es USB, expulsarla. Cada cierre recibe sus propias referencias.
    let raiz = raiz.to_path_buf();
    let mensaje = mensaje.clone();
    let aviso = aviso.clone();
    let boton_aplicar = boton_aplicar.clone();
    let ocupado = Rc::clone(ocupado);

    let raiz_senal = raiz.clone();
    let mensaje_senal = mensaje.clone();
    let aviso_senal = aviso.clone();
    let aplicar_senal = boton_aplicar.clone();
    let ocupado_senal = Rc::clone(&ocupado);
    let actualizando_senal = Rc::clone(actualizando);
    let selector_destino_senal = selector_destino.clone();
    let destinos_senal = Rc::clone(destinos_transferencia);
    let nombre_senal = nombre.clone();

    fila.connect_active_notify(move |fila| {
        if actualizando_senal.get() {
            return;
        }

        let actual = match configuracion::leer(&raiz_senal.join("configuracion.toml")) {
            Ok(configuracion) => configuracion,
            Err(error) => {
                mensaje_senal.set_text(&error);
                return;
            }
        };

        let mut disponibles = actual.almacenamiento.disponibles;

        if fila.is_active() {
            if !disponibles.iter().any(|unidad| unidad == &nombre_senal) {
                disponibles.push(nombre_senal.clone());
            }
        } else {
            disponibles.retain(|unidad| unidad != &nombre_senal);
        }

        match configuracion::cambiar_almacenamiento(
            &raiz_senal.join("configuracion.toml"),
            &disponibles,
        ) {
            Ok(true) => {
                mensaje_guardado(
                    &mensaje_senal,
                    &raiz_senal,
                    &aviso_senal,
                    &aplicar_senal,
                    &ocupado_senal,
                    &format!(
                        "✓ «{}» quedó {} en la configuración. NixOS todavía no cambió.",
                        nombre_senal,
                        if fila.is_active() {
                            "disponible"
                        } else {
                            "apagado"
                        }
                    ),
                );

                let _ = actualizar_destinos_transferencia(
                    &selector_destino_senal,
                    &destinos_senal,
                    &raiz_senal,
                );
            }
            Ok(false) => {}
            Err(error) => {
                mensaje_senal.set_text(&error);
                actualizando_senal.set(true);
                fila.set_active(!fila.is_active());
                actualizando_senal.set(false);
            }
        }
    });

    if expulsable {
        let expulsar = gtk::Button::with_label("Expulsar");
        expulsar.set_valign(gtk::Align::Center);
        fila.add_suffix(&expulsar);

        let fila_estado = fila.clone();
        let detalle_normal = detalle.clone();

        conectar_expulsion(
            &expulsar,
            nombre.clone(),
            &raiz,
            &mensaje,
            &aviso,
            &boton_aplicar,
            controles,
            &ocupado,
            Rc::new(move |resultado| match resultado {
                Ok(()) => fila_estado
                    .set_subtitle("Expulsada con seguridad · ya puedes desconectarla físicamente"),
                Err(error) => {
                    let resumen = error
                        .lines()
                        .next()
                        .unwrap_or("No pude expulsar la unidad.");
                    fila_estado.set_subtitle(&format!("{detalle_normal} · {resumen}"));
                }
            }),
        );
    }

    lista.append(&fila);
    filas.borrow_mut().push((nombre, fila));
}

fn cargar_almacenamiento(
    lista: &gtk::ListBox,
    estado: &gtk::Label,
    filas: &Rc<RefCell<Vec<(String, adw::SwitchRow)>>>,
    raiz: &Path,
    mensaje: &gtk::Label,
    aviso: &gtk::Revealer,
    boton_aplicar: &gtk::Button,
    ocupado: &Rc<Cell<bool>>,
    actualizando: &Rc<Cell<bool>>,
    controles: &[gtk::Widget],
    selector_destino: &gtk::DropDown,
    destinos_transferencia: &Rc<RefCell<Vec<String>>>,
) {
    estado.set_text("Comprobando los discos locales…");

    let (envio, recepcion) = mpsc::channel();
    thread::spawn(move || {
        let _ = envio.send(almacenamiento::leer());
    });

    let lista = lista.clone();
    let estado = estado.clone();
    let filas = Rc::clone(filas);
    let raiz = raiz.to_path_buf();
    let mensaje = mensaje.clone();
    let aviso = aviso.clone();
    let boton_aplicar = boton_aplicar.clone();
    let ocupado = Rc::clone(ocupado);
    let actualizando = Rc::clone(actualizando);
    let controles = controles.to_vec();
    let selector_destino = selector_destino.clone();
    let destinos_transferencia = Rc::clone(destinos_transferencia);

    glib::timeout_add_local(Duration::from_millis(50), move || {
        match recepcion.try_recv() {
            Ok(resultado) => {
                limpiar_lista_almacenamiento(&lista);
                filas.borrow_mut().clear();

                let configuracion = match configuracion::leer(&raiz.join("configuracion.toml")) {
                    Ok(configuracion) => configuracion,
                    Err(error) => {
                        estado.set_text(&error);
                        return glib::ControlFlow::Break;
                    }
                };

                match resultado {
                    Ok(unidades) => {
                        let mut vistas = Vec::new();

                        for unidad in unidades {
                            let conocida = match almacenamiento::administrada(&raiz, &unidad.nombre)
                            {
                                Ok(conocida) => conocida,
                                Err(error) => {
                                    estado.set_text(&error);
                                    false
                                }
                            };

                            if conocida {
                                let activa = configuracion
                                    .almacenamiento
                                    .disponibles
                                    .iter()
                                    .any(|nombre| nombre == &unidad.nombre);

                                let detalle = format!(
                                    "{} · {}",
                                    unidad.detalle,
                                    if activa {
                                        "Disponible en Korunix · se monta al usarlo"
                                    } else {
                                        "No disponible en Korunix"
                                    }
                                );

                                vistas.push(unidad.nombre.clone());
                                let expulsable = unidad.puede_expulsar();

                                agregar_fila_almacenamiento(
                                    &lista,
                                    &filas,
                                    unidad.nombre,
                                    detalle,
                                    activa,
                                    expulsable,
                                    &raiz,
                                    &mensaje,
                                    &aviso,
                                    &boton_aplicar,
                                    &ocupado,
                                    &actualizando,
                                    &selector_destino,
                                    &destinos_transferencia,
                                    &controles,
                                );
                            } else if let Some(problema) = unidad.problema_adopcion() {
                                let detalle =
                                    format!("{} · Detectado · {}", unidad.detalle, problema);
                                let fila = adw::ActionRow::builder()
                                    .title(unidad.nombre.as_str())
                                    .subtitle(detalle.as_str())
                                    .build();

                                if unidad.puede_expulsar() {
                                    let expulsar = gtk::Button::with_label("Expulsar");
                                    expulsar.set_valign(gtk::Align::Center);
                                    fila.add_suffix(&expulsar);

                                    let fila_estado = fila.clone();
                                    let detalle_normal = detalle.clone();

                                    conectar_expulsion(
                                        &expulsar,
                                        unidad.nombre.clone(),
                                        &raiz,
                                        &mensaje,
                                        &aviso,
                                        &boton_aplicar,
                                        &controles,
                                        &ocupado,
                                        Rc::new(move |resultado| {
                                            match resultado {
                                            Ok(()) => fila_estado.set_subtitle(
                                                "Expulsada con seguridad · ya puedes desconectarla físicamente",
                                            ),
                                            Err(error) => {
                                                let resumen = error
                                                    .lines()
                                                    .next()
                                                    .unwrap_or("No pude expulsar la unidad.");
                                                fila_estado.set_subtitle(&format!(
                                                    "{detalle_normal} · {resumen}"
                                                ));
                                            }
                                        }
                                        }),
                                    );
                                }

                                lista.append(&fila);
                                vistas.push(unidad.nombre);
                            } else {
                                let detalle = format!(
                                    "{} · Detectado · todavía no administrado por Korunix",
                                    unidad.detalle
                                );
                                let fila = adw::ActionRow::builder()
                                    .title(unidad.nombre.as_str())
                                    .subtitle(detalle.as_str())
                                    .build();
                                let administrar = gtk::Button::with_label("Administrar");
                                administrar.set_valign(gtk::Align::Center);
                                fila.add_suffix(&administrar);

                                if unidad.puede_expulsar() {
                                    let expulsar = gtk::Button::with_label("Expulsar");
                                    expulsar.set_valign(gtk::Align::Center);
                                    fila.add_suffix(&expulsar);

                                    let fila_estado = fila.clone();
                                    let detalle_normal = detalle.clone();

                                    conectar_expulsion(
                                        &expulsar,
                                        unidad.nombre.clone(),
                                        &raiz,
                                        &mensaje,
                                        &aviso,
                                        &boton_aplicar,
                                        &controles,
                                        &ocupado,
                                        Rc::new(move |resultado| {
                                            match resultado {
                                            Ok(()) => fila_estado.set_subtitle(
                                                "Expulsada con seguridad · ya puedes desconectarla físicamente",
                                            ),
                                            Err(error) => {
                                                let resumen = error
                                                    .lines()
                                                    .next()
                                                    .unwrap_or("No pude expulsar la unidad.");
                                                fila_estado.set_subtitle(&format!(
                                                    "{detalle_normal} · {resumen}"
                                                ));
                                            }
                                        }
                                        }),
                                    );
                                }

                                lista.append(&fila);
                                vistas.push(unidad.nombre.clone());

                                let nombre = unidad.nombre;
                                let raiz_adoptar = raiz.clone();
                                let mensaje_adoptar = mensaje.clone();
                                let aviso_adoptar = aviso.clone();
                                let aplicar_adoptar = boton_aplicar.clone();
                                let ocupado_adoptar = Rc::clone(&ocupado);
                                let controles_adoptar = controles.clone();
                                let fila_adoptar = fila.clone();
                                let boton_adoptar = administrar.clone();
                                let selector_destino_adoptar = selector_destino.clone();
                                let destinos_adoptar = Rc::clone(&destinos_transferencia);

                                administrar.connect_clicked(move |_| {
                                    if ocupado_adoptar.get() {
                                        return;
                                    }

                                    ocupado_adoptar.set(true);
                                    sensibilidad(&controles_adoptar, false);
                                    boton_adoptar.set_label("Comprobando…");
                                    boton_adoptar.set_sensitive(false);
                                    fila_adoptar.set_subtitle(
                                        "Comprobando la unidad y preparando su identidad técnica…",
                                    );
                                    mensaje_adoptar.set_text(&format!(
                                        "Comprobando «{}» y preparando su identidad técnica…",
                                        nombre
                                    ));

                                    let (envio, recepcion) = mpsc::channel();
                                    let raiz_trabajo = raiz_adoptar.clone();
                                    let nombre_trabajo = nombre.clone();

                                    thread::spawn(move || {
                                        let _ = envio.send(almacenamiento::adoptar(
                                            &raiz_trabajo,
                                            &nombre_trabajo,
                                        ));
                                    });

                                    let nombre_final = nombre.clone();
                                    let raiz_final = raiz_adoptar.clone();
                                    let mensaje_final = mensaje_adoptar.clone();
                                    let aviso_final = aviso_adoptar.clone();
                                    let aplicar_final = aplicar_adoptar.clone();
                                    let ocupado_final = Rc::clone(&ocupado_adoptar);
                                    let controles_final = controles_adoptar.clone();
                                    let fila_final = fila_adoptar.clone();
                                    let boton_final = boton_adoptar.clone();
                                    let selector_destino_final =
                                        selector_destino_adoptar.clone();
                                    let destinos_final = Rc::clone(&destinos_adoptar);

                                    glib::timeout_add_local(
                                        Duration::from_millis(80),
                                        move || match recepcion.try_recv() {
                                            Ok(resultado) => {
                                                sensibilidad(&controles_final, true);
                                                ocupado_final.set(false);

                                                match resultado {
                                                    Ok(true) => {
                                                        fila_final.set_subtitle(
                                                            "Disponible en Korunix · crea un preview para aplicarlo",
                                                        );
                                                        boton_final.set_label("Administrada");
                                                        boton_final.set_sensitive(false);
                                                        mensaje_guardado(
                                                            &mensaje_final,
                                                            &raiz_final,
                                                            &aviso_final,
                                                            &aplicar_final,
                                                            &ocupado_final,
                                                            &format!(
                                                                "✓ «{}» quedó administrada. Korunix guardó los datos técnicos por dentro. NixOS todavía no cambió.",
                                                                nombre_final
                                                            ),
                                                        );

                                                        let _ =
                                                            actualizar_destinos_transferencia(
                                                                &selector_destino_final,
                                                                &destinos_final,
                                                                &raiz_final,
                                                            );
                                                    }
                                                    Ok(false) => {
                                                        fila_final.set_subtitle(
                                                            "Esta unidad ya estaba administrada por Korunix.",
                                                        );
                                                        boton_final.set_label("Administrada");
                                                        boton_final.set_sensitive(false);
                                                        mensaje_final.set_text(
                                                            "La unidad ya estaba administrada. No cambié nada.",
                                                        );
                                                    }
                                                    Err(error) => {
                                                        let resumen = error
                                                            .lines()
                                                            .next()
                                                            .unwrap_or("No pude administrar esta unidad.");
                                                        fila_final.set_subtitle(&format!(
                                                            "No pude administrarla · {resumen}"
                                                        ));
                                                        boton_final.set_label("Reintentar");
                                                        boton_final.set_sensitive(true);
                                                        mensaje_final.set_text(&error);
                                                        actualizar_estado_preview(
                                                            &raiz_final,
                                                            &aviso_final,
                                                            &aplicar_final,
                                                            &ocupado_final,
                                                        );
                                                    }
                                                }

                                                glib::ControlFlow::Break
                                            }
                                            Err(TryRecvError::Empty) => {
                                                glib::ControlFlow::Continue
                                            }
                                            Err(TryRecvError::Disconnected) => {
                                                sensibilidad(&controles_final, true);
                                                boton_final.set_label("Reintentar");
                                                boton_final.set_sensitive(true);
                                                fila_final.set_subtitle(
                                                    "La comprobación terminó sin respuesta. No doy el cambio por hecho.",
                                                );
                                                ocupado_final.set(false);
                                                mensaje_final.set_text(
                                                    "La adopción terminó sin respuesta. No doy el cambio por hecho.",
                                                );
                                                glib::ControlFlow::Break
                                            }
                                        },
                                    );
                                });
                            }
                        }

                        for nombre in &configuracion.almacenamiento.disponibles {
                            if !vistas.iter().any(|vista| vista == nombre) {
                                agregar_fila_almacenamiento(
                                    &lista,
                                    &filas,
                                    nombre.clone(),
                                    "No está conectado ahora. Tu elección se conserva.".to_string(),
                                    true,
                                    false,
                                    &raiz,
                                    &mensaje,
                                    &aviso,
                                    &boton_aplicar,
                                    &ocupado,
                                    &actualizando,
                                    &selector_destino,
                                    &destinos_transferencia,
                                    &controles,
                                );
                            }
                        }

                        if lista.first_child().is_none() {
                            estado.set_text("No encontré discos adicionales.");
                        } else {
                            estado.set_text(
                                "Estado local leído una vez. Nix no se ejecutó para mostrar esta sección.",
                            );
                        }
                    }
                    Err(error) => {
                        for nombre in &configuracion.almacenamiento.disponibles {
                            agregar_fila_almacenamiento(
                                &lista,
                                &filas,
                                nombre.clone(),
                                "No pude comprobar si está conectado ahora.".to_string(),
                                true,
                                false,
                                &raiz,
                                &mensaje,
                                &aviso,
                                &boton_aplicar,
                                &ocupado,
                                &actualizando,
                                &selector_destino,
                                &destinos_transferencia,
                                &controles,
                            );
                        }

                        estado.set_text(&error);
                    }
                }

                glib::ControlFlow::Break
            }
            Err(TryRecvError::Empty) => glib::ControlFlow::Continue,
            Err(TryRecvError::Disconnected) => {
                estado.set_text("La lectura local de discos terminó sin respuesta.");
                glib::ControlFlow::Break
            }
        }
    });
}

fn anexar_linea(vista: &gtk::TextView, linea: &str) {
    let buffer = vista.buffer();
    let mut final_texto = buffer.end_iter();
    buffer.insert(&mut final_texto, linea);
    buffer.insert(&mut final_texto, "\n");
}

fn sensibilidad(controles: &[gtk::Widget], sensible: bool) {
    for control in controles {
        control.set_sensitive(sensible);
    }
}

fn actualizar_estado_preview(
    raiz: &Path,
    aviso: &gtk::Revealer,
    boton_aplicar: &gtk::Button,
    ocupado: &Rc<Cell<bool>>,
) {
    let pendiente = configuracion_pendiente(raiz);
    aviso.set_reveal_child(pendiente);

    if !ocupado.get() {
        boton_aplicar.set_sensitive(!pendiente);
    }
}

fn mensaje_guardado(
    mensaje: &gtk::Label,
    raiz: &Path,
    aviso: &gtk::Revealer,
    boton_aplicar: &gtk::Button,
    ocupado: &Rc<Cell<bool>>,
    texto: &str,
) {
    mensaje.set_text(texto);
    actualizar_estado_preview(raiz, aviso, boton_aplicar, ocupado);
}

fn limpiar_lista(lista: &gtk::ListBox) {
    while let Some(hijo) = lista.first_child() {
        lista.remove(&hijo);
    }
}

fn recargar_aplicaciones(
    lista: &gtk::ListBox,
    contador: &gtk::Label,
    raiz: &Path,
    mensaje: &gtk::Label,
    aviso: &gtk::Revealer,
    boton_aplicar: &gtk::Button,
    ocupado: &Rc<Cell<bool>>,
) {
    limpiar_lista(lista);

    let configuracion = match configuracion::leer(&raiz.join("configuracion.toml")) {
        Ok(configuracion) => configuracion,
        Err(error) => {
            contador.set_text("No pude leer la lista");
            mensaje.set_text(&error);
            return;
        }
    };

    contador.set_text(&format!(
        "{} elegidas",
        configuracion.aplicaciones.instaladas.len()
    ));

    for nombre in configuracion.aplicaciones.instaladas {
        let fila = gtk::ListBoxRow::new();
        let caja = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        caja.set_margin_top(6);
        caja.set_margin_bottom(6);
        caja.set_margin_start(10);
        caja.set_margin_end(8);

        let etiqueta = gtk::Label::new(Some(&nombre));
        etiqueta.set_halign(gtk::Align::Start);
        etiqueta.set_hexpand(true);
        etiqueta.set_ellipsize(gtk::pango::EllipsizeMode::End);

        let quitar = gtk::Button::with_label("Quitar");
        quitar.add_css_class("flat");

        caja.append(&etiqueta);
        caja.append(&quitar);
        fila.set_child(Some(&caja));
        lista.append(&fila);

        let lista = lista.clone();
        let contador = contador.clone();
        let raiz = raiz.to_path_buf();
        let mensaje = mensaje.clone();
        let aviso = aviso.clone();
        let boton_aplicar = boton_aplicar.clone();
        let ocupado = Rc::clone(ocupado);
        let nombre = nombre.clone();

        quitar.connect_clicked(move |_| {
            match configuracion::quitar_aplicacion(&raiz.join("configuracion.toml"), &nombre) {
                Ok(true) => {
                    mensaje_guardado(
                        &mensaje,
                        &raiz,
                        &aviso,
                        &boton_aplicar,
                        &ocupado,
                        &format!("✓ Quité «{nombre}». NixOS todavía no cambió."),
                    );

                    recargar_aplicaciones(
                        &lista,
                        &contador,
                        &raiz,
                        &mensaje,
                        &aviso,
                        &boton_aplicar,
                        &ocupado,
                    );
                }
                Ok(false) => mensaje.set_text(&format!("«{nombre}» ya no estaba en la lista.")),
                Err(error) => mensaje.set_text(&error),
            }
        });
    }
}

fn guardar_nombre(
    entrada: &gtk::Entry,
    raiz: &Path,
    mensaje: &gtk::Label,
    aviso: &gtk::Revealer,
    boton_aplicar: &gtk::Button,
    ocupado: &Rc<Cell<bool>>,
) {
    let nombre = entrada.text().to_string();

    match configuracion::cambiar_nombre(&raiz.join("configuracion.toml"), &nombre) {
        Ok(true) => mensaje_guardado(
            mensaje,
            raiz,
            aviso,
            boton_aplicar,
            ocupado,
            &format!("✓ El equipo ahora se llama «{nombre}» en la configuración. NixOS todavía no cambió."),
        ),
        Ok(false) => mensaje.set_text("Ese nombre ya estaba guardado."),
        Err(error) => mensaje.set_text(&error),
    }
}

fn agregar_aplicacion(
    entrada: &gtk::Entry,
    lista: &gtk::ListBox,
    contador: &gtk::Label,
    raiz: &Path,
    mensaje: &gtk::Label,
    aviso: &gtk::Revealer,
    boton_aplicar: &gtk::Button,
    ocupado: &Rc<Cell<bool>>,
) {
    let nombre = entrada.text().to_string();

    match configuracion::agregar_aplicacion(&raiz.join("configuracion.toml"), &nombre) {
        Ok(true) => {
            entrada.set_text("");
            mensaje_guardado(
                mensaje,
                raiz,
                aviso,
                boton_aplicar,
                ocupado,
                &format!("✓ Agregué «{nombre}». NixOS todavía no cambió."),
            );

            recargar_aplicaciones(
                lista,
                contador,
                raiz,
                mensaje,
                aviso,
                boton_aplicar,
                ocupado,
            );
        }
        Ok(false) => mensaje.set_text(&format!("«{nombre}» ya estaba elegida.")),
        Err(error) => mensaje.set_text(&error),
    }
}

fn recargar_controles(
    raiz: &Path,
    entrada_nombre: &gtk::Entry,
    selector_canal: &gtk::DropDown,
    selector_escritorio: &gtk::DropDown,
    escritorio_niri: &adw::SwitchRow,
    escritorio_hyprland: &adw::SwitchRow,
    escritorio_plasma: &adw::SwitchRow,
    escritorio_cinnamon: &adw::SwitchRow,
    teclado_espana: &adw::SwitchRow,
    teclado_latinoamerica: &adw::SwitchRow,
    entrada_resolucion: &gtk::Entry,
    entrada_hz: &gtk::SpinButton,
    filas_almacenamiento: &Rc<RefCell<Vec<(String, adw::SwitchRow)>>>,
    selector_estilo: &gtk::DropDown,
    selector_modo: &gtk::DropDown,
    bluetooth: &adw::SwitchRow,
    sunshine: &adw::SwitchRow,
    sunshine_autoinicio: &adw::SwitchRow,
    steam: &adw::SwitchRow,
    steam_remote_play: &adw::SwitchRow,
    steam_servidor: &adw::SwitchRow,
    impresion: &adw::SwitchRow,
    virtualizacion: &adw::SwitchRow,
    lista: &gtk::ListBox,
    contador: &gtk::Label,
    mensaje: &gtk::Label,
    aviso: &gtk::Revealer,
    boton_aplicar: &gtk::Button,
    ocupado: &Rc<Cell<bool>>,
    actualizando: &Rc<Cell<bool>>,
) {
    let configuracion = match configuracion::leer(&raiz.join("configuracion.toml")) {
        Ok(configuracion) => configuracion,
        Err(error) => {
            mensaje.set_text(&error);
            return;
        }
    };

    actualizando.set(true);
    entrada_nombre.set_text(&configuracion.nombre);
    selector_canal.set_selected(indice_canal(&configuracion.canal));
    selector_escritorio.set_selected(indice_escritorio(&configuracion.escritorio.principal));

    let escritorios = configuracion.escritorio.instalados_efectivos();
    escritorio_niri.set_active(escritorios.contains(&"niri"));
    escritorio_hyprland.set_active(escritorios.contains(&"hyprland"));
    escritorio_plasma.set_active(escritorios.contains(&"plasma"));
    escritorio_cinnamon.set_active(escritorios.contains(&"cinnamon"));

    teclado_espana.set_active(
        configuracion
            .teclado
            .distribuciones
            .iter()
            .any(|distribucion| distribucion == "españa"),
    );
    teclado_latinoamerica.set_active(
        configuracion
            .teclado
            .distribuciones
            .iter()
            .any(|distribucion| distribucion == "latinoamérica"),
    );

    entrada_resolucion.set_text(&configuracion.monitor.resolucion);
    entrada_hz.set_value(f64::from(configuracion.monitor.hz));

    for (nombre, fila) in filas_almacenamiento.borrow().iter() {
        fila.set_active(
            configuracion
                .almacenamiento
                .disponibles
                .iter()
                .any(|unidad| unidad == nombre),
        );
    }

    selector_estilo.set_selected(indice_estilo(&configuracion.apariencia.estilo));
    selector_modo.set_selected(indice_modo(&configuracion.apariencia.modo));
    bluetooth.set_active(configuracion.bluetooth.activo);
    sunshine.set_active(configuracion.sunshine.activo);
    sunshine_autoinicio.set_active(configuracion.sunshine.autoinicio);
    steam.set_active(configuracion.steam.activo);
    steam_remote_play.set_active(configuracion.steam.remote_play);
    steam_servidor.set_active(configuracion.steam.servidor_dedicado);
    impresion.set_active(configuracion.impresion.activa);
    virtualizacion.set_active(configuracion.virtualizacion.activa);
    actualizando.set(false);

    recargar_aplicaciones(
        lista,
        contador,
        raiz,
        mensaje,
        aviso,
        boton_aplicar,
        ocupado,
    );

    actualizar_estado_preview(raiz, aviso, boton_aplicar, ocupado);
}

fn ejecutar_motor(
    raiz: PathBuf,
    motor: PathBuf,
    argumentos: Vec<String>,
    autorizacion: bool,
    envio: mpsc::Sender<Mensaje>,
) {
    thread::spawn(move || {
        let mut comando = Command::new(&motor);
        comando
            .args(&argumentos)
            .env("KORUNIX_ROOT", &raiz)
            .current_dir(&raiz)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());

        if autorizacion {
            if let Some(pkexec) = autorizador_grafico() {
                comando.env("KORUNIX_SUDO_BIN", pkexec);
            }
        }

        let mut hijo = match comando.spawn() {
            Ok(hijo) => hijo,
            Err(error) => {
                let _ = envio.send(Mensaje::Linea(format!(
                    "No pude iniciar Korunix.\nDetalle: {error}"
                )));
                let _ = envio.send(Mensaje::Terminado(false));
                return;
            }
        };

        let salida = hijo.stdout.take();
        let errores = hijo.stderr.take();

        let envio_salida = envio.clone();
        let lector_salida = salida.map(|salida| {
            thread::spawn(move || {
                for linea in BufReader::new(salida).lines().map_while(Result::ok) {
                    let _ = envio_salida.send(Mensaje::Linea(linea));
                }
            })
        });

        let envio_errores = envio.clone();
        let lector_errores = errores.map(|errores| {
            thread::spawn(move || {
                for linea in BufReader::new(errores).lines().map_while(Result::ok) {
                    let _ = envio_errores.send(Mensaje::Linea(linea));
                }
            })
        });

        let correcto = hijo.wait().map(|estado| estado.success()).unwrap_or(false);

        if let Some(lector) = lector_salida {
            let _ = lector.join();
        }

        if let Some(lector) = lector_errores {
            let _ = lector.join();
        }

        let _ = envio.send(Mensaje::Terminado(correcto));
    });
}

fn iniciar_operacion(
    nombre: &'static str,
    argumentos: &[&str],
    autorizacion: bool,
    raiz: &Path,
    motor: &Path,
    vista: &gtk::TextView,
    salida_visible: &gtk::Revealer,
    estado: &gtk::Label,
    controles: &[gtk::Widget],
    ocupado: &Rc<Cell<bool>>,
    al_terminar: Option<AlTerminar>,
) {
    if ocupado.get() {
        return;
    }

    ocupado.set(true);
    sensibilidad(controles, false);

    vista.buffer().set_text("");
    salida_visible.set_reveal_child(true);
    estado.set_text(nombre);
    anexar_linea(vista, &format!("→ {nombre}"));

    let (envio, recepcion) = mpsc::channel();
    ejecutar_motor(
        raiz.to_path_buf(),
        motor.to_path_buf(),
        argumentos.iter().map(|valor| valor.to_string()).collect(),
        autorizacion,
        envio,
    );

    let vista = vista.clone();
    let estado = estado.clone();
    let controles = controles.to_vec();
    let ocupado = Rc::clone(ocupado);

    glib::timeout_add_local(Duration::from_millis(80), move || {
        loop {
            match recepcion.try_recv() {
                Ok(Mensaje::Linea(linea)) => anexar_linea(&vista, &linea),
                Ok(Mensaje::Terminado(correcto)) => {
                    if correcto {
                        estado.set_text("Listo");
                        anexar_linea(&vista, "✓ Operación terminada.");
                    } else {
                        estado.set_text("Hay que revisar el resultado");
                        anexar_linea(&vista, "✗ Korunix devolvió un error.");
                    }

                    sensibilidad(&controles, true);
                    ocupado.set(false);

                    if let Some(ref terminar) = al_terminar {
                        terminar(correcto);
                    }

                    return glib::ControlFlow::Break;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    estado.set_text("La operación terminó sin respuesta");
                    sensibilidad(&controles, true);
                    ocupado.set(false);

                    if let Some(ref terminar) = al_terminar {
                        terminar(false);
                    }

                    return glib::ControlFlow::Break;
                }
            }
        }

        glib::ControlFlow::Continue
    });
}

fn crear_pagina_area(titulo: &str, descripcion: &str) -> (gtk::ScrolledWindow, gtk::Box) {
    let contenido = gtk::Box::new(gtk::Orientation::Vertical, 18);
    contenido.set_margin_top(18);
    contenido.set_margin_bottom(24);
    contenido.set_margin_start(18);
    contenido.set_margin_end(18);

    let titulo = gtk::Label::new(Some(titulo));
    titulo.add_css_class("title-1");
    titulo.set_halign(gtk::Align::Start);

    let descripcion = gtk::Label::new(Some(descripcion));
    descripcion.set_wrap(true);
    descripcion.set_halign(gtk::Align::Start);
    descripcion.add_css_class("dim-label");

    contenido.append(&titulo);
    contenido.append(&descripcion);

    let clamp = adw::Clamp::builder()
        .maximum_size(680)
        .child(&contenido)
        .build();

    let desplazamiento = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&clamp)
        .build();

    (desplazamiento, contenido)
}

fn crear_acceso_area(titulo: &str, descripcion: &str) -> gtk::Button {
    let textos = gtk::Box::new(gtk::Orientation::Vertical, 2);
    textos.set_hexpand(true);

    let titulo = gtk::Label::new(Some(titulo));
    titulo.set_halign(gtk::Align::Start);
    titulo.add_css_class("heading");

    let descripcion = gtk::Label::new(Some(descripcion));
    descripcion.set_halign(gtk::Align::Start);
    descripcion.set_wrap(true);
    descripcion.add_css_class("dim-label");

    textos.append(&titulo);
    textos.append(&descripcion);

    let flecha = gtk::Image::from_icon_name("go-next-symbolic");

    let caja = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    caja.set_margin_top(8);
    caja.set_margin_bottom(8);
    caja.set_margin_start(10);
    caja.set_margin_end(10);
    caja.append(&textos);
    caja.append(&flecha);

    let boton = gtk::Button::new();
    boton.set_child(Some(&caja));
    boton.set_hexpand(true);
    boton.add_css_class("card");
    boton
}

fn conectar_acceso_area(
    boton: &gtk::Button,
    paginas: &gtk::Stack,
    titulo_barra: &adw::WindowTitle,
    boton_inicio: &gtk::Button,
    nombre: &'static str,
    titulo: &'static str,
) {
    let paginas = paginas.clone();
    let titulo_barra = titulo_barra.clone();
    let boton_inicio = boton_inicio.clone();

    boton.connect_clicked(move |_| {
        paginas.set_visible_child_name(nombre);
        titulo_barra.set_subtitle(titulo);
        boton_inicio.set_visible(true);
    });
}

fn construir_ventana(aplicacion: &Application) {
    let raiz = raiz_korunix();
    let motor = motor_korunix();

    let ventana = ApplicationWindow::builder()
        .application(aplicacion)
        .title("Korunix")
        .default_width(520)
        .default_height(760)
        .build();

    let barra = HeaderBar::new();

    let titulo_barra = adw::WindowTitle::new("Korunix", "Inicio");
    barra.set_title_widget(Some(&titulo_barra));

    let boton_inicio = gtk::Button::builder()
        .icon_name("go-previous-symbolic")
        .tooltip_text("Volver a Inicio")
        .build();
    boton_inicio.add_css_class("flat");
    boton_inicio.set_visible(false);
    barra.pack_start(&boton_inicio);

    let paginas = gtk::Stack::new();
    paginas.set_transition_type(gtk::StackTransitionType::SlideLeftRight);
    paginas.set_vhomogeneous(false);
    paginas.set_hhomogeneous(false);
    paginas.set_vexpand(true);

    let (pagina_inicio, contenido_inicio) = crear_pagina_area(
        "Inicio",
        "Entra directamente al área relacionada con lo que quieres cambiar. Las funciones que ya funcionan conservan el mismo motor de Korunix.",
    );
    let (pagina_aplicaciones, contenido_aplicaciones) = crear_pagina_area(
        "Aplicaciones",
        "Instala, quita y ajusta aplicaciones sin convertir el catálogo visual en una lista cerrada.",
    );
    let (pagina_apariencia, contenido_apariencia) = crear_pagina_area(
        "Apariencia",
        "El estilo y el modo siguen siendo decisiones separadas.",
    );
    let (pagina_sistema, contenido_sistema) = crear_pagina_area(
        "Sistema",
        "Nombre del equipo, canal, escritorios disponibles y funciones generales.",
    );
    let (pagina_hardware, contenido_hardware) = crear_pagina_area(
        "Hardware",
        "Controles que describen o ajustan el equipo físico sin exponer identificadores técnicos.",
    );
    let (pagina_almacenamiento, contenido_almacenamiento) = crear_pagina_area(
        "Almacenamiento",
        "Unidades, transferencias y expulsión segura en un solo lugar.",
    );
    let (pagina_personas, contenido_personas) = crear_pagina_area(
        "Personas",
        "Teclados y preferencias personales sin pedir identificadores técnicos.",
    );
    let (pagina_actualizaciones, contenido_actualizaciones) = crear_pagina_area(
        "Actualizaciones",
        "Estado local primero; buscar, revisar y aplicar siguen usando el motor ya probado.",
    );
    let (pagina_copias, contenido_copias) = crear_pagina_area(
        "Copias y recuperación",
        "Copias, historial y restauración sin mezclar estos resultados con otras tareas.",
    );

    paginas.add_named(&pagina_inicio, Some("inicio"));
    paginas.add_named(&pagina_aplicaciones, Some("aplicaciones"));
    paginas.add_named(&pagina_apariencia, Some("apariencia"));
    paginas.add_named(&pagina_sistema, Some("sistema"));
    paginas.add_named(&pagina_hardware, Some("hardware"));
    paginas.add_named(&pagina_almacenamiento, Some("almacenamiento"));
    paginas.add_named(&pagina_personas, Some("personas"));
    paginas.add_named(&pagina_actualizaciones, Some("actualizaciones"));
    paginas.add_named(&pagina_copias, Some("copias"));
    paginas.set_visible_child_name("inicio");

    let general = adw::PreferencesGroup::builder().title("General").build();
    let acceso_aplicaciones = crear_acceso_area(
        "Aplicaciones",
        "Catálogo, aplicaciones elegidas y opciones especiales.",
    );
    let acceso_apariencia = crear_acceso_area(
        "Apariencia",
        "Estilo visual y modo claro, oscuro o automático.",
    );
    general.add(&acceso_aplicaciones);
    general.add(&acceso_apariencia);
    contenido_inicio.append(&general);

    let equipo = adw::PreferencesGroup::builder().title("Equipo").build();
    let acceso_sistema = crear_acceso_area(
        "Sistema",
        "Canal, escritorios, Bluetooth, impresión y virtualización.",
    );
    let acceso_hardware = crear_acceso_area(
        "Hardware",
        "Pantalla y, más adelante, el resto de dispositivos detectados.",
    );
    let acceso_almacenamiento = crear_acceso_area(
        "Almacenamiento",
        "Discos, archivos grandes y expulsión segura.",
    );
    let acceso_personas = crear_acceso_area(
        "Personas",
        "Teclados y preferencias de las personas del equipo.",
    );
    equipo.add(&acceso_sistema);
    equipo.add(&acceso_hardware);
    equipo.add(&acceso_almacenamiento);
    equipo.add(&acceso_personas);
    contenido_inicio.append(&equipo);

    let mantenimiento = adw::PreferencesGroup::builder()
        .title("Mantenimiento")
        .build();
    let acceso_actualizaciones = crear_acceso_area(
        "Actualizaciones",
        "Buscar, revisar y aplicar sin cambiar el motor cerrado.",
    );
    let acceso_copias = crear_acceso_area(
        "Copias y recuperación",
        "Crear copias, revisar restauraciones e historial.",
    );
    mantenimiento.add(&acceso_actualizaciones);
    mantenimiento.add(&acceso_copias);
    contenido_inicio.append(&mantenimiento);

    for (boton, nombre, titulo) in [
        (&acceso_aplicaciones, "aplicaciones", "Aplicaciones"),
        (&acceso_apariencia, "apariencia", "Apariencia"),
        (&acceso_sistema, "sistema", "Sistema"),
        (&acceso_hardware, "hardware", "Hardware"),
        (&acceso_almacenamiento, "almacenamiento", "Almacenamiento"),
        (&acceso_personas, "personas", "Personas"),
        (
            &acceso_actualizaciones,
            "actualizaciones",
            "Actualizaciones",
        ),
        (&acceso_copias, "copias", "Copias y recuperación"),
    ] {
        conectar_acceso_area(
            boton,
            &paginas,
            &titulo_barra,
            &boton_inicio,
            nombre,
            titulo,
        );
    }

    {
        let paginas = paginas.clone();
        let titulo_barra = titulo_barra.clone();
        let boton_inicio_senal = boton_inicio.clone();

        boton_inicio.connect_clicked(move |_| {
            paginas.set_visible_child_name("inicio");
            titulo_barra.set_subtitle("Inicio");
            boton_inicio_senal.set_visible(false);
        });
    }

    let aviso_texto = gtk::Label::new(Some(
        "La configuración y el preview no coinciden. Crea un preview antes de aplicar.",
    ));
    aviso_texto.set_wrap(true);
    aviso_texto.set_halign(gtk::Align::Start);
    aviso_texto.set_margin_top(10);
    aviso_texto.set_margin_bottom(10);
    aviso_texto.set_margin_start(12);
    aviso_texto.set_margin_end(12);

    let aviso_caja = gtk::Box::new(gtk::Orientation::Vertical, 0);
    aviso_caja.add_css_class("card");
    aviso_caja.append(&aviso_texto);

    let aviso = gtk::Revealer::new();
    aviso.set_child(Some(&aviso_caja));

    let configuracion_grupo = adw::PreferencesGroup::builder()
        .title("Configuración")
        .description(
            "Guardar una opción modifica configuracion.toml. No activa ni reconstruye NixOS.",
        )
        .build();

    let nombre_titulo = gtk::Label::new(Some("Nombre del equipo"));
    nombre_titulo.set_halign(gtk::Align::Start);
    nombre_titulo.add_css_class("heading");

    let nombre_caja = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let entrada_nombre = gtk::Entry::new();
    entrada_nombre.set_hexpand(true);
    entrada_nombre.set_placeholder_text(Some("por ejemplo, korunix"));
    let boton_nombre = gtk::Button::with_label("Guardar");
    nombre_caja.append(&entrada_nombre);
    nombre_caja.append(&boton_nombre);

    let canal_titulo = gtk::Label::new(Some("Canal"));
    canal_titulo.set_halign(gtk::Align::Start);
    canal_titulo.add_css_class("heading");

    let selector_canal = gtk::DropDown::from_strings(&["Estable", "Inestable"]);

    let escritorio_titulo = gtk::Label::new(Some("Escritorio principal"));
    escritorio_titulo.set_halign(gtk::Align::Start);
    escritorio_titulo.add_css_class("heading");

    let selector_escritorio =
        gtk::DropDown::from_strings(&["Niri", "Hyprland", "Cinnamon", "Plasma"]);

    let edicion = gtk::Box::new(gtk::Orientation::Vertical, 8);
    edicion.append(&nombre_titulo);
    edicion.append(&nombre_caja);
    edicion.append(&canal_titulo);
    edicion.append(&selector_canal);
    edicion.append(&escritorio_titulo);
    edicion.append(&selector_escritorio);

    configuracion_grupo.add(&edicion);
    contenido_sistema.append(&configuracion_grupo);

    let escritorios_grupo = adw::PreferencesGroup::builder()
        .title("Escritorios")
        .description("El principal y los disponibles siguen siendo decisiones distintas.")
        .build();

    let escritorio_niri = adw::SwitchRow::builder()
        .title("Niri disponible")
        .subtitle("No puedes apagarlo mientras sea el escritorio principal.")
        .build();

    let escritorio_hyprland = adw::SwitchRow::builder()
        .title("Hyprland disponible")
        .build();

    let escritorio_plasma = adw::SwitchRow::builder().title("Plasma disponible").build();

    let escritorio_cinnamon = adw::SwitchRow::builder()
        .title("Cinnamon disponible")
        .build();

    escritorios_grupo.add(&escritorio_niri);
    escritorios_grupo.add(&escritorio_hyprland);
    escritorios_grupo.add(&escritorio_plasma);
    escritorios_grupo.add(&escritorio_cinnamon);
    contenido_sistema.append(&escritorios_grupo);

    let teclados_grupo = adw::PreferencesGroup::builder()
        .title("Teclados")
        .description("Las distribuciones humanas quedan separadas de los identificadores XKB.")
        .build();

    let teclado_espana = adw::SwitchRow::builder()
        .title("Teclado de España")
        .subtitle("Incluye la variante con composición usada por Korunix.")
        .build();

    let teclado_latinoamerica = adw::SwitchRow::builder()
        .title("Teclado latinoamericano")
        .build();

    let cambio_teclado = adw::ActionRow::builder()
        .title("Cambiar entre teclados")
        .subtitle("Alt + Shift")
        .build();

    teclados_grupo.add(&teclado_espana);
    teclados_grupo.add(&teclado_latinoamerica);
    teclados_grupo.add(&cambio_teclado);
    contenido_personas.append(&teclados_grupo);

    let hardware_grupo = adw::PreferencesGroup::builder()
        .title("Pantalla")
        .description("La salida física sigue siendo un hecho detectado; aquí solo aparecen decisiones humanas.")
        .build();

    let monitor_titulo = gtk::Label::new(Some("Resolución y frecuencia"));
    monitor_titulo.set_halign(gtk::Align::Start);
    monitor_titulo.add_css_class("heading");

    let monitor_caja = gtk::Box::new(gtk::Orientation::Horizontal, 8);

    let entrada_resolucion = gtk::Entry::new();
    entrada_resolucion.set_hexpand(true);
    entrada_resolucion.set_placeholder_text(Some("1920x1080"));

    let entrada_hz = gtk::SpinButton::with_range(1.0, 1000.0, 1.0);
    entrada_hz.set_numeric(true);
    entrada_hz.set_width_chars(5);

    let boton_monitor = gtk::Button::with_label("Guardar");
    monitor_caja.append(&entrada_resolucion);
    monitor_caja.append(&entrada_hz);
    monitor_caja.append(&boton_monitor);

    let monitor_bloque = gtk::Box::new(gtk::Orientation::Vertical, 8);
    monitor_bloque.append(&monitor_titulo);
    monitor_bloque.append(&monitor_caja);

    hardware_grupo.add(&monitor_bloque);
    contenido_hardware.append(&hardware_grupo);

    let almacenamiento_grupo = adw::PreferencesGroup::builder()
        .title("Almacenamiento")
        .description(
            "Korunix muestra los discos con un nombre reconocible. UUID, /dev y rutas de montaje quedan por dentro.",
        )
        .build();

    let almacenamiento_lista = gtk::ListBox::new();
    almacenamiento_lista.set_selection_mode(gtk::SelectionMode::None);
    almacenamiento_lista.add_css_class("boxed-list");

    let almacenamiento_estado =
        gtk::Label::new(Some("La ventana abrirá antes de comprobar los discos."));
    almacenamiento_estado.set_wrap(true);
    almacenamiento_estado.set_halign(gtk::Align::Start);
    almacenamiento_estado.add_css_class("dim-label");

    let almacenamiento_caja = gtk::Box::new(gtk::Orientation::Vertical, 8);
    almacenamiento_caja.append(&almacenamiento_lista);
    almacenamiento_caja.append(&almacenamiento_estado);

    let transferencia_titulo = gtk::Label::new(Some("Transferir un archivo"));
    transferencia_titulo.set_halign(gtk::Align::Start);
    transferencia_titulo.add_css_class("heading");

    let archivo_transferencia = Rc::new(RefCell::new(None::<PathBuf>));
    let archivo_transferencia_texto = gtk::Label::new(Some("Ningún archivo elegido"));
    archivo_transferencia_texto.set_halign(gtk::Align::Start);
    archivo_transferencia_texto.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
    archivo_transferencia_texto.add_css_class("dim-label");

    let boton_elegir_archivo = gtk::Button::with_label("Elegir archivo");
    let selector_destino = gtk::DropDown::from_strings(&["Sin unidades disponibles"]);
    let destinos_transferencia = Rc::new(RefCell::new(Vec::<String>::new()));
    let boton_transferir = gtk::Button::with_label("Copiar archivo");
    boton_transferir.add_css_class("suggested-action");

    let transferencia_progreso = gtk::ProgressBar::new();
    transferencia_progreso.set_show_text(true);
    transferencia_progreso.set_text(Some("Sin transferencia en curso"));

    let transferencia_estado = gtk::Label::new(Some(
        "El nombre final aparecerá solo cuando el archivo esté completo y verificado.",
    ));
    transferencia_estado.set_wrap(true);
    transferencia_estado.set_halign(gtk::Align::Start);
    transferencia_estado.add_css_class("dim-label");

    let transferencia_botones = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    transferencia_botones.append(&boton_elegir_archivo);
    transferencia_botones.append(&selector_destino);
    selector_destino.set_hexpand(true);
    transferencia_botones.append(&boton_transferir);

    let transferencia_caja = gtk::Box::new(gtk::Orientation::Vertical, 8);
    transferencia_caja.set_margin_top(10);
    transferencia_caja.append(&transferencia_titulo);
    transferencia_caja.append(&archivo_transferencia_texto);
    transferencia_caja.append(&transferencia_botones);
    transferencia_caja.append(&transferencia_progreso);
    transferencia_caja.append(&transferencia_estado);

    almacenamiento_caja.append(&transferencia_caja);
    almacenamiento_grupo.add(&almacenamiento_caja);
    contenido_almacenamiento.append(&almacenamiento_grupo);

    let filas_almacenamiento: Rc<RefCell<Vec<(String, adw::SwitchRow)>>> =
        Rc::new(RefCell::new(Vec::new()));

    let copias_grupo = adw::PreferencesGroup::builder()
        .title("Copias e historial")
        .description(
            "Una copia de Korunix guarda tus decisiones, flake.lock y los avatares usados. No guarda hardware, contraseñas ni claves privadas.",
        )
        .build();

    let copia_seleccionada = Rc::new(RefCell::new(None::<PathBuf>));
    let copia_revisada = Rc::new(RefCell::new(None::<PathBuf>));

    let copia_estado = gtk::Label::new(Some(
        "Puedes crear una copia portable o elegir una existente para revisarla antes de restaurar.",
    ));
    copia_estado.set_wrap(true);
    copia_estado.set_halign(gtk::Align::Start);
    copia_estado.add_css_class("dim-label");

    let copia_archivo = gtk::Label::new(Some("Ninguna copia elegida"));
    copia_archivo.set_halign(gtk::Align::Start);
    copia_archivo.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
    copia_archivo.add_css_class("heading");

    let boton_crear_copia = gtk::Button::with_label("Crear copia de Korunix");
    let boton_elegir_copia = gtk::Button::with_label("Elegir copia");
    let boton_revisar_copia = gtk::Button::with_label("Revisar restauración");
    let boton_restaurar_copia = gtk::Button::with_label("Restaurar esta copia");
    boton_restaurar_copia.add_css_class("suggested-action");
    boton_restaurar_copia.set_sensitive(false);
    let boton_historial = gtk::Button::with_label("Ver historial");

    let copias_botones_primarios = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    copias_botones_primarios.append(&boton_crear_copia);
    copias_botones_primarios.append(&boton_elegir_copia);

    let copias_botones_restaurar = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    copias_botones_restaurar.append(&boton_revisar_copia);
    copias_botones_restaurar.append(&boton_restaurar_copia);
    copias_botones_restaurar.append(&boton_historial);

    let copias_caja = gtk::Box::new(gtk::Orientation::Vertical, 8);
    copias_caja.append(&copia_estado);
    copias_caja.append(&copia_archivo);
    copias_caja.append(&copias_botones_primarios);
    copias_caja.append(&copias_botones_restaurar);

    copias_grupo.add(&copias_caja);
    contenido_copias.append(&copias_grupo);

    let apariencia_grupo = adw::PreferencesGroup::builder()
        .title("Apariencia")
        .description(
            "El estilo y el modo son decisiones separadas. Noctalia deriva el resto cuando corresponde.",
        )
        .build();

    let estilo_titulo = gtk::Label::new(Some("Estilo"));
    estilo_titulo.set_halign(gtk::Align::Start);
    estilo_titulo.add_css_class("heading");
    let selector_estilo =
        gtk::DropDown::from_strings(&["Predeterminado", "Dinámico", "Everforest"]);

    let modo_titulo = gtk::Label::new(Some("Modo"));
    modo_titulo.set_halign(gtk::Align::Start);
    modo_titulo.add_css_class("heading");
    let selector_modo = gtk::DropDown::from_strings(&["Claro", "Oscuro", "Automático"]);

    let apariencia_caja = gtk::Box::new(gtk::Orientation::Vertical, 8);
    apariencia_caja.append(&estilo_titulo);
    apariencia_caja.append(&selector_estilo);
    apariencia_caja.append(&modo_titulo);
    apariencia_caja.append(&selector_modo);
    apariencia_grupo.add(&apariencia_caja);
    contenido_apariencia.append(&apariencia_grupo);

    let sistema_funciones_grupo = adw::PreferencesGroup::builder()
        .title("Funciones del sistema")
        .description("Estas capacidades afectan al equipo completo.")
        .build();

    let aplicaciones_opciones_grupo = adw::PreferencesGroup::builder()
        .title("Opciones especiales")
        .description(
            "Apagar una función principal conserva sus preferencias internas para cuando vuelva a activarse.",
        )
        .build();

    let bluetooth = adw::SwitchRow::builder()
        .title("Bluetooth")
        .subtitle("También prepara el soporte de mandos compatible que Korunix deriva.")
        .build();

    let sunshine = adw::SwitchRow::builder()
        .title("Sunshine")
        .subtitle("Acceso y transmisión remota.")
        .build();

    let sunshine_autoinicio = adw::SwitchRow::builder()
        .title("Iniciar Sunshine automáticamente")
        .subtitle("Esta preferencia se conserva aunque Sunshine esté apagado.")
        .build();

    let steam = adw::SwitchRow::builder()
        .title("Steam")
        .subtitle("Korunix deriva GameMode, Millennium y la integración visual.")
        .build();

    let steam_remote_play = adw::SwitchRow::builder()
        .title("Steam Remote Play")
        .subtitle("Solo abre sus reglas cuando Steam también está activo.")
        .build();

    let steam_servidor = adw::SwitchRow::builder()
        .title("Servidor dedicado de Steam")
        .subtitle("La preferencia se conserva aunque Steam esté apagado.")
        .build();

    let impresion = adw::SwitchRow::builder()
        .title("Impresión")
        .subtitle("El controlador conocido del equipo sigue siendo un detalle técnico.")
        .build();

    let virtualizacion = adw::SwitchRow::builder()
        .title("Virtualización")
        .subtitle("Activa la capacidad de ejecutar máquinas virtuales.")
        .build();

    sistema_funciones_grupo.add(&bluetooth);
    sistema_funciones_grupo.add(&impresion);
    sistema_funciones_grupo.add(&virtualizacion);
    contenido_sistema.append(&sistema_funciones_grupo);

    aplicaciones_opciones_grupo.add(&sunshine);
    aplicaciones_opciones_grupo.add(&sunshine_autoinicio);
    aplicaciones_opciones_grupo.add(&steam);
    aplicaciones_opciones_grupo.add(&steam_remote_play);
    aplicaciones_opciones_grupo.add(&steam_servidor);
    contenido_aplicaciones.append(&aplicaciones_opciones_grupo);

    let aplicaciones_grupo = adw::PreferencesGroup::builder()
        .title("Aplicaciones")
        .description(
            "Puedes escribir cualquier nombre. El catálogo visual no limita lo que Korunix puede intentar resolver.",
        )
        .build();

    let agregar_caja = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let entrada_aplicacion = gtk::Entry::new();
    entrada_aplicacion.set_hexpand(true);
    entrada_aplicacion.set_placeholder_text(Some("por ejemplo, karere"));
    let boton_agregar = gtk::Button::with_label("Agregar");
    agregar_caja.append(&entrada_aplicacion);
    agregar_caja.append(&boton_agregar);

    let contador_aplicaciones = gtk::Label::new(Some("—"));
    contador_aplicaciones.set_halign(gtk::Align::Start);
    contador_aplicaciones.add_css_class("dim-label");

    let lista_aplicaciones = gtk::ListBox::new();
    lista_aplicaciones.set_selection_mode(gtk::SelectionMode::None);
    lista_aplicaciones.add_css_class("boxed-list");

    let aplicaciones_expandir = gtk::Expander::new(Some("Mostrar aplicaciones elegidas"));
    aplicaciones_expandir.set_child(Some(&lista_aplicaciones));

    let aplicaciones_caja = gtk::Box::new(gtk::Orientation::Vertical, 8);
    aplicaciones_caja.append(&agregar_caja);
    aplicaciones_caja.append(&contador_aplicaciones);
    aplicaciones_caja.append(&aplicaciones_expandir);

    aplicaciones_grupo.add(&aplicaciones_caja);
    contenido_aplicaciones.append(&aplicaciones_grupo);

    let mensaje_configuracion = gtk::Label::new(Some(
        "Los cambios de esta sección se guardan enseguida, pero NixOS permanece igual.",
    ));
    mensaje_configuracion.set_wrap(true);
    mensaje_configuracion.set_halign(gtk::Align::Start);
    mensaje_configuracion.add_css_class("dim-label");

    let actualizaciones_grupo = adw::PreferencesGroup::builder()
        .title("Actualizaciones")
        .description(
            "Primero ves lo que Korunix ya sabe. Buscar usa Internet y no cambia NixOS. Revisar construye una generación completa. Aplicar usa exactamente esa generación.",
        )
        .build();

    let actualizaciones_estado =
        gtk::Label::new(Some("Leyendo el estado local después de abrir la ventana…"));
    actualizaciones_estado.set_wrap(true);
    actualizaciones_estado.set_halign(gtk::Align::Start);
    actualizaciones_estado.add_css_class("heading");

    let boton_buscar_actualizaciones = gtk::Button::with_label("Buscar actualizaciones");
    let boton_revisar_actualizacion = gtk::Button::with_label("Revisar actualización");
    let boton_aplicar_actualizacion = gtk::Button::with_label("Aplicar actualización");
    boton_aplicar_actualizacion.add_css_class("suggested-action");
    boton_aplicar_actualizacion.set_sensitive(false);

    let actualizaciones_botones = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    actualizaciones_botones.append(&boton_buscar_actualizaciones);
    actualizaciones_botones.append(&boton_revisar_actualizacion);
    actualizaciones_botones.append(&boton_aplicar_actualizacion);

    let actualizaciones_salida = gtk::TextView::new();
    actualizaciones_salida.set_editable(false);
    actualizaciones_salida.set_cursor_visible(false);
    actualizaciones_salida.set_monospace(true);
    actualizaciones_salida.set_wrap_mode(gtk::WrapMode::WordChar);

    let actualizaciones_desplazamiento = gtk::ScrolledWindow::builder()
        .min_content_height(180)
        .child(&actualizaciones_salida)
        .build();
    actualizaciones_desplazamiento.add_css_class("card");

    let actualizaciones_salida_visible = gtk::Revealer::new();
    actualizaciones_salida_visible.set_child(Some(&actualizaciones_desplazamiento));

    let actualizaciones_caja = gtk::Box::new(gtk::Orientation::Vertical, 8);
    actualizaciones_caja.append(&actualizaciones_estado);
    actualizaciones_caja.append(&actualizaciones_botones);
    actualizaciones_caja.append(&actualizaciones_salida_visible);

    actualizaciones_grupo.add(&actualizaciones_caja);
    contenido_actualizaciones.append(&actualizaciones_grupo);

    let acciones = adw::PreferencesGroup::builder()
        .title("Cambios del sistema")
        .description(
            "Preview construye una generación completa sin activarla. Aplicar usa exactamente el preview revisado. Volver recupera la generación protegida.",
        )
        .build();

    let caja_acciones = gtk::Box::new(gtk::Orientation::Vertical, 8);

    let boton_preview = gtk::Button::with_label("Crear preview");
    boton_preview.add_css_class("suggested-action");

    let boton_aplicar = gtk::Button::with_label("Aplicar cambios");
    let boton_volver = gtk::Button::with_label("Volver a la generación anterior");

    caja_acciones.append(&boton_preview);
    caja_acciones.append(&boton_aplicar);
    caja_acciones.append(&boton_volver);
    acciones.add(&caja_acciones);
    contenido_inicio.append(&acciones);

    let estado = gtk::Label::new(Some("Listo"));
    estado.set_halign(gtk::Align::Start);
    estado.add_css_class("heading");
    contenido_inicio.append(&estado);

    let salida = gtk::TextView::new();
    salida.set_editable(false);
    salida.set_cursor_visible(false);
    salida.set_monospace(true);
    salida.set_wrap_mode(gtk::WrapMode::WordChar);

    let desplazamiento_salida = gtk::ScrolledWindow::builder()
        .min_content_height(190)
        .child(&salida)
        .build();
    desplazamiento_salida.add_css_class("card");

    let salida_visible = gtk::Revealer::new();
    salida_visible.set_child(Some(&desplazamiento_salida));
    contenido_inicio.append(&salida_visible);

    aviso.set_margin_top(10);
    aviso.set_margin_start(18);
    aviso.set_margin_end(18);

    mensaje_configuracion.set_margin_top(8);
    mensaje_configuracion.set_margin_bottom(10);
    mensaje_configuracion.set_margin_start(18);
    mensaje_configuracion.set_margin_end(18);

    let raiz_visual = gtk::Box::new(gtk::Orientation::Vertical, 0);
    raiz_visual.append(&aviso);
    raiz_visual.append(&paginas);
    raiz_visual.append(&mensaje_configuracion);

    let vista = ToolbarView::new();
    vista.add_top_bar(&barra);
    vista.set_content(Some(&raiz_visual));
    ventana.set_content(Some(&vista));

    let ocupado = Rc::new(Cell::new(false));
    let ocupado_actualizaciones = Rc::new(Cell::new(false));
    let actualizando = Rc::new(Cell::new(false));

    let controles: Vec<gtk::Widget> = vec![
        entrada_nombre.clone().upcast(),
        boton_nombre.clone().upcast(),
        selector_canal.clone().upcast(),
        selector_escritorio.clone().upcast(),
        escritorio_niri.clone().upcast(),
        escritorio_hyprland.clone().upcast(),
        escritorio_plasma.clone().upcast(),
        escritorio_cinnamon.clone().upcast(),
        teclado_espana.clone().upcast(),
        teclado_latinoamerica.clone().upcast(),
        entrada_resolucion.clone().upcast(),
        entrada_hz.clone().upcast(),
        boton_monitor.clone().upcast(),
        almacenamiento_lista.clone().upcast(),
        boton_elegir_archivo.clone().upcast(),
        selector_destino.clone().upcast(),
        boton_transferir.clone().upcast(),
        boton_crear_copia.clone().upcast(),
        boton_elegir_copia.clone().upcast(),
        boton_revisar_copia.clone().upcast(),
        boton_historial.clone().upcast(),
        boton_buscar_actualizaciones.clone().upcast(),
        boton_revisar_actualizacion.clone().upcast(),
        boton_aplicar_actualizacion.clone().upcast(),
        selector_estilo.clone().upcast(),
        selector_modo.clone().upcast(),
        bluetooth.clone().upcast(),
        sunshine.clone().upcast(),
        sunshine_autoinicio.clone().upcast(),
        steam.clone().upcast(),
        steam_remote_play.clone().upcast(),
        steam_servidor.clone().upcast(),
        impresion.clone().upcast(),
        virtualizacion.clone().upcast(),
        entrada_aplicacion.clone().upcast(),
        boton_agregar.clone().upcast(),
        lista_aplicaciones.clone().upcast(),
        boton_preview.clone().upcast(),
        boton_aplicar.clone().upcast(),
        boton_volver.clone().upcast(),
    ];

    {
        let ventana = ventana.clone();
        let archivo = Rc::clone(&archivo_transferencia);
        let texto_archivo = archivo_transferencia_texto.clone();

        boton_elegir_archivo.connect_clicked(move |_| {
            let dialogo = gtk::FileChooserNative::new(
                Some("Elegir archivo para transferir"),
                Some(&ventana),
                gtk::FileChooserAction::Open,
                Some("Elegir"),
                Some("Cancelar"),
            );

            let archivo = Rc::clone(&archivo);
            let texto_archivo = texto_archivo.clone();

            dialogo.connect_response(move |dialogo, respuesta| {
                if respuesta == gtk::ResponseType::Accept {
                    if let Some(ruta) = dialogo.file().and_then(|archivo| archivo.path()) {
                        texto_archivo.set_text(
                            &ruta
                                .file_name()
                                .map(|nombre| nombre.to_string_lossy().into_owned())
                                .unwrap_or_else(|| ruta.display().to_string()),
                        );
                        *archivo.borrow_mut() = Some(ruta);
                    }
                }
            });

            dialogo.show();
        });
    }

    {
        let archivo = Rc::clone(&archivo_transferencia);
        let selector = selector_destino.clone();
        let destinos = Rc::clone(&destinos_transferencia);
        let raiz = raiz.clone();
        let progreso = transferencia_progreso.clone();
        let estado_transferencia = transferencia_estado.clone();
        let controles = controles.clone();
        let ocupado = Rc::clone(&ocupado);
        let aviso = aviso.clone();
        let boton_aplicar = boton_aplicar.clone();

        boton_transferir.connect_clicked(move |_| {
            let Some(origen) = archivo.borrow().clone() else {
                estado_transferencia.set_text("Elige primero el archivo que quieres copiar.");
                return;
            };

            let indice = selector.selected() as usize;
            let Some(unidad) = destinos.borrow().get(indice).cloned() else {
                estado_transferencia
                    .set_text("No hay una unidad disponible elegida para recibir el archivo.");
                return;
            };

            iniciar_transferencia_gui(
                origen,
                unidad,
                &raiz,
                &progreso,
                &estado_transferencia,
                &controles,
                &ocupado,
                &aviso,
                &boton_aplicar,
            );
        });
    }

    {
        let entrada_nombre = entrada_nombre.clone();
        let raiz = raiz.clone();
        let mensaje = mensaje_configuracion.clone();
        let aviso = aviso.clone();
        let boton_aplicar = boton_aplicar.clone();
        let ocupado = Rc::clone(&ocupado);

        boton_nombre.connect_clicked(move |_| {
            guardar_nombre(
                &entrada_nombre,
                &raiz,
                &mensaje,
                &aviso,
                &boton_aplicar,
                &ocupado,
            );
        });
    }

    {
        let raiz = raiz.clone();
        let mensaje = mensaje_configuracion.clone();
        let aviso = aviso.clone();
        let boton_aplicar = boton_aplicar.clone();
        let ocupado = Rc::clone(&ocupado);

        entrada_nombre.connect_activate(move |entrada| {
            guardar_nombre(entrada, &raiz, &mensaje, &aviso, &boton_aplicar, &ocupado);
        });
    }

    {
        let raiz = raiz.clone();
        let mensaje = mensaje_configuracion.clone();
        let aviso = aviso.clone();
        let boton_aplicar = boton_aplicar.clone();
        let ocupado = Rc::clone(&ocupado);
        let actualizando = Rc::clone(&actualizando);

        selector_canal.connect_selected_notify(move |selector| {
            if actualizando.get() {
                return;
            }

            let Some(canal) = canal_por_indice(selector.selected()) else {
                return;
            };

            match configuracion::cambiar_canal(&raiz.join("configuracion.toml"), canal) {
                Ok(true) => mensaje_guardado(
                    &mensaje,
                    &raiz,
                    &aviso,
                    &boton_aplicar,
                    &ocupado,
                    &format!("✓ Canal cambiado a «{canal}». NixOS todavía no cambió."),
                ),
                Ok(false) => mensaje.set_text("Ese canal ya estaba elegido."),
                Err(error) => mensaje.set_text(&error),
            }
        });
    }

    {
        let raiz = raiz.clone();
        let mensaje = mensaje_configuracion.clone();
        let aviso = aviso.clone();
        let boton_aplicar = boton_aplicar.clone();
        let ocupado = Rc::clone(&ocupado);
        let actualizando = Rc::clone(&actualizando);

        selector_escritorio.connect_selected_notify(move |selector| {
            if actualizando.get() {
                return;
            }

            let Some(escritorio) = escritorio_por_indice(selector.selected()) else {
                return;
            };

            match configuracion::cambiar_escritorio(&raiz.join("configuracion.toml"), escritorio) {
                Ok(true) => mensaje_guardado(
                    &mensaje,
                    &raiz,
                    &aviso,
                    &boton_aplicar,
                    &ocupado,
                    &format!(
                        "✓ {} quedó como escritorio principal. NixOS todavía no cambió.",
                        nombre_escritorio(escritorio)
                    ),
                ),
                Ok(false) => mensaje.set_text("Ese escritorio ya era el principal."),
                Err(error) => {
                    mensaje.set_text(&error);

                    if let Ok(configuracion) = configuracion::leer(&raiz.join("configuracion.toml"))
                    {
                        actualizando.set(true);
                        selector
                            .set_selected(indice_escritorio(&configuracion.escritorio.principal));
                        actualizando.set(false);
                    }
                }
            }
        });
    }

    for (fila, nombre) in [
        (escritorio_niri.clone(), "niri"),
        (escritorio_hyprland.clone(), "hyprland"),
        (escritorio_plasma.clone(), "plasma"),
        (escritorio_cinnamon.clone(), "cinnamon"),
    ] {
        let niri = escritorio_niri.clone();
        let hyprland = escritorio_hyprland.clone();
        let plasma = escritorio_plasma.clone();
        let cinnamon = escritorio_cinnamon.clone();
        let raiz = raiz.clone();
        let mensaje = mensaje_configuracion.clone();
        let aviso = aviso.clone();
        let boton_aplicar = boton_aplicar.clone();
        let ocupado = Rc::clone(&ocupado);
        let actualizando = Rc::clone(&actualizando);

        fila.connect_active_notify(move |_| {
            if actualizando.get() {
                return;
            }

            let instalados = escritorios_elegidos(&niri, &hyprland, &plasma, &cinnamon);

            match configuracion::cambiar_escritorios(&raiz.join("configuracion.toml"), &instalados)
            {
                Ok(true) => mensaje_guardado(
                    &mensaje,
                    &raiz,
                    &aviso,
                    &boton_aplicar,
                    &ocupado,
                    &format!("✓ Cambié la disponibilidad de {nombre}. NixOS todavía no cambió."),
                ),
                Ok(false) => {}
                Err(error) => {
                    mensaje.set_text(&error);

                    if let Ok(configuracion) = configuracion::leer(&raiz.join("configuracion.toml"))
                    {
                        let instalados = configuracion.escritorio.instalados_efectivos();
                        actualizando.set(true);
                        niri.set_active(instalados.contains(&"niri"));
                        hyprland.set_active(instalados.contains(&"hyprland"));
                        plasma.set_active(instalados.contains(&"plasma"));
                        cinnamon.set_active(instalados.contains(&"cinnamon"));
                        actualizando.set(false);
                    }
                }
            }
        });
    }

    for (fila, nombre) in [
        (teclado_espana.clone(), "españa"),
        (teclado_latinoamerica.clone(), "latinoamérica"),
    ] {
        let espana = teclado_espana.clone();
        let latinoamerica = teclado_latinoamerica.clone();
        let raiz = raiz.clone();
        let mensaje = mensaje_configuracion.clone();
        let aviso = aviso.clone();
        let boton_aplicar = boton_aplicar.clone();
        let ocupado = Rc::clone(&ocupado);
        let actualizando = Rc::clone(&actualizando);

        fila.connect_active_notify(move |_| {
            if actualizando.get() {
                return;
            }

            let distribuciones = teclados_elegidos(&espana, &latinoamerica);

            match configuracion::cambiar_teclado(&raiz.join("configuracion.toml"), &distribuciones)
            {
                Ok(true) => mensaje_guardado(
                    &mensaje,
                    &raiz,
                    &aviso,
                    &boton_aplicar,
                    &ocupado,
                    &format!("✓ Cambié el teclado «{nombre}». NixOS todavía no cambió."),
                ),
                Ok(false) => {}
                Err(error) => {
                    mensaje.set_text(&error);

                    if let Ok(configuracion) = configuracion::leer(&raiz.join("configuracion.toml"))
                    {
                        actualizando.set(true);
                        espana.set_active(
                            configuracion
                                .teclado
                                .distribuciones
                                .iter()
                                .any(|valor| valor == "españa"),
                        );
                        latinoamerica.set_active(
                            configuracion
                                .teclado
                                .distribuciones
                                .iter()
                                .any(|valor| valor == "latinoamérica"),
                        );
                        actualizando.set(false);
                    }
                }
            }
        });
    }

    {
        let resolucion = entrada_resolucion.clone();
        let hz = entrada_hz.clone();
        let raiz = raiz.clone();
        let mensaje = mensaje_configuracion.clone();
        let aviso = aviso.clone();
        let boton_aplicar = boton_aplicar.clone();
        let ocupado = Rc::clone(&ocupado);

        boton_monitor.connect_clicked(move |_| {
            guardar_monitor(
                &resolucion,
                &hz,
                &raiz,
                &mensaje,
                &aviso,
                &boton_aplicar,
                &ocupado,
            );
        });
    }

    {
        let hz = entrada_hz.clone();
        let raiz = raiz.clone();
        let mensaje = mensaje_configuracion.clone();
        let aviso = aviso.clone();
        let boton_aplicar = boton_aplicar.clone();
        let ocupado = Rc::clone(&ocupado);

        entrada_resolucion.connect_activate(move |resolucion| {
            guardar_monitor(
                resolucion,
                &hz,
                &raiz,
                &mensaje,
                &aviso,
                &boton_aplicar,
                &ocupado,
            );
        });
    }

    {
        let raiz = raiz.clone();
        let mensaje = mensaje_configuracion.clone();
        let aviso = aviso.clone();
        let boton_aplicar = boton_aplicar.clone();
        let ocupado = Rc::clone(&ocupado);
        let actualizando = Rc::clone(&actualizando);

        selector_estilo.connect_selected_notify(move |selector| {
            if actualizando.get() {
                return;
            }

            let Some(estilo) = estilo_por_indice(selector.selected()) else {
                return;
            };

            let configuracion = match configuracion::leer(&raiz.join("configuracion.toml")) {
                Ok(configuracion) => configuracion,
                Err(error) => {
                    mensaje.set_text(&error);
                    return;
                }
            };

            match configuracion::cambiar_apariencia(
                &raiz.join("configuracion.toml"),
                estilo,
                &configuracion.apariencia.modo,
            ) {
                Ok(true) => mensaje_guardado(
                    &mensaje,
                    &raiz,
                    &aviso,
                    &boton_aplicar,
                    &ocupado,
                    &format!("✓ Estilo cambiado a «{estilo}». NixOS todavía no cambió."),
                ),
                Ok(false) => mensaje.set_text("Ese estilo ya estaba elegido."),
                Err(error) => mensaje.set_text(&error),
            }
        });
    }

    {
        let raiz = raiz.clone();
        let mensaje = mensaje_configuracion.clone();
        let aviso = aviso.clone();
        let boton_aplicar = boton_aplicar.clone();
        let ocupado = Rc::clone(&ocupado);
        let actualizando = Rc::clone(&actualizando);

        selector_modo.connect_selected_notify(move |selector| {
            if actualizando.get() {
                return;
            }

            let Some(modo) = modo_por_indice(selector.selected()) else {
                return;
            };

            let configuracion = match configuracion::leer(&raiz.join("configuracion.toml")) {
                Ok(configuracion) => configuracion,
                Err(error) => {
                    mensaje.set_text(&error);
                    return;
                }
            };

            match configuracion::cambiar_apariencia(
                &raiz.join("configuracion.toml"),
                &configuracion.apariencia.estilo,
                modo,
            ) {
                Ok(true) => mensaje_guardado(
                    &mensaje,
                    &raiz,
                    &aviso,
                    &boton_aplicar,
                    &ocupado,
                    &format!("✓ Modo cambiado a «{modo}». NixOS todavía no cambió."),
                ),
                Ok(false) => mensaje.set_text("Ese modo ya estaba elegido."),
                Err(error) => mensaje.set_text(&error),
            }
        });
    }

    {
        let raiz = raiz.clone();
        let mensaje = mensaje_configuracion.clone();
        let aviso = aviso.clone();
        let boton_aplicar = boton_aplicar.clone();
        let ocupado = Rc::clone(&ocupado);
        let actualizando = Rc::clone(&actualizando);

        bluetooth.connect_active_notify(move |fila| {
            if actualizando.get() {
                return;
            }

            let activo = fila.is_active();

            match configuracion::cambiar_bluetooth(&raiz.join("configuracion.toml"), activo) {
                Ok(true) => mensaje_guardado(
                    &mensaje,
                    &raiz,
                    &aviso,
                    &boton_aplicar,
                    &ocupado,
                    &format!(
                        "✓ Bluetooth {} en la configuración. NixOS todavía no cambió.",
                        palabra_estado(activo)
                    ),
                ),
                Ok(false) => {}
                Err(error) => mensaje.set_text(&error),
            }
        });
    }

    {
        let raiz = raiz.clone();
        let mensaje = mensaje_configuracion.clone();
        let aviso = aviso.clone();
        let boton_aplicar = boton_aplicar.clone();
        let ocupado = Rc::clone(&ocupado);
        let actualizando = Rc::clone(&actualizando);

        sunshine.connect_active_notify(move |fila| {
            if actualizando.get() {
                return;
            }

            let configuracion = match configuracion::leer(&raiz.join("configuracion.toml")) {
                Ok(configuracion) => configuracion,
                Err(error) => {
                    mensaje.set_text(&error);
                    return;
                }
            };
            let activo = fila.is_active();

            match configuracion::cambiar_sunshine(
                &raiz.join("configuracion.toml"),
                activo,
                configuracion.sunshine.autoinicio,
            ) {
                Ok(true) => mensaje_guardado(
                    &mensaje,
                    &raiz,
                    &aviso,
                    &boton_aplicar,
                    &ocupado,
                    &format!(
                        "✓ Sunshine {}. Su preferencia de autoinicio se conservó. NixOS todavía no cambió.",
                        palabra_estado(activo)
                    ),
                ),
                Ok(false) => {}
                Err(error) => mensaje.set_text(&error),
            }
        });
    }

    {
        let raiz = raiz.clone();
        let mensaje = mensaje_configuracion.clone();
        let aviso = aviso.clone();
        let boton_aplicar = boton_aplicar.clone();
        let ocupado = Rc::clone(&ocupado);
        let actualizando = Rc::clone(&actualizando);

        sunshine_autoinicio.connect_active_notify(move |fila| {
            if actualizando.get() {
                return;
            }

            let configuracion = match configuracion::leer(&raiz.join("configuracion.toml")) {
                Ok(configuracion) => configuracion,
                Err(error) => {
                    mensaje.set_text(&error);
                    return;
                }
            };
            let autoinicio = fila.is_active();

            match configuracion::cambiar_sunshine(
                &raiz.join("configuracion.toml"),
                configuracion.sunshine.activo,
                autoinicio,
            ) {
                Ok(true) => mensaje_guardado(
                    &mensaje,
                    &raiz,
                    &aviso,
                    &boton_aplicar,
                    &ocupado,
                    "✓ Cambié el autoinicio de Sunshine. NixOS todavía no cambió.",
                ),
                Ok(false) => {}
                Err(error) => mensaje.set_text(&error),
            }
        });
    }

    {
        let raiz = raiz.clone();
        let mensaje = mensaje_configuracion.clone();
        let aviso = aviso.clone();
        let boton_aplicar = boton_aplicar.clone();
        let ocupado = Rc::clone(&ocupado);
        let actualizando = Rc::clone(&actualizando);

        steam.connect_active_notify(move |fila| {
            if actualizando.get() {
                return;
            }

            let configuracion = match configuracion::leer(&raiz.join("configuracion.toml")) {
                Ok(configuracion) => configuracion,
                Err(error) => {
                    mensaje.set_text(&error);
                    return;
                }
            };
            let activo = fila.is_active();

            match configuracion::cambiar_steam(
                &raiz.join("configuracion.toml"),
                activo,
                configuracion.steam.remote_play,
                configuracion.steam.servidor_dedicado,
            ) {
                Ok(true) => mensaje_guardado(
                    &mensaje,
                    &raiz,
                    &aviso,
                    &boton_aplicar,
                    &ocupado,
                    &format!(
                        "✓ Steam {}. Remote Play y servidor dedicado conservaron sus preferencias. NixOS todavía no cambió.",
                        palabra_estado(activo)
                    ),
                ),
                Ok(false) => {}
                Err(error) => mensaje.set_text(&error),
            }
        });
    }

    {
        let raiz = raiz.clone();
        let mensaje = mensaje_configuracion.clone();
        let aviso = aviso.clone();
        let boton_aplicar = boton_aplicar.clone();
        let ocupado = Rc::clone(&ocupado);
        let actualizando = Rc::clone(&actualizando);

        steam_remote_play.connect_active_notify(move |fila| {
            if actualizando.get() {
                return;
            }

            let configuracion = match configuracion::leer(&raiz.join("configuracion.toml")) {
                Ok(configuracion) => configuracion,
                Err(error) => {
                    mensaje.set_text(&error);
                    return;
                }
            };

            match configuracion::cambiar_steam(
                &raiz.join("configuracion.toml"),
                configuracion.steam.activo,
                fila.is_active(),
                configuracion.steam.servidor_dedicado,
            ) {
                Ok(true) => mensaje_guardado(
                    &mensaje,
                    &raiz,
                    &aviso,
                    &boton_aplicar,
                    &ocupado,
                    "✓ Cambié Steam Remote Play. NixOS todavía no cambió.",
                ),
                Ok(false) => {}
                Err(error) => mensaje.set_text(&error),
            }
        });
    }

    {
        let raiz = raiz.clone();
        let mensaje = mensaje_configuracion.clone();
        let aviso = aviso.clone();
        let boton_aplicar = boton_aplicar.clone();
        let ocupado = Rc::clone(&ocupado);
        let actualizando = Rc::clone(&actualizando);

        steam_servidor.connect_active_notify(move |fila| {
            if actualizando.get() {
                return;
            }

            let configuracion = match configuracion::leer(&raiz.join("configuracion.toml")) {
                Ok(configuracion) => configuracion,
                Err(error) => {
                    mensaje.set_text(&error);
                    return;
                }
            };

            match configuracion::cambiar_steam(
                &raiz.join("configuracion.toml"),
                configuracion.steam.activo,
                configuracion.steam.remote_play,
                fila.is_active(),
            ) {
                Ok(true) => mensaje_guardado(
                    &mensaje,
                    &raiz,
                    &aviso,
                    &boton_aplicar,
                    &ocupado,
                    "✓ Cambié el servidor dedicado de Steam. NixOS todavía no cambió.",
                ),
                Ok(false) => {}
                Err(error) => mensaje.set_text(&error),
            }
        });
    }

    {
        let raiz = raiz.clone();
        let mensaje = mensaje_configuracion.clone();
        let aviso = aviso.clone();
        let boton_aplicar = boton_aplicar.clone();
        let ocupado = Rc::clone(&ocupado);
        let actualizando = Rc::clone(&actualizando);

        impresion.connect_active_notify(move |fila| {
            if actualizando.get() {
                return;
            }

            let activa = fila.is_active();

            match configuracion::cambiar_impresion(&raiz.join("configuracion.toml"), activa) {
                Ok(true) => mensaje_guardado(
                    &mensaje,
                    &raiz,
                    &aviso,
                    &boton_aplicar,
                    &ocupado,
                    &format!(
                        "✓ Impresión {} en la configuración. NixOS todavía no cambió.",
                        palabra_estado(activa)
                    ),
                ),
                Ok(false) => {}
                Err(error) => mensaje.set_text(&error),
            }
        });
    }

    {
        let raiz = raiz.clone();
        let mensaje = mensaje_configuracion.clone();
        let aviso = aviso.clone();
        let boton_aplicar = boton_aplicar.clone();
        let ocupado = Rc::clone(&ocupado);
        let actualizando = Rc::clone(&actualizando);

        virtualizacion.connect_active_notify(move |fila| {
            if actualizando.get() {
                return;
            }

            let activa = fila.is_active();

            match configuracion::cambiar_virtualizacion(&raiz.join("configuracion.toml"), activa) {
                Ok(true) => mensaje_guardado(
                    &mensaje,
                    &raiz,
                    &aviso,
                    &boton_aplicar,
                    &ocupado,
                    &format!(
                        "✓ Virtualización {} en la configuración. NixOS todavía no cambió.",
                        palabra_estado(activa)
                    ),
                ),
                Ok(false) => {}
                Err(error) => mensaje.set_text(&error),
            }
        });
    }

    {
        let entrada = entrada_aplicacion.clone();
        let lista = lista_aplicaciones.clone();
        let contador = contador_aplicaciones.clone();
        let raiz = raiz.clone();
        let mensaje = mensaje_configuracion.clone();
        let aviso = aviso.clone();
        let boton_aplicar = boton_aplicar.clone();
        let ocupado = Rc::clone(&ocupado);

        boton_agregar.connect_clicked(move |_| {
            agregar_aplicacion(
                &entrada,
                &lista,
                &contador,
                &raiz,
                &mensaje,
                &aviso,
                &boton_aplicar,
                &ocupado,
            );
        });
    }

    {
        let lista = lista_aplicaciones.clone();
        let contador = contador_aplicaciones.clone();
        let raiz = raiz.clone();
        let mensaje = mensaje_configuracion.clone();
        let aviso = aviso.clone();
        let boton_aplicar = boton_aplicar.clone();
        let ocupado = Rc::clone(&ocupado);

        entrada_aplicacion.connect_activate(move |entrada| {
            agregar_aplicacion(
                entrada,
                &lista,
                &contador,
                &raiz,
                &mensaje,
                &aviso,
                &boton_aplicar,
                &ocupado,
            );
        });
    }

    let refrescar: AlTerminar = {
        let raiz = raiz.clone();
        let entrada_nombre = entrada_nombre.clone();
        let selector_canal = selector_canal.clone();
        let selector_escritorio = selector_escritorio.clone();
        let escritorio_niri = escritorio_niri.clone();
        let escritorio_hyprland = escritorio_hyprland.clone();
        let escritorio_plasma = escritorio_plasma.clone();
        let escritorio_cinnamon = escritorio_cinnamon.clone();
        let teclado_espana = teclado_espana.clone();
        let teclado_latinoamerica = teclado_latinoamerica.clone();
        let entrada_resolucion = entrada_resolucion.clone();
        let entrada_hz = entrada_hz.clone();
        let filas_almacenamiento = Rc::clone(&filas_almacenamiento);
        let selector_estilo = selector_estilo.clone();
        let selector_modo = selector_modo.clone();
        let bluetooth = bluetooth.clone();
        let sunshine = sunshine.clone();
        let sunshine_autoinicio = sunshine_autoinicio.clone();
        let steam = steam.clone();
        let steam_remote_play = steam_remote_play.clone();
        let steam_servidor = steam_servidor.clone();
        let impresion = impresion.clone();
        let virtualizacion = virtualizacion.clone();
        let lista = lista_aplicaciones.clone();
        let contador = contador_aplicaciones.clone();
        let mensaje = mensaje_configuracion.clone();
        let aviso = aviso.clone();
        let boton_aplicar = boton_aplicar.clone();
        let ocupado = Rc::clone(&ocupado);
        let actualizando = Rc::clone(&actualizando);
        let selector_destino = selector_destino.clone();
        let destinos_transferencia = Rc::clone(&destinos_transferencia);
        let transferencia_estado = transferencia_estado.clone();

        Rc::new(move |_| {
            recargar_controles(
                &raiz,
                &entrada_nombre,
                &selector_canal,
                &selector_escritorio,
                &escritorio_niri,
                &escritorio_hyprland,
                &escritorio_plasma,
                &escritorio_cinnamon,
                &teclado_espana,
                &teclado_latinoamerica,
                &entrada_resolucion,
                &entrada_hz,
                &filas_almacenamiento,
                &selector_estilo,
                &selector_modo,
                &bluetooth,
                &sunshine,
                &sunshine_autoinicio,
                &steam,
                &steam_remote_play,
                &steam_servidor,
                &impresion,
                &virtualizacion,
                &lista,
                &contador,
                &mensaje,
                &aviso,
                &boton_aplicar,
                &ocupado,
                &actualizando,
            );

            if let Err(error) =
                actualizar_destinos_transferencia(&selector_destino, &destinos_transferencia, &raiz)
            {
                transferencia_estado.set_text(&error);
            }
        })
    };

    {
        let raiz = raiz.clone();
        let motor = motor.clone();
        let vista = actualizaciones_salida.clone();
        let salida_visible = actualizaciones_salida_visible.clone();
        let estado_actualizaciones = actualizaciones_estado.clone();
        let ocupado_actualizaciones = Rc::clone(&ocupado_actualizaciones);
        let boton_buscar = boton_buscar_actualizaciones.clone();
        let boton_revisar = boton_revisar_actualizacion.clone();
        let boton_aplicar_actualizacion = boton_aplicar_actualizacion.clone();
        let boton_preview_sistema = boton_preview.clone();
        let boton_aplicar_sistema = boton_aplicar.clone();
        let boton_volver_sistema = boton_volver.clone();
        let selector_canal = selector_canal.clone();

        boton_buscar_actualizaciones.connect_clicked(move |_| {
            let controles_busqueda: Vec<gtk::Widget> = vec![
                boton_buscar.clone().upcast(),
                boton_revisar.clone().upcast(),
                boton_aplicar_actualizacion.clone().upcast(),
                boton_preview_sistema.clone().upcast(),
                boton_aplicar_sistema.clone().upcast(),
                boton_volver_sistema.clone().upcast(),
                selector_canal.clone().upcast(),
            ];

            boton_aplicar_actualizacion.set_sensitive(false);

            let raiz_final = raiz.clone();
            let boton_final = boton_aplicar_actualizacion.clone();
            let ocupado_final = Rc::clone(&ocupado_actualizaciones);

            let al_terminar: AlTerminar = Rc::new(move |correcto| {
                if correcto {
                    boton_final.set_sensitive(false);
                } else {
                    actualizar_boton_aplicar_actualizacion(
                        &raiz_final,
                        &boton_final,
                        &ocupado_final,
                    );
                }
            });

            iniciar_operacion(
                "Buscando actualizaciones sin cambiar NixOS",
                &["actualizaciones", "buscar"],
                false,
                &raiz,
                &motor,
                &vista,
                &salida_visible,
                &estado_actualizaciones,
                &controles_busqueda,
                &ocupado_actualizaciones,
                Some(al_terminar),
            );
        });
    }

    {
        let raiz = raiz.clone();
        let motor = motor.clone();
        let vista = actualizaciones_salida.clone();
        let salida_visible = actualizaciones_salida_visible.clone();
        let estado_actualizaciones = actualizaciones_estado.clone();
        let controles = controles.clone();
        let ocupado = Rc::clone(&ocupado);
        let boton_aplicar_actualizacion = boton_aplicar_actualizacion.clone();
        let refrescar = Rc::clone(&refrescar);

        boton_revisar_actualizacion.connect_clicked(move |_| {
            boton_aplicar_actualizacion.set_sensitive(false);

            let raiz_final = raiz.clone();
            let boton_final = boton_aplicar_actualizacion.clone();
            let ocupado_final = Rc::clone(&ocupado);
            let refrescar_final = Rc::clone(&refrescar);

            let al_terminar: AlTerminar = Rc::new(move |correcto| {
                refrescar_final(correcto);

                if correcto {
                    actualizar_boton_aplicar_actualizacion(
                        &raiz_final,
                        &boton_final,
                        &ocupado_final,
                    );
                } else {
                    boton_final.set_sensitive(false);
                }
            });

            iniciar_operacion(
                "Construyendo el preview de la actualización",
                &["actualizaciones", "preview"],
                false,
                &raiz,
                &motor,
                &vista,
                &salida_visible,
                &estado_actualizaciones,
                &controles,
                &ocupado,
                Some(al_terminar),
            );
        });
    }

    {
        let raiz = raiz.clone();
        let motor = motor.clone();
        let vista = actualizaciones_salida.clone();
        let salida_visible = actualizaciones_salida_visible.clone();
        let estado_actualizaciones = actualizaciones_estado.clone();
        let controles = controles.clone();
        let ocupado = Rc::clone(&ocupado);
        let boton_aplicar_actualizacion = boton_aplicar_actualizacion.clone();
        let refrescar = Rc::clone(&refrescar);

        boton_aplicar_actualizacion
            .clone()
            .connect_clicked(move |_| {
                let raiz_final = raiz.clone();
                let boton_final = boton_aplicar_actualizacion.clone();
                let ocupado_final = Rc::clone(&ocupado);
                let refrescar_final = Rc::clone(&refrescar);

                let al_terminar: AlTerminar = Rc::new(move |correcto| {
                    refrescar_final(correcto);
                    actualizar_boton_aplicar_actualizacion(
                        &raiz_final,
                        &boton_final,
                        &ocupado_final,
                    );
                });

                iniciar_operacion(
                    "Aplicando exactamente la actualización revisada",
                    &["aplicar"],
                    true,
                    &raiz,
                    &motor,
                    &vista,
                    &salida_visible,
                    &estado_actualizaciones,
                    &controles,
                    &ocupado,
                    Some(al_terminar),
                );
            });
    }

    {
        let ventana = ventana.clone();
        let raiz = raiz.clone();
        let motor = motor.clone();
        let salida = salida.clone();
        let salida_visible = salida_visible.clone();
        let estado = estado.clone();
        let controles = controles.clone();
        let ocupado = Rc::clone(&ocupado);
        let seleccionada = Rc::clone(&copia_seleccionada);
        let revisada = Rc::clone(&copia_revisada);
        let archivo_texto = copia_archivo.clone();
        let copia_estado = copia_estado.clone();
        let boton_restaurar = boton_restaurar_copia.clone();

        boton_crear_copia.connect_clicked(move |_| {
            if ocupado.get() {
                return;
            }

            let dialogo = gtk::FileChooserNative::new(
                Some("Guardar copia de Korunix"),
                Some(&ventana),
                gtk::FileChooserAction::Save,
                Some("Guardar"),
                Some("Cancelar"),
            );
            dialogo.set_current_name("Copia de Korunix.korunix-copia");
            dialogo.add_filter(&filtro_copias());

            let raiz = raiz.clone();
            let motor = motor.clone();
            let salida = salida.clone();
            let salida_visible = salida_visible.clone();
            let estado = estado.clone();
            let controles = controles.clone();
            let ocupado = Rc::clone(&ocupado);
            let seleccionada = Rc::clone(&seleccionada);
            let revisada = Rc::clone(&revisada);
            let archivo_texto = archivo_texto.clone();
            let copia_estado = copia_estado.clone();
            let boton_restaurar = boton_restaurar.clone();

            dialogo.connect_response(move |dialogo, respuesta| {
                if respuesta != gtk::ResponseType::Accept {
                    return;
                }

                let Some(ruta) = dialogo.file().and_then(|archivo| archivo.path()) else {
                    copia_estado.set_text("No pude obtener la ruta elegida para la copia.");
                    return;
                };

                let ruta = asegurar_extension_copia(ruta);
                let ruta_texto = ruta.to_string_lossy().into_owned();
                let ruta_final = ruta.clone();
                let seleccionada_final = Rc::clone(&seleccionada);
                let revisada_final = Rc::clone(&revisada);
                let archivo_final = archivo_texto.clone();
                let estado_final = copia_estado.clone();
                let restaurar_final = boton_restaurar.clone();

                let al_terminar: AlTerminar = Rc::new(move |correcto| {
                    if correcto {
                        archivo_final.set_text(&nombre_archivo_humano(&ruta_final));
                        *seleccionada_final.borrow_mut() = Some(ruta_final.clone());
                        *revisada_final.borrow_mut() = None;
                        restaurar_final.set_sensitive(false);
                        estado_final.set_text(
                            "✓ Copia creada y verificada. Puedes revisarla antes de restaurarla.",
                        );
                    }
                });

                iniciar_operacion(
                    "Creando una copia portable de Korunix",
                    &["copias", "crear", &ruta_texto],
                    false,
                    &raiz,
                    &motor,
                    &salida,
                    &salida_visible,
                    &estado,
                    &controles,
                    &ocupado,
                    Some(al_terminar),
                );
            });

            dialogo.show();
        });
    }

    {
        let ventana = ventana.clone();
        let seleccionada = Rc::clone(&copia_seleccionada);
        let revisada = Rc::clone(&copia_revisada);
        let archivo_texto = copia_archivo.clone();
        let copia_estado = copia_estado.clone();
        let boton_restaurar = boton_restaurar_copia.clone();

        boton_elegir_copia.connect_clicked(move |_| {
            let dialogo = gtk::FileChooserNative::new(
                Some("Elegir copia de Korunix"),
                Some(&ventana),
                gtk::FileChooserAction::Open,
                Some("Elegir"),
                Some("Cancelar"),
            );
            dialogo.add_filter(&filtro_copias());

            let seleccionada = Rc::clone(&seleccionada);
            let revisada = Rc::clone(&revisada);
            let archivo_texto = archivo_texto.clone();
            let copia_estado = copia_estado.clone();
            let boton_restaurar = boton_restaurar.clone();

            dialogo.connect_response(move |dialogo, respuesta| {
                if respuesta != gtk::ResponseType::Accept {
                    return;
                }

                if let Some(ruta) = dialogo.file().and_then(|archivo| archivo.path()) {
                    archivo_texto.set_text(&nombre_archivo_humano(&ruta));
                    *seleccionada.borrow_mut() = Some(ruta);
                    *revisada.borrow_mut() = None;
                    boton_restaurar.set_sensitive(false);
                    copia_estado.set_text(
                        "Copia elegida. Revisa primero qué recuperaría antes de restaurarla.",
                    );
                }
            });

            dialogo.show();
        });
    }

    {
        let raiz = raiz.clone();
        let motor = motor.clone();
        let salida = salida.clone();
        let salida_visible = salida_visible.clone();
        let estado = estado.clone();
        let controles = controles.clone();
        let ocupado = Rc::clone(&ocupado);
        let seleccionada = Rc::clone(&copia_seleccionada);
        let revisada = Rc::clone(&copia_revisada);
        let copia_estado = copia_estado.clone();
        let boton_restaurar = boton_restaurar_copia.clone();

        boton_revisar_copia.connect_clicked(move |_| {
            let Some(ruta) = seleccionada.borrow().clone() else {
                copia_estado.set_text("Elige primero la copia que quieres revisar.");
                return;
            };

            *revisada.borrow_mut() = None;
            boton_restaurar.set_sensitive(false);

            let ruta_texto = ruta.to_string_lossy().into_owned();
            let ruta_revisada = ruta.clone();
            let revisada_final = Rc::clone(&revisada);
            let copia_estado_final = copia_estado.clone();
            let boton_restaurar_final = boton_restaurar.clone();

            let al_terminar: AlTerminar = Rc::new(move |correcto| {
                if correcto {
                    *revisada_final.borrow_mut() = Some(ruta_revisada.clone());
                    boton_restaurar_final.set_sensitive(true);
                    copia_estado_final.set_text(
                        "✓ Plan revisado. «Restaurar esta copia» recuperará exactamente esa copia y protegerá primero lo que tienes ahora.",
                    );
                } else {
                    boton_restaurar_final.set_sensitive(false);
                    copia_estado_final.set_text(
                        "No voy a habilitar Restaurar porque la copia o su Plan necesitan revisión.",
                    );
                }
            });

            iniciar_operacion(
                "Revisando qué recuperaría esta copia",
                &["copias", "plan-restaurar", &ruta_texto],
                false,
                &raiz,
                &motor,
                &salida,
                &salida_visible,
                &estado,
                &controles,
                &ocupado,
                Some(al_terminar),
            );
        });
    }

    {
        let raiz = raiz.clone();
        let motor = motor.clone();
        let salida = salida.clone();
        let salida_visible = salida_visible.clone();
        let estado = estado.clone();
        let controles = controles.clone();
        let ocupado = Rc::clone(&ocupado);
        let seleccionada = Rc::clone(&copia_seleccionada);
        let revisada = Rc::clone(&copia_revisada);
        let copia_estado = copia_estado.clone();
        let boton_restaurar = boton_restaurar_copia.clone();
        let refrescar = Rc::clone(&refrescar);

        boton_restaurar_copia.connect_clicked(move |_| {
            if ocupado.get() {
                return;
            }

            let Some(ruta) = seleccionada.borrow().clone() else {
                copia_estado.set_text("Elige primero la copia que quieres restaurar.");
                boton_restaurar.set_sensitive(false);
                return;
            };

            if revisada.borrow().as_ref() != Some(&ruta) {
                copia_estado.set_text(
                    "La copia elegida no es la que se revisó. Revisa el Plan otra vez.",
                );
                boton_restaurar.set_sensitive(false);
                return;
            }

            let ruta_texto = ruta.to_string_lossy().into_owned();
            let revisada_final = Rc::clone(&revisada);
            let copia_estado_final = copia_estado.clone();
            let boton_restaurar_final = boton_restaurar.clone();
            let refrescar_final = Rc::clone(&refrescar);

            let al_terminar: AlTerminar = Rc::new(move |correcto| {
                if correcto {
                    *revisada_final.borrow_mut() = None;
                    boton_restaurar_final.set_sensitive(false);
                    copia_estado_final.set_text(
                        "✓ Restauración terminada. La configuración ya cambió; NixOS todavía no. Crea un preview antes de aplicar.",
                    );
                    refrescar_final(true);
                }
            });

            iniciar_operacion(
                "Restaurando la copia revisada",
                &["copias", "restaurar", &ruta_texto],
                false,
                &raiz,
                &motor,
                &salida,
                &salida_visible,
                &estado,
                &controles,
                &ocupado,
                Some(al_terminar),
            );
        });
    }

    {
        let raiz = raiz.clone();
        let motor = motor.clone();
        let salida = salida.clone();
        let salida_visible = salida_visible.clone();
        let estado = estado.clone();
        let controles = controles.clone();
        let ocupado = Rc::clone(&ocupado);

        boton_historial.connect_clicked(move |_| {
            iniciar_operacion(
                "Leyendo el Historial local",
                &["historial"],
                false,
                &raiz,
                &motor,
                &salida,
                &salida_visible,
                &estado,
                &controles,
                &ocupado,
                None,
            );
        });
    }

    {
        let raiz = raiz.clone();
        let motor = motor.clone();
        let salida = salida.clone();
        let salida_visible = salida_visible.clone();
        let estado = estado.clone();
        let controles = controles.clone();
        let ocupado = Rc::clone(&ocupado);
        let refrescar = Rc::clone(&refrescar);

        boton_preview.connect_clicked(move |_| {
            iniciar_operacion(
                "Construyendo un preview completo",
                &["preview"],
                false,
                &raiz,
                &motor,
                &salida,
                &salida_visible,
                &estado,
                &controles,
                &ocupado,
                Some(Rc::clone(&refrescar)),
            );
        });
    }

    {
        let raiz = raiz.clone();
        let motor = motor.clone();
        let salida = salida.clone();
        let salida_visible = salida_visible.clone();
        let estado = estado.clone();
        let controles = controles.clone();
        let ocupado = Rc::clone(&ocupado);
        let refrescar = Rc::clone(&refrescar);

        boton_aplicar.connect_clicked(move |_| {
            iniciar_operacion(
                "Aplicando el preview revisado",
                &["aplicar"],
                true,
                &raiz,
                &motor,
                &salida,
                &salida_visible,
                &estado,
                &controles,
                &ocupado,
                Some(Rc::clone(&refrescar)),
            );
        });
    }

    {
        let raiz = raiz.clone();
        let motor = motor.clone();
        let salida = salida.clone();
        let salida_visible = salida_visible.clone();
        let estado = estado.clone();
        let controles = controles.clone();
        let ocupado = Rc::clone(&ocupado);
        let refrescar = Rc::clone(&refrescar);

        boton_volver.connect_clicked(move |_| {
            iniciar_operacion(
                "Volviendo a la generación anterior",
                &["rollback"],
                true,
                &raiz,
                &motor,
                &salida,
                &salida_visible,
                &estado,
                &controles,
                &ocupado,
                Some(Rc::clone(&refrescar)),
            );
        });
    }

    recargar_controles(
        &raiz,
        &entrada_nombre,
        &selector_canal,
        &selector_escritorio,
        &escritorio_niri,
        &escritorio_hyprland,
        &escritorio_plasma,
        &escritorio_cinnamon,
        &teclado_espana,
        &teclado_latinoamerica,
        &entrada_resolucion,
        &entrada_hz,
        &filas_almacenamiento,
        &selector_estilo,
        &selector_modo,
        &bluetooth,
        &sunshine,
        &sunshine_autoinicio,
        &steam,
        &steam_remote_play,
        &steam_servidor,
        &impresion,
        &virtualizacion,
        &lista_aplicaciones,
        &contador_aplicaciones,
        &mensaje_configuracion,
        &aviso,
        &boton_aplicar,
        &ocupado,
        &actualizando,
    );

    if let Err(error) =
        actualizar_destinos_transferencia(&selector_destino, &destinos_transferencia, &raiz)
    {
        transferencia_estado.set_text(&error);
    }

    ventana.present();

    {
        let controles_actualizaciones: Vec<gtk::Widget> = vec![
            boton_buscar_actualizaciones.clone().upcast(),
            boton_revisar_actualizacion.clone().upcast(),
            boton_aplicar_actualizacion.clone().upcast(),
        ];

        let raiz_final = raiz.clone();
        let boton_final = boton_aplicar_actualizacion.clone();
        let ocupado_final = Rc::clone(&ocupado_actualizaciones);

        let al_terminar: AlTerminar = Rc::new(move |_| {
            actualizar_boton_aplicar_actualizacion(&raiz_final, &boton_final, &ocupado_final);
        });

        iniciar_operacion(
            "Leyendo el estado local de Actualizaciones",
            &["actualizaciones"],
            false,
            &raiz,
            &motor,
            &actualizaciones_salida,
            &actualizaciones_salida_visible,
            &actualizaciones_estado,
            &controles_actualizaciones,
            &ocupado_actualizaciones,
            Some(al_terminar),
        );
    }

    cargar_almacenamiento(
        &almacenamiento_lista,
        &almacenamiento_estado,
        &filas_almacenamiento,
        &raiz,
        &mensaje_configuracion,
        &aviso,
        &boton_aplicar,
        &ocupado,
        &actualizando,
        &controles,
        &selector_destino,
        &destinos_transferencia,
    );
}

fn main() -> glib::ExitCode {
    let aplicacion = Application::builder().application_id(ID_APLICACION).build();

    aplicacion.connect_activate(construir_ventana);
    aplicacion.run()
}

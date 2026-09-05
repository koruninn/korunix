#[allow(dead_code)]
mod configuracion;

use adw::prelude::*;
use adw::{Application, ApplicationWindow, HeaderBar, ToolbarView};
use gtk::glib;
use std::cell::Cell;
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

enum Mensaje {
    Linea(String),
    Terminado(bool),
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
    let contenido = gtk::Box::new(gtk::Orientation::Vertical, 18);
    contenido.set_margin_top(18);
    contenido.set_margin_bottom(24);
    contenido.set_margin_start(18);
    contenido.set_margin_end(18);

    let titulo = gtk::Label::new(Some("Korunix"));
    titulo.add_css_class("title-1");
    titulo.set_halign(gtk::Align::Start);

    let subtitulo = gtk::Label::new(Some(
        "Lo que cambies aquí se guarda como una decisión humana. NixOS solo cambia después de revisar y aplicar un preview.",
    ));
    subtitulo.set_wrap(true);
    subtitulo.set_halign(gtk::Align::Start);
    subtitulo.add_css_class("dim-label");

    contenido.append(&titulo);
    contenido.append(&subtitulo);

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
    contenido.append(&aviso);

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
    contenido.append(&configuracion_grupo);

    let equipo_grupo = adw::PreferencesGroup::builder()
        .title("Sesión y equipo")
        .description("Aquí eliges qué escritorios quedan disponibles, tus teclados y la pantalla.")
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

    let monitor_titulo = gtk::Label::new(Some("Pantalla"));
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

    equipo_grupo.add(&escritorio_niri);
    equipo_grupo.add(&escritorio_hyprland);
    equipo_grupo.add(&escritorio_plasma);
    equipo_grupo.add(&escritorio_cinnamon);
    equipo_grupo.add(&teclado_espana);
    equipo_grupo.add(&teclado_latinoamerica);
    equipo_grupo.add(&cambio_teclado);
    equipo_grupo.add(&monitor_bloque);
    contenido.append(&equipo_grupo);

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
    contenido.append(&apariencia_grupo);

    let funciones_grupo = adw::PreferencesGroup::builder()
        .title("Funciones")
        .description(
            "Apagar una función no borra sus preferencias internas. Solo deja de aplicarlas mientras esté apagada.",
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

    funciones_grupo.add(&bluetooth);
    funciones_grupo.add(&sunshine);
    funciones_grupo.add(&sunshine_autoinicio);
    funciones_grupo.add(&steam);
    funciones_grupo.add(&steam_remote_play);
    funciones_grupo.add(&steam_servidor);
    funciones_grupo.add(&impresion);
    funciones_grupo.add(&virtualizacion);
    contenido.append(&funciones_grupo);

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
    contenido.append(&aplicaciones_grupo);

    let mensaje_configuracion = gtk::Label::new(Some(
        "Los cambios de esta sección se guardan enseguida, pero NixOS permanece igual.",
    ));
    mensaje_configuracion.set_wrap(true);
    mensaje_configuracion.set_halign(gtk::Align::Start);
    mensaje_configuracion.add_css_class("dim-label");
    contenido.append(&mensaje_configuracion);

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
    contenido.append(&acciones);

    let estado = gtk::Label::new(Some("Listo"));
    estado.set_halign(gtk::Align::Start);
    estado.add_css_class("heading");
    contenido.append(&estado);

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
    contenido.append(&salida_visible);

    let clamp = adw::Clamp::builder()
        .maximum_size(680)
        .child(&contenido)
        .build();

    let desplazamiento_principal = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&clamp)
        .build();

    let vista = ToolbarView::new();
    vista.add_top_bar(&barra);
    vista.set_content(Some(&desplazamiento_principal));
    ventana.set_content(Some(&vista));

    let ocupado = Rc::new(Cell::new(false));
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
        })
    };

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

    ventana.present();
}

fn main() -> glib::ExitCode {
    let aplicacion = Application::builder().application_id(ID_APLICACION).build();

    aplicacion.connect_activate(construir_ventana);
    aplicacion.run()
}

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
use toml::Value;

const ID_APLICACION: &str = "io.github.koruninn.Korunix";

#[derive(Clone)]
struct Resumen {
    nombre: String,
    canal: String,
    escritorio: String,
    apariencia: String,
    aplicaciones: usize,
}

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

fn texto(tabla: &Value, ruta: &[&str], predeterminado: &str) -> String {
    let mut actual = tabla;

    for parte in ruta {
        let Some(siguiente) = actual.get(*parte) else {
            return predeterminado.to_string();
        };
        actual = siguiente;
    }

    actual
        .as_str()
        .map(str::to_string)
        .unwrap_or_else(|| predeterminado.to_string())
}

fn leer_resumen(raiz: &Path) -> Result<Resumen, String> {
    let ruta = raiz.join("configuracion.toml");
    let contenido = fs::read_to_string(&ruta)
        .map_err(|error| format!("No pude leer {}.\nDetalle: {error}", ruta.display()))?;
    let datos = contenido
        .parse::<Value>()
        .map_err(|error| format!("configuracion.toml no se pudo leer.\nDetalle: {error}"))?;

    let estilo = texto(&datos, &["apariencia", "estilo"], "predeterminado");
    let modo = texto(&datos, &["apariencia", "modo"], "automatico");
    let aplicaciones = datos
        .get("aplicaciones")
        .and_then(|valor| valor.get("instaladas"))
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);

    Ok(Resumen {
        nombre: texto(&datos, &["nombre"], "nixos"),
        canal: texto(&datos, &["canal"], "estable"),
        escritorio: texto(&datos, &["escritorio", "principal"], "niri"),
        apariencia: format!("{estilo} · {modo}"),
        aplicaciones,
    })
}

fn fila(titulo: &str, valor: &str) -> adw::ActionRow {
    adw::ActionRow::builder()
        .title(titulo)
        .subtitle(valor)
        .build()
}

fn anexar_linea(vista: &gtk::TextView, linea: &str) {
    let buffer = vista.buffer();
    let mut final_texto = buffer.end_iter();
    buffer.insert(&mut final_texto, linea);
    buffer.insert(&mut final_texto, "\n");
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
    estado: &gtk::Label,
    botones: &[gtk::Button],
    ocupado: &Rc<Cell<bool>>,
) {
    if ocupado.get() {
        return;
    }

    ocupado.set(true);

    for boton in botones {
        boton.set_sensitive(false);
    }

    vista.buffer().set_text("");
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
    let botones = botones.to_vec();
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

                    for boton in &botones {
                        boton.set_sensitive(true);
                    }

                    ocupado.set(false);
                    return glib::ControlFlow::Break;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    estado.set_text("La operación terminó sin respuesta");
                    for boton in &botones {
                        boton.set_sensitive(true);
                    }
                    ocupado.set(false);
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
        .default_height(720)
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
        "Configura y mantiene NixOS sin tener que tocar la parte técnica.",
    ));
    subtitulo.set_wrap(true);
    subtitulo.set_halign(gtk::Align::Start);
    subtitulo.add_css_class("dim-label");

    contenido.append(&titulo);
    contenido.append(&subtitulo);

    let grupo = adw::PreferencesGroup::builder()
        .title("Este equipo")
        .build();

    match leer_resumen(&raiz) {
        Ok(resumen) => {
            grupo.add(&fila("Nombre", &resumen.nombre));
            grupo.add(&fila("Canal", &resumen.canal));
            grupo.add(&fila("Escritorio", &resumen.escritorio));
            grupo.add(&fila("Apariencia", &resumen.apariencia));
            grupo.add(&fila(
                "Aplicaciones elegidas",
                &resumen.aplicaciones.to_string(),
            ));
        }
        Err(error) => {
            grupo.add(&fila("Configuración", &error));
        }
    }

    contenido.append(&grupo);

    let acciones = adw::PreferencesGroup::builder()
        .title("Cambios del sistema")
        .description(
            "La interfaz usa el mismo motor de Korunix. Preview no cambia NixOS; aplicar y volver usan las generaciones ya preparadas.",
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

    let desplazamiento = gtk::ScrolledWindow::builder()
        .min_content_height(190)
        .vexpand(true)
        .child(&salida)
        .build();
    desplazamiento.add_css_class("card");
    contenido.append(&desplazamiento);

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

    let botones = [
        boton_preview.clone(),
        boton_aplicar.clone(),
        boton_volver.clone(),
    ];

    {
        let raiz = raiz.clone();
        let motor = motor.clone();
        let salida = salida.clone();
        let estado = estado.clone();
        let botones = botones.clone();
        let ocupado = Rc::clone(&ocupado);

        boton_preview.connect_clicked(move |_| {
            iniciar_operacion(
                "Construyendo un preview completo",
                &["preview"],
                false,
                &raiz,
                &motor,
                &salida,
                &estado,
                &botones,
                &ocupado,
            );
        });
    }

    {
        let raiz = raiz.clone();
        let motor = motor.clone();
        let salida = salida.clone();
        let estado = estado.clone();
        let botones = botones.clone();
        let ocupado = Rc::clone(&ocupado);

        boton_aplicar.connect_clicked(move |_| {
            iniciar_operacion(
                "Aplicando el preview revisado",
                &["aplicar"],
                true,
                &raiz,
                &motor,
                &salida,
                &estado,
                &botones,
                &ocupado,
            );
        });
    }

    {
        let salida = salida.clone();
        let estado = estado.clone();
        let botones = botones.clone();
        let ocupado = Rc::clone(&ocupado);

        boton_volver.connect_clicked(move |_| {
            iniciar_operacion(
                "Volviendo a la generación anterior",
                &["rollback"],
                true,
                &raiz,
                &motor,
                &salida,
                &estado,
                &botones,
                &ocupado,
            );
        });
    }

    ventana.present();
}

fn main() -> glib::ExitCode {
    let aplicacion = Application::builder().application_id(ID_APLICACION).build();

    aplicacion.connect_activate(construir_ventana);
    aplicacion.run()
}

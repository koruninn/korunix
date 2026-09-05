mod almacenamiento;
mod aplicar;
mod configuracion;
mod preview;
mod rollback;
mod sistema;

use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;

fn es_raiz(ruta: &Path) -> bool {
    ruta.join("configuracion.toml").is_file()
        && ruta.join("flake.nix").is_file()
        && ruta.join("sistema.nix").is_file()
        && ruta.join("hardware.nix").is_file()
}

fn raiz_korunix() -> Result<PathBuf, String> {
    if let Some(valor) = env::var_os("KORUNIX_ROOT") {
        let ruta = PathBuf::from(valor);

        if es_raiz(&ruta) {
            return Ok(ruta);
        }

        return Err(format!(
            "KORUNIX_ROOT no apunta a una carpeta de Korunix: {}",
            ruta.display()
        ));
    }

    let actual = env::current_dir()
        .map_err(|error| format!("No pude leer la carpeta actual.\nDetalle: {error}"))?;

    for ruta in actual.ancestors() {
        if es_raiz(ruta) {
            return Ok(ruta.to_path_buf());
        }
    }

    if let Some(home) = env::var_os("HOME") {
        let ruta = PathBuf::from(home).join(".korunix");

        if es_raiz(&ruta) {
            return Ok(ruta);
        }
    }

    Err("No encontré la carpeta de Korunix.".to_string())
}

fn ayuda() {
    eprintln!("Por ahora puedes usar:");
    eprintln!("  korunix validar");
    eprintln!("  korunix plan");
    eprintln!("  korunix preview");
    eprintln!("  korunix aplicar");
    eprintln!("  korunix rollback");
    eprintln!("  korunix nombre");
    eprintln!("  korunix nombre <nuevo>");
    eprintln!("  korunix personas");
    eprintln!("  korunix escritorio");
    eprintln!("  korunix escritorio <niri|hyprland|cinnamon|plasma>");
    eprintln!("  korunix escritorios");
    eprintln!("  korunix escritorios <niri|hyprland|plasma|cinnamon> <activar|desactivar>");
    eprintln!("  korunix teclado");
    eprintln!("  korunix teclado <españa|latinoamérica> <activar|desactivar>");
    eprintln!("  korunix monitor");
    eprintln!("  korunix monitor <resolucion> <hz>");
    eprintln!("  korunix almacenamiento");
    eprintln!("  korunix almacenamiento <nombre> <activar|desactivar>");
    eprintln!("  korunix canal");
    eprintln!("  korunix canal <estable|inestable>");
    eprintln!("  korunix apariencia");
    eprintln!(
        "  korunix apariencia <predeterminado|dinamico|everforest> <claro|oscuro|automatico>"
    );
    eprintln!("  korunix bluetooth [activar|desactivar]");
    eprintln!("  korunix sunshine [activar|desactivar]");
    eprintln!("  korunix sunshine autoinicio <activar|desactivar>");
    eprintln!("  korunix steam [activar|desactivar]");
    eprintln!("  korunix steam remote-play <activar|desactivar>");
    eprintln!("  korunix steam servidor-dedicado <activar|desactivar>");
    eprintln!("  korunix impresion [activar|desactivar]");
    eprintln!("  korunix virtualizacion [activar|desactivar]");
    eprintln!("  korunix aplicaciones");
    eprintln!("  korunix aplicaciones agregar <nombre>");
    eprintln!("  korunix aplicaciones quitar <nombre>");
}

fn salir_con_error(error: &str) -> ! {
    eprintln!("{error}");
    eprintln!();
    eprintln!("No se cambió nada.");
    process::exit(1);
}

fn validar(raiz: &Path) {
    match configuracion::leer(&raiz.join("configuracion.toml")) {
        Ok(configuracion) => {
            println!("✓ La configuración está bien.");
            println!("Equipo: {}", configuracion.nombre);
            println!("Canal: {}", configuracion.canal);
            println!("Escritorio: {}", configuracion.escritorio.principal);
            println!(
                "Escritorios instalados: {}",
                configuracion.escritorio.instalados_efectivos().join(", ")
            );
            println!(
                "Apariencia: {} · {}",
                configuracion.apariencia.estilo, configuracion.apariencia.modo
            );
            println!(
                "Idioma: {} — {}",
                configuracion.idioma.sistema, configuracion.idioma.region
            );
            println!(
                "Teclados: {}",
                configuracion.teclado.distribuciones.join(", ")
            );
            println!(
                "Monitor: {} @ {} Hz",
                configuracion.monitor.resolucion, configuracion.monitor.hz
            );
            println!("Personas: {}", configuracion.personas.len());
            println!(
                "Unidades disponibles: {}",
                if configuracion.almacenamiento.disponibles.is_empty() {
                    "ninguna".to_string()
                } else {
                    configuracion.almacenamiento.disponibles.join(", ")
                }
            );
            println!(
                "Bluetooth: {}",
                if configuracion.bluetooth.activo {
                    "activo"
                } else {
                    "apagado"
                }
            );
            println!(
                "Sunshine: {} · autoinicio {}",
                if configuracion.sunshine.activo {
                    "activo"
                } else {
                    "apagado"
                },
                if configuracion.sunshine.autoinicio {
                    "sí"
                } else {
                    "no"
                }
            );
            println!(
                "Steam: {} · Remote Play {} · servidor dedicado {}",
                if configuracion.steam.activo {
                    "activo"
                } else {
                    "apagado"
                },
                if configuracion.steam.remote_play {
                    "sí"
                } else {
                    "no"
                },
                if configuracion.steam.servidor_dedicado {
                    "sí"
                } else {
                    "no"
                }
            );
            println!(
                "Impresión: {}",
                if configuracion.impresion.activa {
                    "activa"
                } else {
                    "apagada"
                }
            );
            println!(
                "Virtualización: {}",
                if configuracion.virtualizacion.activa {
                    "activa"
                } else {
                    "apagada"
                }
            );
            println!(
                "Aplicaciones elegidas: {}",
                configuracion.aplicaciones.instaladas.len()
            );
        }
        Err(error) => salir_con_error(&error),
    }
}

fn mostrar_plan(raiz: &Path) {
    let configuracion = match configuracion::leer(&raiz.join("configuracion.toml")) {
        Ok(configuracion) => configuracion,
        Err(error) => salir_con_error(&error),
    };

    let plan = match sistema::preparar_plan(raiz, &configuracion) {
        Ok(plan) => plan,
        Err(error) => salir_con_error(&error),
    };

    println!("Plan");
    println!("Equipo: {}", plan.nombre);
    println!("Canal: {}", plan.canal);
    println!("Escritorio principal: {}", plan.escritorio);
    println!("Escritorios instalados: {}", plan.escritorios.join(", "));
    println!(
        "Apariencia: {} · {}",
        plan.apariencia.estilo, plan.apariencia.modo
    );
    println!(
        "Idioma: {} — {} ({})",
        plan.idioma.sistema, plan.idioma.region, plan.idioma.locale
    );
    println!("Zona horaria: {}", plan.idioma.zona_horaria);
    println!(
        "Teclados: {} · cambio {}",
        plan.teclado.distribuciones.join(", "),
        plan.teclado.cambio
    );
    println!(
        "Monitor: {} @ {} Hz",
        plan.monitor.resolucion, plan.monitor.hz
    );
    println!(
        "Entrada: {}{}",
        plan.entrada.backend,
        if plan.entrada.wayland {
            " (Wayland)"
        } else {
            ""
        }
    );

    if plan.noctalia {
        if plan.noctalia_version.is_empty() {
            println!("Noctalia: activo");
        } else {
            println!("Noctalia: activo ({})", plan.noctalia_version);
        }
    } else {
        println!("Noctalia: no se usa");
    }

    println!("Personas:");

    for persona in plan.personas {
        let tipo = if persona.administrador {
            "administrador"
        } else {
            "usuario"
        };

        let avatar = if persona.avatar.is_some() {
            " · avatar sí"
        } else {
            ""
        };

        let github = if persona.clave_github.is_some() {
            " · GitHub sí"
        } else {
            ""
        };

        println!("  - {} ({tipo}){avatar}{github}", persona.cuenta);
    }

    if plan.aplicaciones.is_empty() {
        println!("Aplicaciones: ninguna");
    } else {
        println!("Aplicaciones:");

        for aplicacion in plan.aplicaciones {
            if aplicacion.version.is_empty() {
                println!("  - {} → {}", aplicacion.elegida, aplicacion.nombre);
            } else {
                println!(
                    "  - {} → {} {}",
                    aplicacion.elegida, aplicacion.nombre, aplicacion.version
                );
            }
        }
    }

    if plan.almacenamiento.is_empty() {
        println!("Unidades disponibles: ninguna");
    } else {
        println!("Unidades disponibles:");

        for unidad in &plan.almacenamiento {
            println!("  - {}", unidad.nombre);
        }
    }

    println!(
        "Bluetooth: {}",
        if plan.bluetooth { "activo" } else { "apagado" }
    );

    println!(
        "Sunshine: {} · autoinicio {}",
        if plan.sunshine.activo {
            "activo"
        } else {
            "apagado"
        },
        if plan.sunshine.autoinicio {
            "sí"
        } else {
            "no"
        }
    );

    println!(
        "Steam: {} · Remote Play {} · servidor dedicado {}",
        if plan.steam.activo {
            "activo"
        } else {
            "apagado"
        },
        if plan.steam.remote_play { "sí" } else { "no" },
        if plan.steam.servidor_dedicado {
            "sí"
        } else {
            "no"
        }
    );

    println!(
        "Impresión: {}{}",
        if plan.impresion.activa {
            "activa"
        } else {
            "apagada"
        },
        plan.impresion
            .controlador
            .as_deref()
            .map(|controlador| format!(" · {controlador}"))
            .unwrap_or_default()
    );

    println!(
        "Virtualización: {}",
        if plan.virtualizacion {
            "activa"
        } else {
            "apagada"
        }
    );

    if !plan.revision.is_empty() {
        println!("Nixpkgs: {}", plan.revision);
    }

    println!();
    println!("NixOS no cambió.");
}

fn preparar_preview(raiz: &Path) {
    let configuracion = match configuracion::leer(&raiz.join("configuracion.toml")) {
        Ok(configuracion) => configuracion,
        Err(error) => salir_con_error(&error),
    };

    if let Err(error) = sistema::preparar_plan(raiz, &configuracion) {
        salir_con_error(&error);
    }

    if let Err(error) = aplicar::conservar_aplicada_actual(raiz) {
        salir_con_error(&error);
    }

    println!("Construyendo el preview de NixOS...");
    let _ = io::stdout().flush();

    match preview::crear(raiz) {
        Ok(preview) => {
            println!("✓ El preview quedó listo.");
            println!("Generación: {}", preview.generacion.display());
            println!("Guardado en: {}", preview.enlace.display());
            println!("NixOS no cambió.");
            println!("Al aplicar, Korunix tendrá que activar exactamente esta generación.");
        }
        Err(error) => salir_con_error(&error),
    }
}

fn mostrar_nombre(raiz: &Path) {
    match configuracion::leer(&raiz.join("configuracion.toml")) {
        Ok(configuracion) => println!("Nombre: {}", configuracion.nombre),
        Err(error) => salir_con_error(&error),
    }
}

fn cambiar_nombre(raiz: &Path, nombre: &str) {
    match configuracion::cambiar_nombre(&raiz.join("configuracion.toml"), nombre) {
        Ok(true) => {
            println!("✓ El nombre ahora es «{nombre}» en configuracion.toml.");
            println!("NixOS todavía no cambió.");
        }
        Ok(false) => {
            println!("El nombre ya era «{nombre}».");
            println!("No cambié nada.");
        }
        Err(error) => salir_con_error(&error),
    }
}

fn mostrar_personas(raiz: &Path) {
    match configuracion::leer(&raiz.join("configuracion.toml")) {
        Ok(configuracion) => {
            println!("Personas:");

            for persona in configuracion.personas {
                let tipo = if persona.administrador {
                    "administrador"
                } else {
                    "usuario"
                };
                let avatar = if persona.avatar.is_some() {
                    " · avatar sí"
                } else {
                    ""
                };
                let github = if persona.clave_github.is_some() {
                    " · GitHub sí"
                } else {
                    ""
                };

                println!(
                    "  - {} — {} ({tipo}){avatar}{github}",
                    persona.cuenta, persona.nombre
                );
            }
        }
        Err(error) => salir_con_error(&error),
    }
}

fn mostrar_escritorio(raiz: &Path) {
    match configuracion::leer(&raiz.join("configuracion.toml")) {
        Ok(configuracion) => {
            println!("Principal: {}", configuracion.escritorio.principal);
            println!(
                "Instalados: {}",
                configuracion.escritorio.instalados_efectivos().join(", ")
            );
        }
        Err(error) => salir_con_error(&error),
    }
}

fn cambiar_escritorio(raiz: &Path, escritorio: &str) {
    match configuracion::cambiar_escritorio(&raiz.join("configuracion.toml"), escritorio) {
        Ok(true) => {
            println!("✓ El escritorio principal ahora es «{escritorio}» en configuracion.toml.");
            println!("NixOS todavía no cambió.");
        }
        Ok(false) => {
            println!("El escritorio principal ya era «{escritorio}».");
            println!("No cambié nada.");
        }
        Err(error) => salir_con_error(&error),
    }
}

fn orden_escritorio(nombre: &str) -> usize {
    match nombre {
        "niri" => 0,
        "hyprland" => 1,
        "plasma" => 2,
        "cinnamon" => 3,
        _ => usize::MAX,
    }
}

fn mostrar_escritorios(raiz: &Path) {
    match configuracion::leer(&raiz.join("configuracion.toml")) {
        Ok(configuracion) => {
            println!(
                "Escritorio principal: {}",
                configuracion.escritorio.principal
            );
            println!(
                "Disponibles: {}",
                configuracion.escritorio.instalados_efectivos().join(", ")
            );
        }
        Err(error) => salir_con_error(&error),
    }
}

fn cambiar_escritorio_instalado(raiz: &Path, nombre: &str, valor: &str) {
    if !matches!(nombre, "niri" | "hyprland" | "plasma" | "cinnamon") {
        salir_con_error(&format!(
            "No conozco el escritorio «{nombre}».\nUsa niri, hyprland, plasma o cinnamon."
        ));
    }

    let activo = leer_interruptor(valor).unwrap_or_else(|error| salir_con_error(&error));
    let actual = configuracion::leer(&raiz.join("configuracion.toml"))
        .unwrap_or_else(|error| salir_con_error(&error));

    let mut instalados: Vec<String> = actual
        .escritorio
        .instalados_efectivos()
        .into_iter()
        .map(str::to_string)
        .collect();

    if activo {
        if !instalados.iter().any(|instalado| instalado == nombre) {
            instalados.push(nombre.to_string());
        }
    } else {
        instalados.retain(|instalado| instalado != nombre);
    }

    instalados.sort_by_key(|instalado| orden_escritorio(instalado));

    match configuracion::cambiar_escritorios(&raiz.join("configuracion.toml"), &instalados) {
        Ok(true) => {
            println!(
                "✓ {nombre} quedó {} entre los escritorios disponibles.",
                if activo { "activo" } else { "apagado" }
            );
            println!("NixOS todavía no cambió.");
        }
        Ok(false) => println!("No cambié nada."),
        Err(error) => salir_con_error(&error),
    }
}

fn mostrar_teclado(raiz: &Path) {
    match configuracion::leer(&raiz.join("configuracion.toml")) {
        Ok(configuracion) => {
            println!(
                "Teclados: {}",
                configuracion.teclado.distribuciones.join(", ")
            );
            println!("Cambio: {}", configuracion.teclado.cambio);
        }
        Err(error) => salir_con_error(&error),
    }
}

fn cambiar_teclado(raiz: &Path, nombre: &str, valor: &str) {
    if !matches!(nombre, "españa" | "latinoamérica") {
        salir_con_error(&format!(
            "Todavía no conozco el teclado «{nombre}» en este primer catálogo."
        ));
    }

    let activo = leer_interruptor(valor).unwrap_or_else(|error| salir_con_error(&error));
    let actual = configuracion::leer(&raiz.join("configuracion.toml"))
        .unwrap_or_else(|error| salir_con_error(&error));
    let mut distribuciones = actual.teclado.distribuciones;

    if activo {
        if !distribuciones
            .iter()
            .any(|distribucion| distribucion == nombre)
        {
            distribuciones.push(nombre.to_string());
        }
    } else {
        distribuciones.retain(|distribucion| distribucion != nombre);
    }

    distribuciones.sort_by_key(|distribucion| if distribucion == "españa" { 0 } else { 1 });

    match configuracion::cambiar_teclado(&raiz.join("configuracion.toml"), &distribuciones) {
        Ok(true) => {
            println!(
                "✓ El teclado «{nombre}» quedó {}.",
                if activo { "activo" } else { "apagado" }
            );
            println!("NixOS todavía no cambió.");
        }
        Ok(false) => println!("No cambié nada."),
        Err(error) => salir_con_error(&error),
    }
}

fn mostrar_monitor(raiz: &Path) {
    match configuracion::leer(&raiz.join("configuracion.toml")) {
        Ok(configuracion) => println!(
            "Monitor: {} @ {} Hz",
            configuracion.monitor.resolucion, configuracion.monitor.hz
        ),
        Err(error) => salir_con_error(&error),
    }
}

fn cambiar_monitor(raiz: &Path, resolucion: &str, hz: &str) {
    let hz = hz.parse::<u32>().unwrap_or_else(|_| {
        salir_con_error("Los Hz tienen que ser un número entero mayor que cero.")
    });

    match configuracion::cambiar_monitor(&raiz.join("configuracion.toml"), resolucion, hz) {
        Ok(true) => {
            println!("✓ Monitor: {resolucion} @ {hz} Hz.");
            println!("NixOS todavía no cambió.");
        }
        Ok(false) => println!("El monitor ya tenía esos valores."),
        Err(error) => salir_con_error(&error),
    }
}

fn mostrar_almacenamiento(raiz: &Path) {
    let configuracion = configuracion::leer(&raiz.join("configuracion.toml"))
        .unwrap_or_else(|error| salir_con_error(&error));
    let unidades = almacenamiento::leer().unwrap_or_else(|error| salir_con_error(&error));

    println!("Almacenamiento:");

    if unidades.is_empty() && configuracion.almacenamiento.disponibles.is_empty() {
        println!("  No encontré discos adicionales.");
        return;
    }

    let mut vistas = Vec::new();

    for unidad in unidades {
        let conocida = configuracion::unidad_almacenamiento_conocida(&unidad.nombre);
        let elegida = configuracion
            .almacenamiento
            .disponibles
            .iter()
            .any(|nombre| nombre == &unidad.nombre);

        println!("  - {}", unidad.nombre);

        if conocida {
            println!(
                "    {} · {}",
                unidad.detalle,
                if elegida {
                    "disponible en Korunix · se monta al usarlo"
                } else {
                    "no disponible en Korunix"
                }
            );
        } else {
            println!(
                "    {} · detectado · todavía no administrado por Korunix",
                unidad.detalle
            );
        }

        vistas.push(unidad.nombre);
    }

    for nombre in &configuracion.almacenamiento.disponibles {
        if !vistas.iter().any(|vista| vista == nombre) {
            println!("  - {nombre}");
            println!("    no está conectado ahora · la elección se conserva");
        }
    }
}

fn cambiar_almacenamiento(raiz: &Path, nombre: &str, accion: &str) {
    if !configuracion::unidad_almacenamiento_conocida(nombre) {
        salir_con_error(&format!(
            "Puedo mostrar «{nombre}», pero todavía no puedo administrarlo de forma segura."
        ));
    }

    let activo = leer_interruptor(accion).unwrap_or_else(|error| salir_con_error(&error));
    let actual = configuracion::leer(&raiz.join("configuracion.toml"))
        .unwrap_or_else(|error| salir_con_error(&error));
    let mut disponibles = actual.almacenamiento.disponibles;

    if activo {
        if !disponibles.iter().any(|unidad| unidad == nombre) {
            disponibles.push(nombre.to_string());
        }
    } else {
        disponibles.retain(|unidad| unidad != nombre);
    }

    match configuracion::cambiar_almacenamiento(&raiz.join("configuracion.toml"), &disponibles) {
        Ok(true) => {
            println!(
                "✓ «{nombre}» quedó {} en la configuración.",
                if activo { "disponible" } else { "apagado" }
            );
            println!("NixOS todavía no cambió.");
        }
        Ok(false) => println!("No cambié nada."),
        Err(error) => salir_con_error(&error),
    }
}

fn mostrar_canal(raiz: &Path) {
    match configuracion::leer(&raiz.join("configuracion.toml")) {
        Ok(configuracion) => println!("Canal: {}", configuracion.canal),
        Err(error) => salir_con_error(&error),
    }
}

fn cambiar_canal(raiz: &Path, canal: &str) {
    match configuracion::cambiar_canal(&raiz.join("configuracion.toml"), canal) {
        Ok(true) => {
            println!("✓ El canal ahora es «{canal}» en configuracion.toml.");
            println!("NixOS todavía no cambió.");
        }
        Ok(false) => {
            println!("El canal ya era «{canal}».");
            println!("No cambié nada.");
        }
        Err(error) => salir_con_error(&error),
    }
}

fn leer_interruptor(valor: &str) -> Result<bool, String> {
    match valor {
        "activar" => Ok(true),
        "desactivar" => Ok(false),
        _ => Err(format!(
            "No entiendo «{valor}».\nUsa «activar» o «desactivar»."
        )),
    }
}

fn mostrar_apariencia(raiz: &Path) {
    match configuracion::leer(&raiz.join("configuracion.toml")) {
        Ok(configuracion) => println!(
            "Apariencia: {} · {}",
            configuracion.apariencia.estilo, configuracion.apariencia.modo
        ),
        Err(error) => salir_con_error(&error),
    }
}

fn cambiar_apariencia(raiz: &Path, estilo: &str, modo: &str) {
    match configuracion::cambiar_apariencia(&raiz.join("configuracion.toml"), estilo, modo) {
        Ok(true) => {
            println!("✓ Apariencia: {estilo} · {modo}.");
            println!("NixOS todavía no cambió.");
        }
        Ok(false) => {
            println!("Esa apariencia ya estaba elegida.");
            println!("No cambié nada.");
        }
        Err(error) => salir_con_error(&error),
    }
}

fn mostrar_bluetooth(raiz: &Path) {
    match configuracion::leer(&raiz.join("configuracion.toml")) {
        Ok(configuracion) => println!(
            "Bluetooth: {}",
            if configuracion.bluetooth.activo {
                "activo"
            } else {
                "apagado"
            }
        ),
        Err(error) => salir_con_error(&error),
    }
}

fn cambiar_bluetooth(raiz: &Path, valor: &str) {
    let activo = leer_interruptor(valor).unwrap_or_else(|error| salir_con_error(&error));

    match configuracion::cambiar_bluetooth(&raiz.join("configuracion.toml"), activo) {
        Ok(true) => {
            println!(
                "✓ Bluetooth quedó {}.",
                if activo { "activo" } else { "apagado" }
            );
            println!("NixOS todavía no cambió.");
        }
        Ok(false) => println!(
            "Bluetooth ya estaba {}.",
            if activo { "activo" } else { "apagado" }
        ),
        Err(error) => salir_con_error(&error),
    }
}

fn mostrar_sunshine(raiz: &Path) {
    match configuracion::leer(&raiz.join("configuracion.toml")) {
        Ok(configuracion) => println!(
            "Sunshine: {} · autoinicio {}",
            if configuracion.sunshine.activo {
                "activo"
            } else {
                "apagado"
            },
            if configuracion.sunshine.autoinicio {
                "sí"
            } else {
                "no"
            }
        ),
        Err(error) => salir_con_error(&error),
    }
}

fn cambiar_sunshine(raiz: &Path, valor: &str) {
    let activo = leer_interruptor(valor).unwrap_or_else(|error| salir_con_error(&error));
    let actual = configuracion::leer(&raiz.join("configuracion.toml"))
        .unwrap_or_else(|error| salir_con_error(&error));

    match configuracion::cambiar_sunshine(
        &raiz.join("configuracion.toml"),
        activo,
        actual.sunshine.autoinicio,
    ) {
        Ok(true) => {
            println!(
                "✓ Sunshine quedó {}.",
                if activo { "activo" } else { "apagado" }
            );
            println!("Su preferencia de autoinicio se conservó.");
            println!("NixOS todavía no cambió.");
        }
        Ok(false) => println!(
            "Sunshine ya estaba {}.",
            if activo { "activo" } else { "apagado" }
        ),
        Err(error) => salir_con_error(&error),
    }
}

fn cambiar_sunshine_autoinicio(raiz: &Path, valor: &str) {
    let autoinicio = leer_interruptor(valor).unwrap_or_else(|error| salir_con_error(&error));
    let actual = configuracion::leer(&raiz.join("configuracion.toml"))
        .unwrap_or_else(|error| salir_con_error(&error));

    match configuracion::cambiar_sunshine(
        &raiz.join("configuracion.toml"),
        actual.sunshine.activo,
        autoinicio,
    ) {
        Ok(true) => {
            println!(
                "✓ Autoinicio de Sunshine: {}.",
                if autoinicio { "activo" } else { "apagado" }
            );
            println!("NixOS todavía no cambió.");
        }
        Ok(false) => println!("El autoinicio de Sunshine ya estaba así."),
        Err(error) => salir_con_error(&error),
    }
}

fn mostrar_steam(raiz: &Path) {
    match configuracion::leer(&raiz.join("configuracion.toml")) {
        Ok(configuracion) => println!(
            "Steam: {} · Remote Play {} · servidor dedicado {}",
            if configuracion.steam.activo {
                "activo"
            } else {
                "apagado"
            },
            if configuracion.steam.remote_play {
                "sí"
            } else {
                "no"
            },
            if configuracion.steam.servidor_dedicado {
                "sí"
            } else {
                "no"
            }
        ),
        Err(error) => salir_con_error(&error),
    }
}

fn guardar_steam(
    raiz: &Path,
    activo: bool,
    remote_play: bool,
    servidor_dedicado: bool,
    mensaje: &str,
) {
    match configuracion::cambiar_steam(
        &raiz.join("configuracion.toml"),
        activo,
        remote_play,
        servidor_dedicado,
    ) {
        Ok(true) => {
            println!("✓ {mensaje}");
            println!("NixOS todavía no cambió.");
        }
        Ok(false) => println!("No cambié nada."),
        Err(error) => salir_con_error(&error),
    }
}

fn cambiar_steam(raiz: &Path, valor: &str) {
    let activo = leer_interruptor(valor).unwrap_or_else(|error| salir_con_error(&error));
    let actual = configuracion::leer(&raiz.join("configuracion.toml"))
        .unwrap_or_else(|error| salir_con_error(&error));

    guardar_steam(
        raiz,
        activo,
        actual.steam.remote_play,
        actual.steam.servidor_dedicado,
        &format!(
            "Steam quedó {} y conservó sus subopciones.",
            if activo { "activo" } else { "apagado" }
        ),
    );
}

fn cambiar_steam_remote_play(raiz: &Path, valor: &str) {
    let remote_play = leer_interruptor(valor).unwrap_or_else(|error| salir_con_error(&error));
    let actual = configuracion::leer(&raiz.join("configuracion.toml"))
        .unwrap_or_else(|error| salir_con_error(&error));

    guardar_steam(
        raiz,
        actual.steam.activo,
        remote_play,
        actual.steam.servidor_dedicado,
        &format!(
            "Steam Remote Play: {}.",
            if remote_play { "activo" } else { "apagado" }
        ),
    );
}

fn cambiar_steam_servidor(raiz: &Path, valor: &str) {
    let servidor = leer_interruptor(valor).unwrap_or_else(|error| salir_con_error(&error));
    let actual = configuracion::leer(&raiz.join("configuracion.toml"))
        .unwrap_or_else(|error| salir_con_error(&error));

    guardar_steam(
        raiz,
        actual.steam.activo,
        actual.steam.remote_play,
        servidor,
        &format!(
            "Servidor dedicado de Steam: {}.",
            if servidor { "activo" } else { "apagado" }
        ),
    );
}

fn mostrar_impresion(raiz: &Path) {
    match configuracion::leer(&raiz.join("configuracion.toml")) {
        Ok(configuracion) => println!(
            "Impresión: {}",
            if configuracion.impresion.activa {
                "activa"
            } else {
                "apagada"
            }
        ),
        Err(error) => salir_con_error(&error),
    }
}

fn cambiar_impresion(raiz: &Path, valor: &str) {
    let activa = leer_interruptor(valor).unwrap_or_else(|error| salir_con_error(&error));

    match configuracion::cambiar_impresion(&raiz.join("configuracion.toml"), activa) {
        Ok(true) => {
            println!(
                "✓ Impresión: {}.",
                if activa { "activa" } else { "apagada" }
            );
            println!("NixOS todavía no cambió.");
        }
        Ok(false) => println!("La impresión ya estaba así."),
        Err(error) => salir_con_error(&error),
    }
}

fn mostrar_virtualizacion(raiz: &Path) {
    match configuracion::leer(&raiz.join("configuracion.toml")) {
        Ok(configuracion) => println!(
            "Virtualización: {}",
            if configuracion.virtualizacion.activa {
                "activa"
            } else {
                "apagada"
            }
        ),
        Err(error) => salir_con_error(&error),
    }
}

fn cambiar_virtualizacion(raiz: &Path, valor: &str) {
    let activa = leer_interruptor(valor).unwrap_or_else(|error| salir_con_error(&error));

    match configuracion::cambiar_virtualizacion(&raiz.join("configuracion.toml"), activa) {
        Ok(true) => {
            println!(
                "✓ Virtualización: {}.",
                if activa { "activa" } else { "apagada" }
            );
            println!("NixOS todavía no cambió.");
        }
        Ok(false) => println!("La virtualización ya estaba así."),
        Err(error) => salir_con_error(&error),
    }
}

fn listar_aplicaciones(raiz: &Path) {
    match configuracion::leer(&raiz.join("configuracion.toml")) {
        Ok(configuracion) => {
            if configuracion.aplicaciones.instaladas.is_empty() {
                println!("No hay aplicaciones en la lista.");
                return;
            }

            println!("Aplicaciones:");

            for aplicacion in configuracion.aplicaciones.instaladas {
                println!("  - {aplicacion}");
            }
        }
        Err(error) => salir_con_error(&error),
    }
}

fn agregar_aplicacion(raiz: &Path, nombre: &str) {
    match configuracion::agregar_aplicacion(&raiz.join("configuracion.toml"), nombre) {
        Ok(true) => {
            println!("✓ Agregué «{nombre}» a configuracion.toml.");
            println!("NixOS todavía no cambió.");
        }
        Ok(false) => {
            println!("«{nombre}» ya estaba en la lista.");
            println!("No cambié nada.");
        }
        Err(error) => salir_con_error(&error),
    }
}

fn quitar_aplicacion(raiz: &Path, nombre: &str) {
    match configuracion::quitar_aplicacion(&raiz.join("configuracion.toml"), nombre) {
        Ok(true) => {
            println!("✓ Quité «{nombre}» de configuracion.toml.");
            println!("NixOS todavía no cambió.");
        }
        Ok(false) => {
            println!("«{nombre}» no estaba en la lista.");
            println!("No cambié nada.");
        }
        Err(error) => salir_con_error(&error),
    }
}

fn main() {
    let argumentos: Vec<String> = env::args().skip(1).collect();

    if argumentos
        .first()
        .is_some_and(|comando| comando == "__aplicar-raiz")
    {
        process::exit(aplicar::ejecutar_como_root(&argumentos[1..]));
    }

    if argumentos
        .first()
        .is_some_and(|comando| comando == "__rollback-raiz")
    {
        process::exit(rollback::ejecutar_como_root(&argumentos[1..]));
    }

    if matches!(
        argumentos.as_slice(),
        [grupo, accion] if grupo == "sesion" && accion == "preparar"
    ) {
        match sistema::preparar_sesion() {
            Ok(preparada) => {
                println!("✓ La sesión quedó preparada.");
                println!("Noctalia: {}", preparada.configuracion_noctalia.display());
                println!("Capturas: {}", preparada.capturas.display());
            }
            Err(error) => salir_con_error(&error),
        }

        return;
    }

    let raiz = match raiz_korunix() {
        Ok(raiz) => raiz,
        Err(error) => salir_con_error(&error),
    };

    match argumentos.as_slice() {
        [] => validar(&raiz),
        [comando] if comando == "validar" => validar(&raiz),
        [comando] if comando == "plan" => mostrar_plan(&raiz),
        [comando] if comando == "preview" => preparar_preview(&raiz),
        [comando] if comando == "aplicar" => {
            if let Err(error) = aplicar::ejecutar(&raiz) {
                eprintln!("{error}");
                process::exit(1);
            }
        }
        [comando] if comando == "rollback" => {
            if let Err(error) = rollback::ejecutar(&raiz) {
                eprintln!("{error}");
                process::exit(1);
            }
        }
        [comando] if comando == "nombre" => mostrar_nombre(&raiz),
        [comando, nombre] if comando == "nombre" => cambiar_nombre(&raiz, nombre),
        [comando] if comando == "personas" => mostrar_personas(&raiz),
        [comando] if comando == "escritorio" => mostrar_escritorio(&raiz),
        [comando, escritorio] if comando == "escritorio" => cambiar_escritorio(&raiz, escritorio),
        [comando] if comando == "escritorios" => mostrar_escritorios(&raiz),
        [comando, escritorio, valor] if comando == "escritorios" => {
            cambiar_escritorio_instalado(&raiz, escritorio, valor)
        }
        [comando] if comando == "teclado" => mostrar_teclado(&raiz),
        [comando, teclado, valor] if comando == "teclado" => cambiar_teclado(&raiz, teclado, valor),
        [comando] if comando == "monitor" => mostrar_monitor(&raiz),
        [comando, resolucion, hz] if comando == "monitor" => cambiar_monitor(&raiz, resolucion, hz),
        [comando] if comando == "almacenamiento" => mostrar_almacenamiento(&raiz),
        [comando, nombre, accion] if comando == "almacenamiento" => {
            cambiar_almacenamiento(&raiz, nombre, accion)
        }
        [comando] if comando == "canal" => mostrar_canal(&raiz),
        [comando, canal] if comando == "canal" => cambiar_canal(&raiz, canal),
        [comando] if comando == "apariencia" => mostrar_apariencia(&raiz),
        [comando, estilo, modo] if comando == "apariencia" => {
            cambiar_apariencia(&raiz, estilo, modo)
        }
        [comando] if comando == "bluetooth" => mostrar_bluetooth(&raiz),
        [comando, valor] if comando == "bluetooth" => cambiar_bluetooth(&raiz, valor),
        [comando] if comando == "sunshine" => mostrar_sunshine(&raiz),
        [comando, valor] if comando == "sunshine" => cambiar_sunshine(&raiz, valor),
        [comando, opcion, valor] if comando == "sunshine" && opcion == "autoinicio" => {
            cambiar_sunshine_autoinicio(&raiz, valor)
        }
        [comando] if comando == "steam" => mostrar_steam(&raiz),
        [comando, valor] if comando == "steam" => cambiar_steam(&raiz, valor),
        [comando, opcion, valor] if comando == "steam" && opcion == "remote-play" => {
            cambiar_steam_remote_play(&raiz, valor)
        }
        [comando, opcion, valor] if comando == "steam" && opcion == "servidor-dedicado" => {
            cambiar_steam_servidor(&raiz, valor)
        }
        [comando] if comando == "impresion" => mostrar_impresion(&raiz),
        [comando, valor] if comando == "impresion" => cambiar_impresion(&raiz, valor),
        [comando] if comando == "virtualizacion" => mostrar_virtualizacion(&raiz),
        [comando, valor] if comando == "virtualizacion" => cambiar_virtualizacion(&raiz, valor),
        [comando] if comando == "aplicaciones" => listar_aplicaciones(&raiz),
        [grupo, accion, nombre] if grupo == "aplicaciones" && accion == "agregar" => {
            agregar_aplicacion(&raiz, nombre)
        }
        [grupo, accion, nombre] if grupo == "aplicaciones" && accion == "quitar" => {
            quitar_aplicacion(&raiz, nombre)
        }
        _ => {
            eprintln!("No entendí esa orden.");
            ayuda();
            process::exit(2);
        }
    }
}

mod configuracion;
mod preview;
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
    eprintln!("  korunix nombre");
    eprintln!("  korunix nombre <nuevo>");
    eprintln!("  korunix personas");
    eprintln!("  korunix escritorio");
    eprintln!("  korunix escritorio <niri|hyprland|cinnamon|plasma>");
    eprintln!("  korunix canal");
    eprintln!("  korunix canal <estable|inestable>");
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
            println!("  - {} → {}", unidad.nombre, unidad.ruta);
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
        [comando] if comando == "nombre" => mostrar_nombre(&raiz),
        [comando, nombre] if comando == "nombre" => cambiar_nombre(&raiz, nombre),
        [comando] if comando == "personas" => mostrar_personas(&raiz),
        [comando] if comando == "escritorio" => mostrar_escritorio(&raiz),
        [comando, escritorio] if comando == "escritorio" => cambiar_escritorio(&raiz, escritorio),
        [comando] if comando == "canal" => mostrar_canal(&raiz),
        [comando, canal] if comando == "canal" => cambiar_canal(&raiz, canal),
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

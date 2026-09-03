mod configuracion;

use std::env;
use std::path::Path;
use std::process;

const RUTA_CONFIGURACION: &str = "configuracion.toml";

fn ayuda() {
    eprintln!("Por ahora puedes usar:");
    eprintln!("  korunix validar");
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

fn validar() {
    match configuracion::leer(Path::new(RUTA_CONFIGURACION)) {
        Ok(configuracion) => {
            println!("✓ La configuración está bien.");
            println!("Canal: {}", configuracion.canal);
            println!(
                "Aplicaciones elegidas: {}",
                configuracion.aplicaciones.instaladas.len()
            );
        }
        Err(error) => salir_con_error(&error),
    }
}

fn mostrar_canal() {
    match configuracion::leer(Path::new(RUTA_CONFIGURACION)) {
        Ok(configuracion) => println!("Canal: {}", configuracion.canal),
        Err(error) => salir_con_error(&error),
    }
}

fn cambiar_canal(canal: &str) {
    match configuracion::cambiar_canal(Path::new(RUTA_CONFIGURACION), canal) {
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

fn listar_aplicaciones() {
    match configuracion::leer(Path::new(RUTA_CONFIGURACION)) {
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

fn agregar_aplicacion(nombre: &str) {
    match configuracion::agregar_aplicacion(Path::new(RUTA_CONFIGURACION), nombre) {
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

fn quitar_aplicacion(nombre: &str) {
    match configuracion::quitar_aplicacion(Path::new(RUTA_CONFIGURACION), nombre) {
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

    match argumentos.as_slice() {
        [] => validar(),
        [comando] if comando == "validar" => validar(),
        [comando] if comando == "canal" => mostrar_canal(),
        [comando, canal] if comando == "canal" => cambiar_canal(canal),
        [comando] if comando == "aplicaciones" => listar_aplicaciones(),
        [grupo, accion, nombre] if grupo == "aplicaciones" && accion == "agregar" => {
            agregar_aplicacion(nombre)
        }
        [grupo, accion, nombre] if grupo == "aplicaciones" && accion == "quitar" => {
            quitar_aplicacion(nombre)
        }
        _ => {
            eprintln!("No entendí esa orden.");
            ayuda();
            process::exit(2);
        }
    }
}

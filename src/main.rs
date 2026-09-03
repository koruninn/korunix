mod configuracion;

use std::env;
use std::path::Path;
use std::process;

fn main() {
    let comando = env::args().nth(1).unwrap_or_else(|| "validar".to_string());

    if comando != "validar" {
        eprintln!("No conozco la orden «{comando}».");
        eprintln!("Por ahora puedes usar: korunix validar");
        process::exit(2);
    }

    match configuracion::leer(Path::new("configuracion.toml")) {
        Ok(configuracion) => {
            println!("✓ La configuración está bien.");
            println!("Canal: {}", configuracion.canal);
            println!(
                "Aplicaciones elegidas: {}",
                configuracion.aplicaciones.instaladas.len()
            );
        }
        Err(error) => {
            eprintln!("{error}");
            eprintln!();
            eprintln!("No se cambió nada.");
            process::exit(1);
        }
    }
}

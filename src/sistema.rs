use crate::configuracion::Configuracion;
use serde::Deserialize;
use std::env;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Deserialize)]
pub struct Plan {
    pub canal: String,
    pub revision: String,
    pub aplicaciones: Vec<Aplicacion>,
}

#[derive(Debug, Deserialize)]
pub struct Aplicacion {
    pub elegida: String,
    pub nombre: String,
    pub version: String,
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
    if plan.canal != configuracion.canal {
        return Err(
            "El canal que resolvió Nix no coincide con configuracion.toml. No voy a usar ese plan."
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

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::configuracion::Aplicaciones;

    fn configuracion() -> Configuracion {
        Configuracion {
            canal: "inestable".to_string(),
            aplicaciones: Aplicaciones {
                instaladas: vec!["firefox".to_string(), "karere".to_string()],
            },
        }
    }

    fn plan() -> Plan {
        Plan {
            canal: "inestable".to_string(),
            revision: "abc123".to_string(),
            aplicaciones: vec![
                Aplicacion {
                    elegida: "firefox".to_string(),
                    nombre: "firefox".to_string(),
                    version: "1".to_string(),
                },
                Aplicacion {
                    elegida: "karere".to_string(),
                    nombre: "karere".to_string(),
                    version: "2".to_string(),
                },
            ],
        }
    }

    #[test]
    fn entiende_el_plan_de_nix() {
        let texto = br#"{
          "canal": "inestable",
          "revision": "abc123",
          "aplicaciones": [
            {"elegida": "firefox", "nombre": "firefox", "version": "1"}
          ]
        }"#;

        let plan = leer_plan(texto).expect("el plan debería entenderse");

        assert_eq!(plan.canal, "inestable");
        assert_eq!(plan.aplicaciones[0].elegida, "firefox");
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
    fn rechaza_un_plan_con_otras_aplicaciones() {
        let configuracion = configuracion();
        let mut plan = plan();
        plan.aplicaciones.pop();

        let error = comprobar_plan(&configuracion, &plan)
            .expect_err("una lista distinta debería rechazarse");

        assert!(error.contains("no coinciden"));
    }
}

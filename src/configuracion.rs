use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Configuracion {
    #[serde(default = "canal_por_defecto")]
    pub canal: String,

    #[serde(default)]
    pub aplicaciones: Aplicaciones,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Aplicaciones {
    #[serde(default)]
    pub instaladas: Vec<String>,
}

fn canal_por_defecto() -> String {
    "estable".to_string()
}

pub fn leer(ruta: &Path) -> Result<Configuracion, String> {
    let texto = fs::read_to_string(ruta)
        .map_err(|error| format!("No pude leer {}.\nDetalle: {error}", ruta.display()))?;

    let configuracion: Configuracion = toml::from_str(&texto).map_err(|error| {
        format!(
            "No pude entender {}.\nHay una opción que no conozco o un problema de formato.\nDetalle: {error}",
            ruta.display()
        )
    })?;

    revisar(&configuracion)?;

    Ok(configuracion)
}

pub fn revisar(configuracion: &Configuracion) -> Result<(), String> {
    if configuracion.canal != "estable" && configuracion.canal != "inestable" {
        return Err(format!(
            "No conozco el canal «{}».\nUsa «estable» o «inestable».",
            configuracion.canal
        ));
    }

    let mut vistas = HashSet::new();

    for aplicacion in &configuracion.aplicaciones.instaladas {
        if aplicacion.trim().is_empty() {
            return Err(
                "Hay una aplicación sin nombre en [aplicaciones].\nBorra esa línea vacía o escribe un nombre."
                    .to_string(),
            );
        }

        if aplicacion.trim() != aplicacion {
            return Err(format!(
                "La aplicación «{aplicacion}» tiene espacios de más al comienzo o al final.\nEscríbela como «{}».",
                aplicacion.trim()
            ));
        }

        if !vistas.insert(aplicacion.as_str()) {
            return Err(format!(
                "La aplicación «{aplicacion}» aparece más de una vez.\nDéjala una sola vez."
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn leer_texto(texto: &str) -> Result<Configuracion, String> {
        let configuracion: Configuracion = toml::from_str(texto)
            .map_err(|error| format!("No pude entender la configuración.\nDetalle: {error}"))?;

        revisar(&configuracion)?;

        Ok(configuracion)
    }

    #[test]
    fn una_configuracion_humana_valida_funciona() {
        let configuracion = leer_texto(
            r#"
canal = "inestable"

[aplicaciones]
instaladas = ["firefox", "karere", "blender"]
"#,
        )
        .expect("la configuración debería ser válida");

        assert_eq!(configuracion.canal, "inestable");
        assert_eq!(configuracion.aplicaciones.instaladas.len(), 3);
    }

    #[test]
    fn si_no_se_escribe_canal_se_usa_estable() {
        let configuracion = leer_texto(
            r#"
[aplicaciones]
instaladas = ["firefox"]
"#,
        )
        .expect("la configuración debería usar el canal estable");

        assert_eq!(configuracion.canal, "estable");
    }

    #[test]
    fn un_canal_desconocido_se_explica() {
        let error = leer_texto(
            r#"
canal = "rapidito"
"#,
        )
        .expect_err("el canal debería rechazarse");

        assert!(error.contains("No conozco el canal «rapidito»"));
        assert!(error.contains("estable"));
        assert!(error.contains("inestable"));
    }

    #[test]
    fn una_opcion_inventada_no_se_ignora() {
        let error = leer_texto(
            r#"
canal = "estable"
canalito = "inestable"
"#,
        )
        .expect_err("una opción inventada debería rechazarse");

        assert!(error.contains("canalito"));
    }

    #[test]
    fn una_aplicacion_vacia_se_rechaza() {
        let error = leer_texto(
            r#"
[aplicaciones]
instaladas = ["firefox", ""]
"#,
        )
        .expect_err("un nombre vacío debería rechazarse");

        assert!(error.contains("aplicación sin nombre"));
    }

    #[test]
    fn una_aplicacion_repetida_se_rechaza() {
        let error = leer_texto(
            r#"
[aplicaciones]
instaladas = ["firefox", "firefox"]
"#,
        )
        .expect_err("una aplicación repetida debería rechazarse");

        assert!(error.contains("aparece más de una vez"));
    }

    #[test]
    fn los_espacios_accidentales_se_explican() {
        let error = leer_texto(
            r#"
[aplicaciones]
instaladas = [" firefox"]
"#,
        )
        .expect_err("los espacios accidentales deberían rechazarse");

        assert!(error.contains("espacios de más"));
        assert!(error.contains("«firefox»"));
    }
}

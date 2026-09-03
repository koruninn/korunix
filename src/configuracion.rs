use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};
use toml_edit::{value, Array, DocumentMut, Item, Table, Value};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Configuracion {
    #[serde(default = "nombre_por_defecto")]
    pub nombre: String,

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

fn nombre_por_defecto() -> String {
    "nixos".to_string()
}

fn canal_por_defecto() -> String {
    "estable".to_string()
}

fn revisar_nombre(nombre: &str) -> Result<(), String> {
    let largo_valido = !nombre.is_empty() && nombre.len() <= 63;
    let caracteres_validos = nombre
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
    let bordes_validos = !nombre.starts_with('-') && !nombre.ends_with('-');

    if !largo_valido || !caracteres_validos || !bordes_validos {
        return Err(format!(
            "El nombre «{nombre}» no sirve para la computadora.\nUsa letras minúsculas, números y guiones, sin empezar ni terminar con guion."
        ));
    }

    Ok(())
}

fn revisar_canal(canal: &str) -> Result<(), String> {
    if canal != "estable" && canal != "inestable" {
        return Err(format!(
            "No conozco el canal «{canal}».\nPon «estable» o «inestable»."
        ));
    }

    Ok(())
}

fn entender(texto: &str, origen: &str) -> Result<Configuracion, String> {
    let configuracion: Configuracion = toml::from_str(texto).map_err(|error| {
        format!(
            "No pude entender {origen}.\nHay una opción que no conozco o un problema de formato.\nDetalle: {error}"
        )
    })?;

    revisar(&configuracion)?;

    Ok(configuracion)
}

pub fn leer(ruta: &Path) -> Result<Configuracion, String> {
    let texto = fs::read_to_string(ruta)
        .map_err(|error| format!("No pude leer {}.\nDetalle: {error}", ruta.display()))?;

    entender(&texto, &ruta.display().to_string())
}

fn revisar_nombre_aplicacion(nombre: &str) -> Result<(), String> {
    if nombre.trim().is_empty() {
        return Err("La aplicación necesita un nombre.".to_string());
    }

    if nombre.trim() != nombre {
        return Err(format!(
            "«{nombre}» tiene espacios de más al comienzo o al final.\nPon «{}».",
            nombre.trim()
        ));
    }

    Ok(())
}

pub fn revisar(configuracion: &Configuracion) -> Result<(), String> {
    revisar_nombre(&configuracion.nombre)?;
    revisar_canal(&configuracion.canal)?;

    let mut vistas = HashSet::new();

    for aplicacion in &configuracion.aplicaciones.instaladas {
        revisar_nombre_aplicacion(aplicacion)?;

        if !vistas.insert(aplicacion.as_str()) {
            return Err(format!(
                "«{aplicacion}» aparece más de una vez.\nDéjala una sola vez."
            ));
        }
    }

    Ok(())
}

fn lista_aplicaciones(documento: &mut DocumentMut) -> Result<&mut Array, String> {
    let aplicaciones = documento
        .as_table_mut()
        .entry("aplicaciones")
        .or_insert(Item::Table(Table::new()))
        .as_table_mut()
        .ok_or_else(|| {
            "No pude usar [aplicaciones].\nEsa parte de configuracion.toml tiene que ser una sección."
                .to_string()
        })?;

    aplicaciones
        .entry("instaladas")
        .or_insert(Item::Value(Value::Array(Array::new())))
        .as_array_mut()
        .ok_or_else(|| {
            "No pude editar la lista de aplicaciones.\nEn configuracion.toml, «instaladas» tiene que ser una lista."
                .to_string()
        })
}

fn agregar_en_texto(texto: &str, nombre: &str) -> Result<Option<String>, String> {
    revisar_nombre_aplicacion(nombre)?;
    entender(texto, "la configuración")?;

    let mut documento = texto.parse::<DocumentMut>().map_err(|error| {
        format!("No pude preparar configuracion.toml para editarlo.\nDetalle: {error}")
    })?;

    let lista = lista_aplicaciones(&mut documento)?;

    if lista.iter().any(|valor| valor.as_str() == Some(nombre)) {
        return Ok(None);
    }

    lista.push(nombre);

    let nuevo = documento.to_string();
    entender(&nuevo, "la configuración después del cambio")?;

    Ok(Some(nuevo))
}

fn quitar_en_texto(texto: &str, nombre: &str) -> Result<Option<String>, String> {
    revisar_nombre_aplicacion(nombre)?;
    entender(texto, "la configuración")?;

    let mut documento = texto.parse::<DocumentMut>().map_err(|error| {
        format!("No pude preparar configuracion.toml para editarlo.\nDetalle: {error}")
    })?;

    let lista = lista_aplicaciones(&mut documento)?;

    let Some(posicion) = lista
        .iter()
        .position(|valor| valor.as_str() == Some(nombre))
    else {
        return Ok(None);
    };

    lista.remove(posicion);

    let nuevo = documento.to_string();
    entender(&nuevo, "la configuración después del cambio")?;

    Ok(Some(nuevo))
}

fn cambiar_nombre_en_texto(texto: &str, nombre: &str) -> Result<Option<String>, String> {
    revisar_nombre(nombre)?;
    let actual = entender(texto, "la configuración")?;

    if actual.nombre == nombre {
        return Ok(None);
    }

    let mut documento = texto.parse::<DocumentMut>().map_err(|error| {
        format!("No pude preparar configuracion.toml para editarlo.\nDetalle: {error}")
    })?;

    documento["nombre"] = value(nombre);

    let nuevo = documento.to_string();
    entender(&nuevo, "la configuración después del cambio")?;

    Ok(Some(nuevo))
}

fn cambiar_canal_en_texto(texto: &str, canal: &str) -> Result<Option<String>, String> {
    revisar_canal(canal)?;
    let actual = entender(texto, "la configuración")?;

    if actual.canal == canal {
        return Ok(None);
    }

    let mut documento = texto.parse::<DocumentMut>().map_err(|error| {
        format!("No pude preparar configuracion.toml para editarlo.\nDetalle: {error}")
    })?;

    documento["canal"] = value(canal);

    let nuevo = documento.to_string();
    entender(&nuevo, "la configuración después del cambio")?;

    Ok(Some(nuevo))
}

fn guardar(ruta: &Path, texto: &str) -> Result<(), String> {
    let carpeta = ruta.parent().unwrap_or_else(|| Path::new("."));
    let nombre = ruta
        .file_name()
        .and_then(|nombre| nombre.to_str())
        .unwrap_or("configuracion.toml");

    let momento = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("No pude preparar el guardado.\nDetalle: {error}"))?
        .as_nanos();

    let temporal = carpeta.join(format!(".{nombre}.korunix-{}-{momento}.tmp", process::id()));

    let resultado = (|| {
        fs::write(&temporal, texto)
            .map_err(|error| format!("No pude guardar el cambio.\nDetalle: {error}"))?;

        if let Ok(datos) = fs::metadata(ruta) {
            fs::set_permissions(&temporal, datos.permissions())
                .map_err(|error| format!("No pude conservar los permisos.\nDetalle: {error}"))?;
        }

        fs::rename(&temporal, ruta)
            .map_err(|error| format!("No pude terminar de guardar el cambio.\nDetalle: {error}"))?;

        Ok::<(), String>(())
    })();

    if resultado.is_err() {
        let _ = fs::remove_file(&temporal);
    }

    resultado
}

pub fn agregar_aplicacion(ruta: &Path, nombre: &str) -> Result<bool, String> {
    let texto = fs::read_to_string(ruta)
        .map_err(|error| format!("No pude leer {}.\nDetalle: {error}", ruta.display()))?;

    let Some(nuevo) = agregar_en_texto(&texto, nombre)? else {
        return Ok(false);
    };

    guardar(ruta, &nuevo)?;

    Ok(true)
}

pub fn quitar_aplicacion(ruta: &Path, nombre: &str) -> Result<bool, String> {
    let texto = fs::read_to_string(ruta)
        .map_err(|error| format!("No pude leer {}.\nDetalle: {error}", ruta.display()))?;

    let Some(nuevo) = quitar_en_texto(&texto, nombre)? else {
        return Ok(false);
    };

    guardar(ruta, &nuevo)?;

    Ok(true)
}

pub fn cambiar_nombre(ruta: &Path, nombre: &str) -> Result<bool, String> {
    let texto = fs::read_to_string(ruta)
        .map_err(|error| format!("No pude leer {}.\nDetalle: {error}", ruta.display()))?;

    let Some(nuevo) = cambiar_nombre_en_texto(&texto, nombre)? else {
        return Ok(false);
    };

    guardar(ruta, &nuevo)?;

    Ok(true)
}

pub fn cambiar_canal(ruta: &Path, canal: &str) -> Result<bool, String> {
    let texto = fs::read_to_string(ruta)
        .map_err(|error| format!("No pude leer {}.\nDetalle: {error}", ruta.display()))?;

    let Some(nuevo) = cambiar_canal_en_texto(&texto, canal)? else {
        return Ok(false);
    };

    guardar(ruta, &nuevo)?;

    Ok(true)
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn leer_texto(texto: &str) -> Result<Configuracion, String> {
        entender(texto, "la configuración de prueba")
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

        assert!(error.contains("necesita un nombre"));
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

    #[test]
    fn agregar_conserva_los_comentarios() {
        let original = r#"# Este comentario tiene que quedarse.
canal = "estable"

[aplicaciones]
# Este también.
instaladas = [
  "firefox",
]
"#;

        let nuevo = agregar_en_texto(original, "karere")
            .expect("debería poder agregar Karere")
            .expect("debería existir un cambio");

        assert!(nuevo.contains("# Este comentario tiene que quedarse."));
        assert!(nuevo.contains("# Este también."));
        assert!(nuevo.contains("\"firefox\""));
        assert!(nuevo.contains("\"karere\""));
    }

    #[test]
    fn agregar_lo_que_ya_existe_no_duplica() {
        let original = r#"
[aplicaciones]
instaladas = ["firefox", "karere"]
"#;

        let nuevo = agregar_en_texto(original, "karere").expect("la operación debería ser válida");

        assert!(nuevo.is_none());
    }

    #[test]
    fn quitar_conserva_lo_demas() {
        let original = r#"# Este comentario tiene que quedarse.
canal = "inestable"

[aplicaciones]
instaladas = ["firefox", "karere", "blender"]
"#;

        let nuevo = quitar_en_texto(original, "karere")
            .expect("debería poder quitar Karere")
            .expect("debería existir un cambio");

        assert!(nuevo.contains("# Este comentario tiene que quedarse."));
        assert!(nuevo.contains("\"firefox\""));
        assert!(!nuevo.contains("\"karere\""));
        assert!(nuevo.contains("\"blender\""));
        assert!(nuevo.contains("canal = \"inestable\""));
    }

    #[test]
    fn quitar_lo_que_no_existe_no_cambia_nada() {
        let original = r#"
[aplicaciones]
instaladas = ["firefox"]
"#;

        let nuevo = quitar_en_texto(original, "karere").expect("la operación debería ser válida");

        assert!(nuevo.is_none());
    }

    #[test]
    fn agregar_funciona_si_no_hay_lista() {
        let original = r#"canal = "estable"
"#;

        let nuevo = agregar_en_texto(original, "firefox")
            .expect("debería poder crear la lista")
            .expect("debería existir un cambio");

        let configuracion = leer_texto(&nuevo).expect("el resultado debería seguir siendo válido");

        assert_eq!(
            configuracion.aplicaciones.instaladas,
            vec!["firefox".to_string()]
        );
    }

    #[test]
    fn agregar_funciona_si_existe_la_seccion_pero_no_la_lista() {
        let original = r#"canal = "estable"

[aplicaciones]
"#;

        let nuevo = agregar_en_texto(original, "firefox")
            .expect("debería poder crear la lista")
            .expect("debería existir un cambio");

        let configuracion = leer_texto(&nuevo).expect("el resultado debería seguir siendo válido");

        assert_eq!(
            configuracion.aplicaciones.instaladas,
            vec!["firefox".to_string()]
        );
    }

    #[test]
    fn cambiar_canal_conserva_el_resto() {
        let original = r#"# Este comentario tiene que quedarse.
canal = "inestable"

[aplicaciones]
# Este también.
instaladas = ["firefox", "karere"]
"#;

        let nuevo = cambiar_canal_en_texto(original, "estable")
            .expect("debería poder cambiar el canal")
            .expect("debería existir un cambio");

        assert!(nuevo.contains("# Este comentario tiene que quedarse."));
        assert!(nuevo.contains("# Este también."));
        assert!(nuevo.contains("canal = \"estable\""));
        assert!(nuevo.contains("\"firefox\""));
        assert!(nuevo.contains("\"karere\""));
    }

    #[test]
    fn poner_el_mismo_canal_no_cambia_nada() {
        let original = r#"canal = "estable"

[aplicaciones]
instaladas = ["firefox"]
"#;

        let nuevo =
            cambiar_canal_en_texto(original, "estable").expect("la operación debería ser válida");

        assert!(nuevo.is_none());
    }

    #[test]
    fn un_canal_inventado_no_se_guarda() {
        let original = r#"canal = "estable"
"#;

        let error = cambiar_canal_en_texto(original, "rapidito")
            .expect_err("un canal inventado debería rechazarse");

        assert!(error.contains("No conozco el canal «rapidito»"));
    }

    #[test]
    fn si_no_se_escribe_nombre_se_usa_nixos() {
        let configuracion = leer_texto(
            r#"
canal = "estable"
"#,
        )
        .expect("la configuración debería usar un nombre seguro");

        assert_eq!(configuracion.nombre, "nixos");
    }

    #[test]
    fn un_nombre_raro_se_rechaza() {
        let error = leer_texto(
            r#"
nombre = "Mi PC"
canal = "estable"
"#,
        )
        .expect_err("el nombre debería rechazarse");

        assert!(error.contains("letras minúsculas"));
    }

    #[test]
    fn cambiar_nombre_conserva_el_resto() {
        let original = r#"# Así aparece esta computadora en la red.
nombre = "korunix"

# Este comentario también tiene que quedarse.
canal = "inestable"

[aplicaciones]
instaladas = ["firefox", "karere"]
"#;

        let nuevo = cambiar_nombre_en_texto(original, "korunix-sala")
            .expect("debería poder cambiar el nombre")
            .expect("debería existir un cambio");

        assert!(nuevo.contains("# Así aparece esta computadora en la red."));
        assert!(nuevo.contains("# Este comentario también tiene que quedarse."));
        assert!(nuevo.contains("nombre = \"korunix-sala\""));
        assert!(nuevo.contains("canal = \"inestable\""));
        assert!(nuevo.contains("\"firefox\""));
        assert!(nuevo.contains("\"karere\""));
    }

    #[test]
    fn poner_el_mismo_nombre_no_cambia_nada() {
        let original = r#"nombre = "korunix"
canal = "estable"
"#;

        let nuevo =
            cambiar_nombre_en_texto(original, "korunix").expect("la operación debería ser válida");

        assert!(nuevo.is_none());
    }
}

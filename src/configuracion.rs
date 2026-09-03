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
    pub personas: Vec<Persona>,

    #[serde(default)]
    pub escritorio: Escritorio,

    #[serde(default)]
    pub idioma: Idioma,

    #[serde(default)]
    pub teclado: Teclado,

    #[serde(default)]
    pub monitor: Monitor,

    #[serde(default)]
    pub almacenamiento: Almacenamiento,

    #[serde(default)]
    pub sunshine: Sunshine,

    #[serde(default)]
    pub impresion: Impresion,

    #[serde(default)]
    pub virtualizacion: Virtualizacion,

    #[serde(default)]
    pub aplicaciones: Aplicaciones,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Aplicaciones {
    #[serde(default)]
    pub instaladas: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Persona {
    pub cuenta: String,
    pub nombre: String,

    #[serde(default)]
    pub administrador: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Escritorio {
    #[serde(default = "escritorio_por_defecto")]
    pub principal: String,

    #[serde(default)]
    pub instalados: Vec<String>,
}

impl Escritorio {
    pub fn instalados_efectivos(&self) -> Vec<&str> {
        if self.instalados.is_empty() {
            vec![self.principal.as_str()]
        } else {
            self.instalados.iter().map(String::as_str).collect()
        }
    }
}

impl Default for Escritorio {
    fn default() -> Self {
        Self {
            principal: escritorio_por_defecto(),
            instalados: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Idioma {
    #[serde(default = "idioma_por_defecto")]
    pub sistema: String,
    #[serde(default = "region_por_defecto")]
    pub region: String,
}

impl Default for Idioma {
    fn default() -> Self {
        Self {
            sistema: idioma_por_defecto(),
            region: region_por_defecto(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Teclado {
    #[serde(default = "teclados_por_defecto")]
    pub distribuciones: Vec<String>,
    #[serde(default = "cambio_por_defecto")]
    pub cambio: String,
}

impl Default for Teclado {
    fn default() -> Self {
        Self {
            distribuciones: teclados_por_defecto(),
            cambio: cambio_por_defecto(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Monitor {
    #[serde(default = "resolucion_por_defecto")]
    pub resolucion: String,
    #[serde(default = "hz_por_defecto")]
    pub hz: u32,
}

impl Default for Monitor {
    fn default() -> Self {
        Self {
            resolucion: resolucion_por_defecto(),
            hz: hz_por_defecto(),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Almacenamiento {
    #[serde(default)]
    pub disponibles: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sunshine {
    #[serde(default)]
    pub activo: bool,
    #[serde(default)]
    pub autoinicio: bool,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Impresion {
    #[serde(default)]
    pub activa: bool,
    #[serde(default)]
    pub controlador: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Virtualizacion {
    #[serde(default)]
    pub activa: bool,
}

fn idioma_por_defecto() -> String {
    "español".to_string()
}

fn region_por_defecto() -> String {
    "Perú".to_string()
}

fn teclados_por_defecto() -> Vec<String> {
    vec!["españa".to_string(), "latinoamérica".to_string()]
}

fn cambio_por_defecto() -> String {
    "alt+shift".to_string()
}

fn resolucion_por_defecto() -> String {
    "1920x1080".to_string()
}

fn hz_por_defecto() -> u32 {
    120
}

fn escritorio_por_defecto() -> String {
    "niri".to_string()
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

fn entender_completa(texto: &str, origen: &str) -> Result<Configuracion, String> {
    let configuracion = entender(texto, origen)?;

    if configuracion.personas.is_empty() {
        return Err(
            "No hay ninguna cuenta local en configuracion.toml.\nAñade al menos un bloque [[personas]]."
                .to_string(),
        );
    }

    Ok(configuracion)
}

pub fn leer(ruta: &Path) -> Result<Configuracion, String> {
    let texto = fs::read_to_string(ruta)
        .map_err(|error| format!("No pude leer {}.\nDetalle: {error}", ruta.display()))?;

    entender_completa(&texto, &ruta.display().to_string())
}

fn revisar_cuenta(cuenta: &str) -> Result<(), String> {
    let mut caracteres = cuenta.bytes();
    let primero_valido = caracteres
        .next()
        .is_some_and(|byte| byte.is_ascii_lowercase() || byte == b'_');
    let resto_valido = caracteres.all(|byte| {
        byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'_'
    });

    if cuenta == "root" || cuenta.len() > 31 || !primero_valido || !resto_valido {
        return Err(format!(
            "La cuenta «{cuenta}» no sirve como usuario local.\nUsa minúsculas, números, guiones o _, y empieza con una letra o _."
        ));
    }

    Ok(())
}

fn revisar_persona(persona: &Persona) -> Result<(), String> {
    revisar_cuenta(&persona.cuenta)?;

    if persona.nombre.trim().is_empty() {
        return Err(format!(
            "La cuenta «{}» necesita un nombre visible.",
            persona.cuenta
        ));
    }

    if persona.nombre.trim() != persona.nombre {
        return Err(format!(
            "El nombre visible de «{}» tiene espacios de más al comienzo o al final.",
            persona.cuenta
        ));
    }

    Ok(())
}

fn revisar_escritorio(escritorio: &str) -> Result<(), String> {
    if !matches!(escritorio, "niri" | "hyprland" | "cinnamon" | "plasma") {
        let sugerencia = if escritorio == "niry" {
            "\n¿Querías decir «niri»?"
        } else {
            ""
        };

        return Err(format!(
            "No conozco el escritorio «{escritorio}».\nPon «niri», «hyprland», «cinnamon» o «plasma».{sugerencia}"
        ));
    }

    Ok(())
}

fn revisar_escritorios(escritorio: &Escritorio) -> Result<(), String> {
    revisar_escritorio(&escritorio.principal)?;

    let instalados = escritorio.instalados_efectivos();
    let mut vistos = HashSet::new();

    for instalado in &instalados {
        revisar_escritorio(instalado)?;

        if !vistos.insert(*instalado) {
            return Err(format!(
                "El escritorio «{instalado}» aparece más de una vez en «instalados»."
            ));
        }
    }

    if !instalados.contains(&escritorio.principal.as_str()) {
        return Err(format!(
            "El escritorio principal «{}» también tiene que aparecer en «instalados».",
            escritorio.principal
        ));
    }

    Ok(())
}

fn revisar_almacenamiento(almacenamiento: &Almacenamiento) -> Result<(), String> {
    let mut vistas = HashSet::new();

    for unidad in &almacenamiento.disponibles {
        if unidad != "datos" {
            return Err(format!(
                "Todavía no conozco la unidad «{unidad}» en este equipo."
            ));
        }

        if !vistas.insert(unidad.as_str()) {
            return Err(format!("La unidad «{unidad}» aparece más de una vez."));
        }
    }

    Ok(())
}

fn revisar_impresion(impresion: &Impresion) -> Result<(), String> {
    if let Some(controlador) = impresion.controlador.as_deref() {
        if controlador != "epson-201207w" {
            return Err(format!(
                "Todavía no conozco el controlador de impresión «{controlador}»."
            ));
        }
    }

    Ok(())
}

fn revisar_idioma(idioma: &Idioma) -> Result<(), String> {
    if idioma.sistema != "español" {
        return Err(format!(
            "Todavía no conozco el idioma «{}» en esta reimplementación.",
            idioma.sistema
        ));
    }

    if idioma.region != "Perú" {
        return Err(format!(
            "Todavía no conozco la región «{}» en esta reimplementación.",
            idioma.region
        ));
    }

    Ok(())
}

fn revisar_teclado(teclado: &Teclado) -> Result<(), String> {
    if teclado.distribuciones.is_empty() {
        return Err("Elige al menos una distribución de teclado.".to_string());
    }

    let mut vistas = HashSet::new();

    for distribucion in &teclado.distribuciones {
        if !matches!(distribucion.as_str(), "españa" | "latinoamérica") {
            return Err(format!(
                "Todavía no conozco el teclado «{distribucion}» en esta reimplementación."
            ));
        }

        if !vistas.insert(distribucion.as_str()) {
            return Err(format!(
                "El teclado «{distribucion}» aparece más de una vez."
            ));
        }
    }

    if teclado.cambio != "alt+shift" {
        return Err(format!(
            "No conozco «{}» para cambiar de teclado. Usa «alt+shift».",
            teclado.cambio
        ));
    }

    Ok(())
}

fn revisar_monitor(monitor: &Monitor) -> Result<(), String> {
    let Some((ancho, alto)) = monitor.resolucion.split_once('x') else {
        return Err(format!(
            "No entiendo la resolución «{}». Usa algo como «1920x1080».",
            monitor.resolucion
        ));
    };

    let ancho = ancho.parse::<u32>().ok();
    let alto = alto.parse::<u32>().ok();

    if ancho.unwrap_or(0) == 0 || alto.unwrap_or(0) == 0 {
        return Err(format!(
            "No entiendo la resolución «{}». Usa algo como «1920x1080».",
            monitor.resolucion
        ));
    }

    if monitor.hz == 0 {
        return Err("Los Hz del monitor tienen que ser mayores que cero.".to_string());
    }

    Ok(())
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
    revisar_escritorios(&configuracion.escritorio)?;
    revisar_idioma(&configuracion.idioma)?;
    revisar_teclado(&configuracion.teclado)?;
    revisar_monitor(&configuracion.monitor)?;
    revisar_almacenamiento(&configuracion.almacenamiento)?;
    revisar_impresion(&configuracion.impresion)?;

    let mut cuentas = HashSet::new();

    for persona in &configuracion.personas {
        revisar_persona(persona)?;

        if !cuentas.insert(persona.cuenta.as_str()) {
            return Err(format!(
                "La cuenta «{}» aparece más de una vez.\nDéjala una sola vez.",
                persona.cuenta
            ));
        }
    }

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

fn cambiar_escritorio_en_texto(texto: &str, escritorio: &str) -> Result<Option<String>, String> {
    revisar_escritorio(escritorio)?;
    let actual = entender(texto, "la configuración")?;

    if actual.escritorio.principal == escritorio {
        return Ok(None);
    }

    let mut documento = texto.parse::<DocumentMut>().map_err(|error| {
        format!("No pude preparar configuracion.toml para editarlo.\nDetalle: {error}")
    })?;

    let tabla = documento
        .as_table_mut()
        .entry("escritorio")
        .or_insert(Item::Table(Table::new()))
        .as_table_mut()
        .ok_or_else(|| {
            "No pude usar [escritorio].\nEsa parte de configuracion.toml tiene que ser una sección."
                .to_string()
        })?;

    tabla["principal"] = value(escritorio);

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

    entender_completa(&texto, &ruta.display().to_string())?;

    let Some(nuevo) = agregar_en_texto(&texto, nombre)? else {
        return Ok(false);
    };

    guardar(ruta, &nuevo)?;

    Ok(true)
}

pub fn quitar_aplicacion(ruta: &Path, nombre: &str) -> Result<bool, String> {
    let texto = fs::read_to_string(ruta)
        .map_err(|error| format!("No pude leer {}.\nDetalle: {error}", ruta.display()))?;

    entender_completa(&texto, &ruta.display().to_string())?;

    let Some(nuevo) = quitar_en_texto(&texto, nombre)? else {
        return Ok(false);
    };

    guardar(ruta, &nuevo)?;

    Ok(true)
}

pub fn cambiar_nombre(ruta: &Path, nombre: &str) -> Result<bool, String> {
    let texto = fs::read_to_string(ruta)
        .map_err(|error| format!("No pude leer {}.\nDetalle: {error}", ruta.display()))?;

    entender_completa(&texto, &ruta.display().to_string())?;

    let Some(nuevo) = cambiar_nombre_en_texto(&texto, nombre)? else {
        return Ok(false);
    };

    guardar(ruta, &nuevo)?;

    Ok(true)
}

pub fn cambiar_escritorio(ruta: &Path, escritorio: &str) -> Result<bool, String> {
    let texto = fs::read_to_string(ruta)
        .map_err(|error| format!("No pude leer {}.\nDetalle: {error}", ruta.display()))?;

    entender_completa(&texto, &ruta.display().to_string())?;

    let Some(nuevo) = cambiar_escritorio_en_texto(&texto, escritorio)? else {
        return Ok(false);
    };

    guardar(ruta, &nuevo)?;

    Ok(true)
}

pub fn cambiar_canal(ruta: &Path, canal: &str) -> Result<bool, String> {
    let texto = fs::read_to_string(ruta)
        .map_err(|error| format!("No pude leer {}.\nDetalle: {error}", ruta.display()))?;

    entender_completa(&texto, &ruta.display().to_string())?;

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
    #[test]
    fn una_configuracion_completa_necesita_una_persona() {
        let temporal = std::env::temp_dir().join(format!(
            "korunix-sin-persona-{}-{}.toml",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("el reloj debería funcionar")
                .as_nanos()
        ));

        fs::write(
            &temporal,
            r#"nombre = "korunix"
canal = "estable"
"#,
        )
        .expect("debería poder preparar la prueba");

        let error = leer(&temporal).expect_err("una configuración completa necesita una persona");
        let _ = fs::remove_file(&temporal);

        assert!(error.contains("[[personas]]"));
    }

    #[test]
    fn una_edicion_real_no_guarda_si_falta_la_persona() {
        let temporal = std::env::temp_dir().join(format!(
            "korunix-edicion-sin-persona-{}-{}.toml",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("el reloj debería funcionar")
                .as_nanos()
        ));

        let original = r#"nombre = "korunix"
canal = "estable"
"#;

        fs::write(&temporal, original).expect("debería poder preparar la prueba");

        let error = cambiar_canal(&temporal, "inestable")
            .expect_err("no debería guardar sobre una configuración incompleta");

        let despues = fs::read_to_string(&temporal).expect("debería poder releer la prueba");
        let _ = fs::remove_file(&temporal);

        assert!(error.contains("[[personas]]"));
        assert_eq!(despues, original);
    }

    #[test]
    fn una_cuenta_repetida_se_rechaza() {
        let error = leer_texto(
            r#"
[[personas]]
cuenta = "koru"
nombre = "André"

[[personas]]
cuenta = "koru"
nombre = "Otra persona"
"#,
        )
        .expect_err("una cuenta repetida debería rechazarse");

        assert!(error.contains("aparece más de una vez"));
    }

    #[test]
    fn niry_sugiere_niri() {
        let error = leer_texto(
            r#"
canal = "estable"

[escritorio]
principal = "niry"
"#,
        )
        .expect_err("un escritorio inventado debería rechazarse");

        assert!(error.contains("¿Querías decir «niri»?"));
    }

    #[test]
    fn cambiar_escritorio_conserva_lo_demas() {
        let original = r#"# El nombre tiene que quedarse.
nombre = "korunix"
canal = "inestable"

[[personas]]
cuenta = "koru"
nombre = "André"
administrador = true

[escritorio]
# Este comentario también.
principal = "niri"

[aplicaciones]
instaladas = ["firefox", "karere"]
"#;

        let nuevo = cambiar_escritorio_en_texto(original, "plasma")
            .expect("debería poder cambiar el escritorio")
            .expect("debería existir un cambio");

        assert!(nuevo.contains("# El nombre tiene que quedarse."));
        assert!(nuevo.contains("# Este comentario también."));
        assert!(nuevo.contains("principal = \"plasma\""));
        assert!(nuevo.contains("cuenta = \"koru\""));
        assert!(nuevo.contains("\"karere\""));
    }

    #[test]
    fn poner_el_mismo_escritorio_no_cambia_nada() {
        let original = r#"[[personas]]
cuenta = "koru"
nombre = "André"

[escritorio]
principal = "niri"
"#;

        let nuevo =
            cambiar_escritorio_en_texto(original, "niri").expect("la operación debería ser válida");

        assert!(nuevo.is_none());
    }
    #[test]
    fn un_teclado_repetido_se_rechaza() {
        let error = leer_texto(
            r#"
[teclado]
distribuciones = ["españa", "españa"]
cambio = "alt+shift"
"#,
        )
        .expect_err("un teclado repetido debería rechazarse");

        assert!(error.contains("más de una vez"));
    }

    #[test]
    fn una_resolucion_rara_se_explica() {
        let error = leer_texto(
            r#"
[monitor]
resolucion = "grande"
hz = 120
"#,
        )
        .expect_err("una resolución rara debería rechazarse");

        assert!(error.contains("1920x1080"));
    }

    #[test]
    fn idioma_y_region_se_escriben_con_nombres_humanos() {
        let configuracion = leer_texto(
            r#"
[idioma]
sistema = "español"
region = "Perú"
"#,
        )
        .expect("la configuración debería ser válida");

        assert_eq!(configuracion.idioma.sistema, "español");
        assert_eq!(configuracion.idioma.region, "Perú");
    }

    #[test]
    fn el_cambio_de_teclado_no_expone_xkb() {
        let configuracion = leer_texto(
            r#"
[teclado]
distribuciones = ["españa", "latinoamérica"]
cambio = "alt+shift"
"#,
        )
        .expect("la configuración debería ser válida");

        assert_eq!(configuracion.teclado.cambio, "alt+shift");
    }
    #[test]
    fn varios_escritorios_se_expresan_una_sola_vez() {
        let configuracion = leer_texto(
            r#"
[escritorio]
principal = "niri"
instalados = ["niri", "hyprland", "plasma", "cinnamon"]
"#,
        )
        .expect("los cuatro escritorios deberían ser válidos");

        assert_eq!(configuracion.escritorio.instalados.len(), 4);
    }

    #[test]
    fn el_principal_tiene_que_estar_instalado() {
        let error = leer_texto(
            r#"
[escritorio]
principal = "niri"
instalados = ["plasma"]
"#,
        )
        .expect_err("el principal no puede quedar fuera");

        assert!(error.contains("principal"));
        assert!(error.contains("instalados"));
    }

    #[test]
    fn un_escritorio_instalado_no_se_repite() {
        let error = leer_texto(
            r#"
[escritorio]
principal = "niri"
instalados = ["niri", "niri"]
"#,
        )
        .expect_err("un escritorio repetido debería rechazarse");

        assert!(error.contains("más de una vez"));
    }

    #[test]
    fn la_unidad_datos_se_elige_sin_escribir_uuid() {
        let configuracion = leer_texto(
            r#"
[almacenamiento]
disponibles = ["datos"]
"#,
        )
        .expect("la unidad conocida debería ser válida");

        assert_eq!(configuracion.almacenamiento.disponibles, vec!["datos"]);
    }

    #[test]
    fn una_unidad_inventada_se_rechaza() {
        let error = leer_texto(
            r#"
[almacenamiento]
disponibles = ["disco-magico"]
"#,
        )
        .expect_err("una unidad desconocida debería rechazarse");

        assert!(error.contains("no conozco"));
        assert!(error.contains("disco-magico"));
    }

    #[test]
    fn apagar_sunshine_conserva_su_autoinicio() {
        let configuracion = leer_texto(
            r#"
[sunshine]
activo = false
autoinicio = true
"#,
        )
        .expect("la preferencia interna debería conservarse");

        assert!(!configuracion.sunshine.activo);
        assert!(configuracion.sunshine.autoinicio);
    }

    #[test]
    fn impresion_y_virtualizacion_son_decisiones_humanas() {
        let configuracion = leer_texto(
            r#"
[impresion]
activa = true
controlador = "epson-201207w"

[virtualizacion]
activa = true
"#,
        )
        .expect("las decisiones deberían ser válidas");

        assert!(configuracion.impresion.activa);
        assert_eq!(
            configuracion.impresion.controlador.as_deref(),
            Some("epson-201207w")
        );
        assert!(configuracion.virtualizacion.activa);
    }
}

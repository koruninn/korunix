use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path};
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
    pub apariencia: Apariencia,

    #[serde(default)]
    pub idioma: Idioma,

    #[serde(default)]
    pub teclado: Teclado,

    #[serde(default)]
    pub monitor: Monitor,

    #[serde(default)]
    pub almacenamiento: Almacenamiento,

    #[serde(default)]
    pub bluetooth: Bluetooth,

    #[serde(default)]
    pub sunshine: Sunshine,

    #[serde(default)]
    pub steam: Steam,

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

    #[serde(default)]
    pub avatar: Option<String>,

    #[serde(default)]
    pub clave_github: Option<String>,
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
pub struct Apariencia {
    #[serde(default = "estilo_por_defecto")]
    pub estilo: String,

    #[serde(default = "modo_por_defecto")]
    pub modo: String,
}

impl Default for Apariencia {
    fn default() -> Self {
        Self {
            estilo: estilo_por_defecto(),
            modo: modo_por_defecto(),
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
pub struct Bluetooth {
    #[serde(default)]
    pub activo: bool,
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
pub struct Steam {
    #[serde(default)]
    pub activo: bool,
    #[serde(default)]
    pub remote_play: bool,
    #[serde(default)]
    pub servidor_dedicado: bool,
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

fn estilo_por_defecto() -> String {
    "predeterminado".to_string()
}

fn modo_por_defecto() -> String {
    "automatico".to_string()
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

fn revisar_ruta_humana(valor: &str, nombre: &str) -> Result<(), String> {
    if valor.is_empty() || valor.trim() != valor {
        return Err(format!(
            "«{nombre}» necesita una ruta relativa sin espacios de más."
        ));
    }

    let ruta = Path::new(valor);

    if ruta.is_absolute()
        || ruta
            .components()
            .any(|parte| matches!(parte, Component::ParentDir | Component::RootDir))
    {
        return Err(format!(
            "«{nombre}» tiene que apuntar dentro de la carpeta esperada, sin usar / al comienzo ni «..»."
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

    if let Some(avatar) = persona.avatar.as_deref() {
        revisar_ruta_humana(avatar, "avatar")?;
    }

    if let Some(clave) = persona.clave_github.as_deref() {
        revisar_ruta_humana(clave, "clave_github")?;
    }

    Ok(())
}

fn revisar_apariencia(apariencia: &Apariencia) -> Result<(), String> {
    if !matches!(
        apariencia.estilo.as_str(),
        "predeterminado" | "dinamico" | "everforest"
    ) {
        return Err(format!(
            "No conozco el estilo «{}». Pon «predeterminado», «dinamico» o «everforest».",
            apariencia.estilo
        ));
    }

    if !matches!(apariencia.modo.as_str(), "claro" | "oscuro" | "automatico") {
        return Err(format!(
            "No conozco el modo «{}». Pon «claro», «oscuro» o «automatico».",
            apariencia.modo
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
        if unidad.trim().is_empty() || unidad.trim() != unidad || unidad.len() > 160 {
            return Err(format!(
                "El nombre de almacenamiento «{unidad}» no es una identificación humana válida."
            ));
        }

        if unidad.chars().any(char::is_control) {
            return Err(format!(
                "El nombre de almacenamiento «{unidad}» contiene caracteres que no puedo guardar con seguridad."
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
    revisar_apariencia(&configuracion.apariencia)?;
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

        if aplicacion == "steam" {
            return Err(
                "Steam se configura en [steam], porque tiene opciones propias. No lo repitas en [aplicaciones]."
                    .to_string(),
            );
        }

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

fn seccion<'a>(documento: &'a mut DocumentMut, nombre: &str) -> Result<&'a mut Table, String> {
    documento
        .as_table_mut()
        .entry(nombre)
        .or_insert(Item::Table(Table::new()))
        .as_table_mut()
        .ok_or_else(|| {
            format!(
                "No pude usar [{nombre}].\nEsa parte de configuracion.toml tiene que ser una sección."
            )
        })
}

fn array_humano(valores: &[String]) -> Array {
    let mut array = Array::new();

    for valor in valores {
        array.push(valor.as_str());
    }

    array
}

fn cambiar_escritorios_en_texto(
    texto: &str,
    instalados: &[String],
) -> Result<Option<String>, String> {
    let actual = entender(texto, "la configuración")?;
    let nueva = Escritorio {
        principal: actual.escritorio.principal.clone(),
        instalados: instalados.to_vec(),
    };
    revisar_escritorios(&nueva)?;

    let actuales: Vec<String> = actual
        .escritorio
        .instalados_efectivos()
        .into_iter()
        .map(str::to_string)
        .collect();

    if actuales == instalados {
        return Ok(None);
    }

    let mut documento = texto.parse::<DocumentMut>().map_err(|error| {
        format!("No pude preparar configuracion.toml para editarlo.\nDetalle: {error}")
    })?;
    seccion(&mut documento, "escritorio")?["instalados"] = value(array_humano(instalados));

    let nuevo = documento.to_string();
    entender(&nuevo, "la configuración después del cambio")?;

    Ok(Some(nuevo))
}

fn cambiar_teclado_en_texto(
    texto: &str,
    distribuciones: &[String],
) -> Result<Option<String>, String> {
    let actual = entender(texto, "la configuración")?;
    let nuevo_teclado = Teclado {
        distribuciones: distribuciones.to_vec(),
        cambio: actual.teclado.cambio.clone(),
    };
    revisar_teclado(&nuevo_teclado)?;

    if actual.teclado.distribuciones == distribuciones {
        return Ok(None);
    }

    let mut documento = texto.parse::<DocumentMut>().map_err(|error| {
        format!("No pude preparar configuracion.toml para editarlo.\nDetalle: {error}")
    })?;
    seccion(&mut documento, "teclado")?["distribuciones"] = value(array_humano(distribuciones));

    let nuevo = documento.to_string();
    entender(&nuevo, "la configuración después del cambio")?;

    Ok(Some(nuevo))
}

fn cambiar_monitor_en_texto(
    texto: &str,
    resolucion: &str,
    hz: u32,
) -> Result<Option<String>, String> {
    let actual = entender(texto, "la configuración")?;
    let nuevo_monitor = Monitor {
        resolucion: resolucion.to_string(),
        hz,
    };
    revisar_monitor(&nuevo_monitor)?;

    if actual.monitor.resolucion == resolucion && actual.monitor.hz == hz {
        return Ok(None);
    }

    let mut documento = texto.parse::<DocumentMut>().map_err(|error| {
        format!("No pude preparar configuracion.toml para editarlo.\nDetalle: {error}")
    })?;
    let monitor = seccion(&mut documento, "monitor")?;
    monitor["resolucion"] = value(resolucion);
    monitor["hz"] = value(i64::from(hz));

    let nuevo = documento.to_string();
    entender(&nuevo, "la configuración después del cambio")?;

    Ok(Some(nuevo))
}

fn cambiar_almacenamiento_en_texto(
    texto: &str,
    disponibles: &[String],
) -> Result<Option<String>, String> {
    let actual = entender(texto, "la configuración")?;
    let nuevo_almacenamiento = Almacenamiento {
        disponibles: disponibles.to_vec(),
    };
    revisar_almacenamiento(&nuevo_almacenamiento)?;

    if actual.almacenamiento.disponibles == disponibles {
        return Ok(None);
    }

    let mut documento = texto.parse::<DocumentMut>().map_err(|error| {
        format!("No pude preparar configuracion.toml para editarlo.\nDetalle: {error}")
    })?;
    seccion(&mut documento, "almacenamiento")?["disponibles"] = value(array_humano(disponibles));

    let nuevo = documento.to_string();
    entender(&nuevo, "la configuración después del cambio")?;

    Ok(Some(nuevo))
}

fn cambiar_apariencia_en_texto(
    texto: &str,
    estilo: &str,
    modo: &str,
) -> Result<Option<String>, String> {
    let actual = entender(texto, "la configuración")?;
    let nueva = Apariencia {
        estilo: estilo.to_string(),
        modo: modo.to_string(),
    };
    revisar_apariencia(&nueva)?;

    if actual.apariencia.estilo == estilo && actual.apariencia.modo == modo {
        return Ok(None);
    }

    let mut documento = texto.parse::<DocumentMut>().map_err(|error| {
        format!("No pude preparar configuracion.toml para editarlo.\nDetalle: {error}")
    })?;
    let apariencia = seccion(&mut documento, "apariencia")?;
    apariencia["estilo"] = value(estilo);
    apariencia["modo"] = value(modo);

    let nuevo = documento.to_string();
    entender(&nuevo, "la configuración después del cambio")?;

    Ok(Some(nuevo))
}

fn cambiar_bluetooth_en_texto(texto: &str, activo: bool) -> Result<Option<String>, String> {
    let actual = entender(texto, "la configuración")?;

    if actual.bluetooth.activo == activo {
        return Ok(None);
    }

    let mut documento = texto.parse::<DocumentMut>().map_err(|error| {
        format!("No pude preparar configuracion.toml para editarlo.\nDetalle: {error}")
    })?;
    seccion(&mut documento, "bluetooth")?["activo"] = value(activo);

    let nuevo = documento.to_string();
    entender(&nuevo, "la configuración después del cambio")?;

    Ok(Some(nuevo))
}

fn cambiar_sunshine_en_texto(
    texto: &str,
    activo: bool,
    autoinicio: bool,
) -> Result<Option<String>, String> {
    let actual = entender(texto, "la configuración")?;

    if actual.sunshine.activo == activo && actual.sunshine.autoinicio == autoinicio {
        return Ok(None);
    }

    let mut documento = texto.parse::<DocumentMut>().map_err(|error| {
        format!("No pude preparar configuracion.toml para editarlo.\nDetalle: {error}")
    })?;
    let sunshine = seccion(&mut documento, "sunshine")?;
    sunshine["activo"] = value(activo);
    sunshine["autoinicio"] = value(autoinicio);

    let nuevo = documento.to_string();
    entender(&nuevo, "la configuración después del cambio")?;

    Ok(Some(nuevo))
}

fn cambiar_steam_en_texto(
    texto: &str,
    activo: bool,
    remote_play: bool,
    servidor_dedicado: bool,
) -> Result<Option<String>, String> {
    let actual = entender(texto, "la configuración")?;

    if actual.steam.activo == activo
        && actual.steam.remote_play == remote_play
        && actual.steam.servidor_dedicado == servidor_dedicado
    {
        return Ok(None);
    }

    let mut documento = texto.parse::<DocumentMut>().map_err(|error| {
        format!("No pude preparar configuracion.toml para editarlo.\nDetalle: {error}")
    })?;
    let steam = seccion(&mut documento, "steam")?;
    steam["activo"] = value(activo);
    steam["remote_play"] = value(remote_play);
    steam["servidor_dedicado"] = value(servidor_dedicado);

    let nuevo = documento.to_string();
    entender(&nuevo, "la configuración después del cambio")?;

    Ok(Some(nuevo))
}

fn cambiar_impresion_en_texto(texto: &str, activa: bool) -> Result<Option<String>, String> {
    let actual = entender(texto, "la configuración")?;

    if actual.impresion.activa == activa {
        return Ok(None);
    }

    let mut documento = texto.parse::<DocumentMut>().map_err(|error| {
        format!("No pude preparar configuracion.toml para editarlo.\nDetalle: {error}")
    })?;
    seccion(&mut documento, "impresion")?["activa"] = value(activa);

    let nuevo = documento.to_string();
    entender(&nuevo, "la configuración después del cambio")?;

    Ok(Some(nuevo))
}

fn cambiar_virtualizacion_en_texto(texto: &str, activa: bool) -> Result<Option<String>, String> {
    let actual = entender(texto, "la configuración")?;

    if actual.virtualizacion.activa == activa {
        return Ok(None);
    }

    let mut documento = texto.parse::<DocumentMut>().map_err(|error| {
        format!("No pude preparar configuracion.toml para editarlo.\nDetalle: {error}")
    })?;
    seccion(&mut documento, "virtualizacion")?["activa"] = value(activa);

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

pub fn cambiar_escritorios(ruta: &Path, instalados: &[String]) -> Result<bool, String> {
    let texto = fs::read_to_string(ruta)
        .map_err(|error| format!("No pude leer {}.\nDetalle: {error}", ruta.display()))?;

    entender_completa(&texto, &ruta.display().to_string())?;

    let Some(nuevo) = cambiar_escritorios_en_texto(&texto, instalados)? else {
        return Ok(false);
    };

    guardar(ruta, &nuevo)?;

    Ok(true)
}

pub fn cambiar_teclado(ruta: &Path, distribuciones: &[String]) -> Result<bool, String> {
    let texto = fs::read_to_string(ruta)
        .map_err(|error| format!("No pude leer {}.\nDetalle: {error}", ruta.display()))?;

    entender_completa(&texto, &ruta.display().to_string())?;

    let Some(nuevo) = cambiar_teclado_en_texto(&texto, distribuciones)? else {
        return Ok(false);
    };

    guardar(ruta, &nuevo)?;

    Ok(true)
}

pub fn cambiar_monitor(ruta: &Path, resolucion: &str, hz: u32) -> Result<bool, String> {
    let texto = fs::read_to_string(ruta)
        .map_err(|error| format!("No pude leer {}.\nDetalle: {error}", ruta.display()))?;

    entender_completa(&texto, &ruta.display().to_string())?;

    let Some(nuevo) = cambiar_monitor_en_texto(&texto, resolucion, hz)? else {
        return Ok(false);
    };

    guardar(ruta, &nuevo)?;

    Ok(true)
}

pub fn cambiar_almacenamiento(ruta: &Path, disponibles: &[String]) -> Result<bool, String> {
    let texto = fs::read_to_string(ruta)
        .map_err(|error| format!("No pude leer {}.\nDetalle: {error}", ruta.display()))?;

    entender_completa(&texto, &ruta.display().to_string())?;

    let Some(nuevo) = cambiar_almacenamiento_en_texto(&texto, disponibles)? else {
        return Ok(false);
    };

    guardar(ruta, &nuevo)?;

    Ok(true)
}

pub fn cambiar_apariencia(ruta: &Path, estilo: &str, modo: &str) -> Result<bool, String> {
    let texto = fs::read_to_string(ruta)
        .map_err(|error| format!("No pude leer {}.\nDetalle: {error}", ruta.display()))?;

    entender_completa(&texto, &ruta.display().to_string())?;

    let Some(nuevo) = cambiar_apariencia_en_texto(&texto, estilo, modo)? else {
        return Ok(false);
    };

    guardar(ruta, &nuevo)?;

    Ok(true)
}

pub fn cambiar_bluetooth(ruta: &Path, activo: bool) -> Result<bool, String> {
    let texto = fs::read_to_string(ruta)
        .map_err(|error| format!("No pude leer {}.\nDetalle: {error}", ruta.display()))?;

    entender_completa(&texto, &ruta.display().to_string())?;

    let Some(nuevo) = cambiar_bluetooth_en_texto(&texto, activo)? else {
        return Ok(false);
    };

    guardar(ruta, &nuevo)?;

    Ok(true)
}

pub fn cambiar_sunshine(ruta: &Path, activo: bool, autoinicio: bool) -> Result<bool, String> {
    let texto = fs::read_to_string(ruta)
        .map_err(|error| format!("No pude leer {}.\nDetalle: {error}", ruta.display()))?;

    entender_completa(&texto, &ruta.display().to_string())?;

    let Some(nuevo) = cambiar_sunshine_en_texto(&texto, activo, autoinicio)? else {
        return Ok(false);
    };

    guardar(ruta, &nuevo)?;

    Ok(true)
}

pub fn cambiar_steam(
    ruta: &Path,
    activo: bool,
    remote_play: bool,
    servidor_dedicado: bool,
) -> Result<bool, String> {
    let texto = fs::read_to_string(ruta)
        .map_err(|error| format!("No pude leer {}.\nDetalle: {error}", ruta.display()))?;

    entender_completa(&texto, &ruta.display().to_string())?;

    let Some(nuevo) = cambiar_steam_en_texto(&texto, activo, remote_play, servidor_dedicado)?
    else {
        return Ok(false);
    };

    guardar(ruta, &nuevo)?;

    Ok(true)
}

pub fn cambiar_impresion(ruta: &Path, activa: bool) -> Result<bool, String> {
    let texto = fs::read_to_string(ruta)
        .map_err(|error| format!("No pude leer {}.\nDetalle: {error}", ruta.display()))?;

    entender_completa(&texto, &ruta.display().to_string())?;

    let Some(nuevo) = cambiar_impresion_en_texto(&texto, activa)? else {
        return Ok(false);
    };

    guardar(ruta, &nuevo)?;

    Ok(true)
}

pub fn cambiar_virtualizacion(ruta: &Path, activa: bool) -> Result<bool, String> {
    let texto = fs::read_to_string(ruta)
        .map_err(|error| format!("No pude leer {}.\nDetalle: {error}", ruta.display()))?;

    entender_completa(&texto, &ruta.display().to_string())?;

    let Some(nuevo) = cambiar_virtualizacion_en_texto(&texto, activa)? else {
        return Ok(false);
    };

    guardar(ruta, &nuevo)?;

    Ok(true)
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
    fn la_unidad_reconocible_se_elige_sin_escribir_uuid() {
        let configuracion = leer_texto(
            r#"
[almacenamiento]
disponibles = ["ST3500413AS · 500 GB"]
"#,
        )
        .expect("la unidad conocida debería ser válida");

        assert_eq!(
            configuracion.almacenamiento.disponibles,
            vec!["ST3500413AS · 500 GB"]
        );
    }

    #[test]
    fn una_identidad_humana_no_necesita_estar_hardcodeada() {
        let configuracion = leer_texto(
            r#"
[almacenamiento]
disponibles = ["Disco externo · 2 TB"]
"#,
        )
        .expect("el TOML humano no debería depender de una lista de modelos en Rust");

        assert_eq!(
            configuracion.almacenamiento.disponibles,
            vec!["Disco externo · 2 TB"]
        );
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
    #[test]
    fn apagar_steam_conserva_sus_preferencias() {
        let configuracion = leer_texto(
            r#"
[steam]
activo = false
remote_play = true
servidor_dedicado = true
"#,
        )
        .expect("las preferencias deberían conservarse");

        assert!(!configuracion.steam.activo);
        assert!(configuracion.steam.remote_play);
        assert!(configuracion.steam.servidor_dedicado);
    }

    #[test]
    fn steam_no_se_duplica_en_aplicaciones() {
        let error = leer_texto(
            r#"
[steam]
activo = true
remote_play = true
servidor_dedicado = true

[aplicaciones]
instaladas = ["steam"]
"#,
        )
        .expect_err("Steam debería expresarse una sola vez");

        assert!(error.contains("[steam]"));
        assert!(error.contains("No lo repitas"));
    }
    #[test]
    fn apariencia_humana_se_valida() {
        let configuracion = leer_texto(
            r#"
[apariencia]
estilo = "dinamico"
modo = "automatico"
"#,
        )
        .expect("la apariencia debería ser válida");

        assert_eq!(configuracion.apariencia.estilo, "dinamico");
        assert_eq!(configuracion.apariencia.modo, "automatico");
    }

    #[test]
    fn apariencia_inventada_se_rechaza() {
        let error = leer_texto(
            r#"
[apariencia]
estilo = "superbonito"
modo = "automatico"
"#,
        )
        .expect_err("un estilo inventado debería rechazarse");

        assert!(error.contains("superbonito"));
    }

    #[test]
    fn el_escritorio_principal_no_se_puede_quitar_de_instalados() {
        let original = r#"[escritorio]
principal = "niri"
instalados = ["niri", "hyprland"]
"#;

        let error = cambiar_escritorios_en_texto(original, &["hyprland".to_string()])
            .expect_err("no debería quitar el escritorio principal");

        assert!(error.contains("principal"));
        assert!(error.contains("instalados"));
    }

    #[test]
    fn cambiar_teclados_conserva_el_metodo_de_cambio() {
        let original = r#"# Este comentario sigue.
[teclado]
distribuciones = ["españa", "latinoamérica"]
cambio = "alt+shift"
"#;

        let nuevo = cambiar_teclado_en_texto(original, &["españa".to_string()])
            .expect("debería poder cambiar la lista")
            .expect("debería existir un cambio");

        assert!(nuevo.contains("# Este comentario sigue."));
        assert!(nuevo.contains("cambio = \"alt+shift\""));

        let configuracion = leer_texto(&nuevo).expect("el resultado debería ser válido");
        assert_eq!(configuracion.teclado.distribuciones, vec!["españa"]);
    }

    #[test]
    fn cambiar_monitor_conserva_lo_demas() {
        let original = r#"[monitor]
resolucion = "1920x1080"
hz = 120

[bluetooth]
activo = true
"#;

        let nuevo = cambiar_monitor_en_texto(original, "2560x1440", 144)
            .expect("debería poder cambiar el monitor")
            .expect("debería existir un cambio");

        let configuracion = leer_texto(&nuevo).expect("el resultado debería ser válido");
        assert_eq!(configuracion.monitor.resolucion, "2560x1440");
        assert_eq!(configuracion.monitor.hz, 144);
        assert!(configuracion.bluetooth.activo);
    }

    #[test]
    fn cambiar_almacenamiento_conserva_el_resto() {
        let original = r#"# Este comentario sigue.
[almacenamiento]
disponibles = ["ST3500413AS · 500 GB"]

[bluetooth]
activo = true
"#;

        let nuevo = cambiar_almacenamiento_en_texto(original, &[])
            .expect("debería poder ocultar la unidad")
            .expect("debería existir un cambio");

        assert!(nuevo.contains("# Este comentario sigue."));

        let configuracion = leer_texto(&nuevo).expect("el resultado debería ser válido");
        assert!(configuracion.almacenamiento.disponibles.is_empty());
        assert!(configuracion.bluetooth.activo);
    }

    #[test]
    fn cambiar_apariencia_conserva_lo_demas() {
        let original = r#"# Este comentario tiene que quedarse.
[apariencia]
estilo = "dinamico"
modo = "automatico"

[sunshine]
activo = true
autoinicio = true
"#;

        let nuevo = cambiar_apariencia_en_texto(original, "everforest", "oscuro")
            .expect("debería poder cambiar la apariencia")
            .expect("debería existir un cambio");

        assert!(nuevo.contains("# Este comentario tiene que quedarse."));
        assert!(nuevo.contains("estilo = \"everforest\""));
        assert!(nuevo.contains("modo = \"oscuro\""));
        assert!(nuevo.contains("autoinicio = true"));
    }

    #[test]
    fn apagar_sunshine_no_borra_autoinicio_al_editar() {
        let original = r#"[sunshine]
activo = true
autoinicio = true
"#;

        let nuevo = cambiar_sunshine_en_texto(original, false, true)
            .expect("debería poder apagar Sunshine")
            .expect("debería existir un cambio");

        let configuracion = leer_texto(&nuevo).expect("el resultado debería ser válido");
        assert!(!configuracion.sunshine.activo);
        assert!(configuracion.sunshine.autoinicio);
    }

    #[test]
    fn apagar_steam_no_borra_sus_subopciones_al_editar() {
        let original = r#"[steam]
activo = true
remote_play = true
servidor_dedicado = true
"#;

        let nuevo = cambiar_steam_en_texto(original, false, true, true)
            .expect("debería poder apagar Steam")
            .expect("debería existir un cambio");

        let configuracion = leer_texto(&nuevo).expect("el resultado debería ser válido");
        assert!(!configuracion.steam.activo);
        assert!(configuracion.steam.remote_play);
        assert!(configuracion.steam.servidor_dedicado);
    }

    #[test]
    fn los_interruptores_no_borran_preferencias_ajenas() {
        let original = r#"[bluetooth]
activo = true

[impresion]
activa = true
controlador = "epson-201207w"

[virtualizacion]
activa = true
"#;

        let sin_bluetooth = cambiar_bluetooth_en_texto(original, false)
            .expect("debería poder apagar Bluetooth")
            .expect("debería existir un cambio");
        let sin_impresion = cambiar_impresion_en_texto(&sin_bluetooth, false)
            .expect("debería poder apagar impresión")
            .expect("debería existir un cambio");
        let nuevo = cambiar_virtualizacion_en_texto(&sin_impresion, false)
            .expect("debería poder apagar virtualización")
            .expect("debería existir un cambio");

        let configuracion = leer_texto(&nuevo).expect("el resultado debería ser válido");
        assert!(!configuracion.bluetooth.activo);
        assert!(!configuracion.impresion.activa);
        assert_eq!(
            configuracion.impresion.controlador.as_deref(),
            Some("epson-201207w")
        );
        assert!(!configuracion.virtualizacion.activa);
    }

    #[test]
    fn datos_personales_no_permiten_salir_de_su_carpeta() {
        let error = leer_texto(
            r#"
[[personas]]
cuenta = "koru"
nombre = "André"
avatar = "../otra-cosa.jpg"
"#,
        )
        .expect_err("el avatar no debería escapar de Korunix");

        assert!(error.contains("avatar"));
        assert!(error.contains(".."));
    }

    #[test]
    fn bluetooth_y_datos_personales_son_decisiones_humanas() {
        let configuracion = leer_texto(
            r#"
[[personas]]
cuenta = "koru"
nombre = "André"
avatar = "avatar-koru.jpg"
clave_github = ".ssh/blep"

[bluetooth]
activo = true
"#,
        )
        .expect("las decisiones deberían ser válidas");

        assert!(configuracion.bluetooth.activo);
        assert_eq!(
            configuracion.personas[0].avatar.as_deref(),
            Some("avatar-koru.jpg")
        );
        assert_eq!(
            configuracion.personas[0].clave_github.as_deref(),
            Some(".ssh/blep")
        );
    }
}

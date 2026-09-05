use crate::configuracion;
use serde::Deserialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnidadLocal {
    pub nombre: String,
    pub detalle: String,
    tecnica: Option<UnidadTecnica>,
    problema_adopcion: Option<String>,
    serial: Option<String>,
}

impl UnidadLocal {
    pub fn problema_adopcion(&self) -> Option<&str> {
        self.problema_adopcion.as_deref()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct UnidadTecnica {
    uuid: String,
    sistema_archivos: String,
    modelo: String,
    capacidad: String,
    transporte: String,
    ruta: String,
}

#[derive(Debug, Deserialize)]
struct SalidaLsblk {
    #[serde(default)]
    blockdevices: Vec<Dispositivo>,
}

#[derive(Debug, Deserialize)]
struct Dispositivo {
    #[serde(default)]
    size: Option<Value>,
    #[serde(rename = "type", default)]
    tipo: Option<String>,
    #[serde(default)]
    fstype: Option<String>,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    uuid: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    serial: Option<String>,
    #[serde(default)]
    tran: Option<String>,
    #[serde(default)]
    mountpoints: Option<Vec<Option<String>>>,
    #[serde(default)]
    children: Vec<Dispositivo>,
}

fn bytes(valor: &Option<Value>) -> Option<u64> {
    match valor.as_ref()? {
        Value::Number(numero) => numero.as_u64(),
        Value::String(texto) => texto.parse().ok(),
        _ => None,
    }
}

fn tiene_montaje(dispositivo: &Dispositivo, buscado: Option<&str>) -> bool {
    let aqui = dispositivo
        .mountpoints
        .as_deref()
        .unwrap_or_default()
        .iter()
        .flatten()
        .any(|montaje| match buscado {
            Some(buscado) => montaje == buscado,
            None => !montaje.is_empty(),
        });

    aqui || dispositivo
        .children
        .iter()
        .any(|hijo| tiene_montaje(hijo, buscado))
}

fn capacidad_humana(bytes: u64) -> String {
    let gb = (bytes.saturating_add(500_000_000)) / 1_000_000_000;
    format!("{gb} GB")
}

fn conexion_humana(valor: Option<&str>) -> Option<String> {
    let valor = valor?.trim();

    if valor.is_empty() {
        return None;
    }

    Some(match valor.to_ascii_lowercase().as_str() {
        "sata" | "ata" => "SATA".to_string(),
        "usb" => "USB".to_string(),
        "nvme" => "NVMe".to_string(),
        otro => otro.to_uppercase(),
    })
}

fn sistema_humano(valor: Option<&str>) -> Option<String> {
    let valor = valor?.trim();

    if valor.is_empty() {
        return None;
    }

    Some(match valor.to_ascii_lowercase().as_str() {
        "ntfs" | "ntfs3" => "NTFS".to_string(),
        "vfat" | "fat" | "fat32" => "FAT".to_string(),
        "exfat" => "exFAT".to_string(),
        otro => otro.to_string(),
    })
}

fn sistema_tecnico(valor: &str) -> Option<&'static str> {
    match valor.to_ascii_lowercase().as_str() {
        "ntfs" | "ntfs3" => Some("ntfs"),
        "exfat" => Some("exfat"),
        "vfat" | "fat" | "fat32" => Some("vfat"),
        "ext4" => Some("ext4"),
        "btrfs" => Some("btrfs"),
        "xfs" => Some("xfs"),
        _ => None,
    }
}

fn candidatas_adoptables<'a>(dispositivo: &'a Dispositivo, salida: &mut Vec<&'a Dispositivo>) {
    let sistema = dispositivo
        .fstype
        .as_deref()
        .and_then(sistema_tecnico)
        .is_some();
    let uuid = dispositivo
        .uuid
        .as_deref()
        .is_some_and(|valor| !valor.trim().is_empty());

    if sistema && uuid {
        salida.push(dispositivo);
    }

    for hijo in &dispositivo.children {
        candidatas_adoptables(hijo, salida);
    }
}

fn elegir_particion(dispositivo: &Dispositivo) -> Result<&Dispositivo, String> {
    let mut candidatas = Vec::new();
    candidatas_adoptables(dispositivo, &mut candidatas);

    if candidatas.is_empty() {
        return Err(
            "No encontré una partición con un formato compatible y una identidad estable."
                .to_string(),
        );
    }

    candidatas.sort_by_key(|candidata| std::cmp::Reverse(bytes(&candidata.size).unwrap_or(0)));

    if candidatas.len() > 1 {
        let primera = bytes(&candidatas[0].size).unwrap_or(0);
        let segunda = bytes(&candidatas[1].size).unwrap_or(0);
        let segunda_pequena = segunda <= 128 * 1024 * 1024;
        let primera_domina = primera > 0 && segunda.saturating_mul(10) < primera;

        if !segunda_pequena && !primera_domina {
            return Err(
                "La unidad tiene varias particiones de tamaño importante. No voy a adivinar cuál quieres administrar."
                    .to_string(),
            );
        }
    }

    Ok(candidatas[0])
}

fn identificador_ruta(uuid: &str) -> Result<String, String> {
    let identificador: String = uuid
        .chars()
        .filter(|caracter| caracter.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect();

    if identificador.len() < 4 {
        return Err("La partición no tiene una identidad estable que pueda conservar.".to_string());
    }

    Ok(format!("/mnt/korunix/{identificador}"))
}

fn preparar_tecnica(
    disco: &Dispositivo,
    modelo: &str,
    capacidad: &str,
) -> Result<UnidadTecnica, String> {
    let particion = elegir_particion(disco)?;
    let uuid = particion
        .uuid
        .as_deref()
        .map(str::trim)
        .filter(|valor| !valor.is_empty())
        .ok_or_else(|| "La partición no tiene una UUID estable.".to_string())?;
    let sistema_archivos = particion
        .fstype
        .as_deref()
        .and_then(sistema_tecnico)
        .ok_or_else(|| "El formato de la partición todavía no es compatible.".to_string())?;
    let transporte =
        conexion_humana(disco.tran.as_deref()).unwrap_or_else(|| "desconocido".to_string());

    Ok(UnidadTecnica {
        uuid: uuid.to_string(),
        sistema_archivos: sistema_archivos.to_string(),
        modelo: if modelo.is_empty() {
            "Disco".to_string()
        } else {
            modelo.to_string()
        },
        capacidad: capacidad.to_string(),
        transporte,
        ruta: identificador_ruta(uuid)?,
    })
}

fn leer_json(datos: &[u8]) -> Result<Vec<UnidadLocal>, String> {
    let salida: SalidaLsblk = serde_json::from_slice(datos).map_err(|error| {
        format!("No pude entender el estado local de los discos.\nDetalle: {error}")
    })?;

    let mut unidades = Vec::new();

    for disco in salida.blockdevices {
        if disco.tipo.as_deref() != Some("disk") || tiene_montaje(&disco, Some("/")) {
            continue;
        }

        let modelo = disco.model.as_deref().unwrap_or("").trim().to_string();
        let capacidad = bytes(&disco.size)
            .map(capacidad_humana)
            .unwrap_or_else(|| "capacidad desconocida".to_string());

        let nombre = if modelo.is_empty() {
            format!("Disco · {capacidad}")
        } else {
            format!("{modelo} · {capacidad}")
        };

        let particion_visible = {
            let mut candidatas = Vec::new();
            candidatas_adoptables(&disco, &mut candidatas);
            candidatas
                .into_iter()
                .max_by_key(|candidata| bytes(&candidata.size).unwrap_or(0))
        };

        let mut detalles = Vec::new();

        if let Some(conexion) = conexion_humana(disco.tran.as_deref()) {
            detalles.push(conexion);
        }

        if let Some(sistema) =
            sistema_humano(particion_visible.and_then(|parte| parte.fstype.as_deref()))
        {
            detalles.push(sistema);
        }

        if let Some(etiqueta) = particion_visible
            .and_then(|parte| parte.label.as_deref())
            .map(str::trim)
            .filter(|etiqueta| !etiqueta.is_empty())
        {
            detalles.push(format!("Etiqueta: {etiqueta}"));
        }

        let (tecnica, problema_adopcion) = match preparar_tecnica(&disco, &modelo, &capacidad) {
            Ok(tecnica) => (Some(tecnica), None),
            Err(error) => (None, Some(error)),
        };

        unidades.push(UnidadLocal {
            nombre,
            detalle: detalles.join(" · "),
            tecnica,
            problema_adopcion,
            serial: disco
                .serial
                .as_deref()
                .map(str::trim)
                .filter(|serial| !serial.is_empty())
                .map(str::to_string),
        });
    }

    let mut repeticiones = HashMap::new();

    for unidad in &unidades {
        *repeticiones.entry(unidad.nombre.clone()).or_insert(0usize) += 1;
    }

    for unidad in &mut unidades {
        if repeticiones.get(&unidad.nombre).copied().unwrap_or(0) <= 1 {
            continue;
        }

        let nombre_base = unidad.nombre.clone();

        if let Some(serial) = unidad.serial.as_deref() {
            let sufijo_invertido: String = serial.chars().rev().take(6).collect();
            let sufijo: String = sufijo_invertido.chars().rev().collect();
            unidad.nombre = format!("{nombre_base} · serie …{sufijo}");
        } else {
            unidad.tecnica = None;
            unidad.problema_adopcion = Some(
                "Hay otra unidad con el mismo modelo y capacidad y no tengo un dato estable para distinguirlas."
                    .to_string(),
            );
        }
    }

    let mut nombres_finales = HashSet::new();

    for unidad in &mut unidades {
        if !nombres_finales.insert(unidad.nombre.clone()) {
            unidad.tecnica = None;
            unidad.problema_adopcion = Some(
                "Hay otra unidad indistinguible con el mismo nombre. No voy a adivinar cuál es."
                    .to_string(),
            );
        }
    }

    unidades.sort_by(|izquierda, derecha| izquierda.nombre.cmp(&derecha.nombre));
    Ok(unidades)
}

pub fn leer() -> Result<Vec<UnidadLocal>, String> {
    let programa = env::var_os("KORUNIX_LSBLK_BIN").unwrap_or_else(|| {
        if Path::new("/run/current-system/sw/bin/lsblk").is_file() {
            "/run/current-system/sw/bin/lsblk".into()
        } else {
            "lsblk".into()
        }
    });

    let salida = Command::new(programa)
        .args([
            "--json",
            "--bytes",
            "--tree",
            "--output",
            "NAME,SIZE,TYPE,FSTYPE,LABEL,UUID,MODEL,SERIAL,TRAN,MOUNTPOINTS",
        ])
        .output()
        .map_err(|error| format!("No pude leer los discos locales.\nDetalle: {error}"))?;

    if !salida.status.success() {
        let detalle = String::from_utf8_lossy(&salida.stderr).trim().to_string();

        return Err(if detalle.is_empty() {
            "No pude leer los discos locales.".to_string()
        } else {
            format!("No pude leer los discos locales.\nDetalle: {detalle}")
        });
    }

    leer_json(&salida.stdout)
}

fn escapar_nix(valor: &str) -> String {
    valor
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace("${", "\\${")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn clave_unidad(nombre: &str) -> String {
    format!("    \"{}\" = {{", escapar_nix(nombre))
}

pub fn administrada(raiz: &Path, nombre: &str) -> Result<bool, String> {
    let texto = fs::read_to_string(raiz.join("hardware.nix"))
        .map_err(|error| format!("No pude leer hardware.nix.\nDetalle: {error}"))?;

    Ok(texto.lines().any(|linea| linea == clave_unidad(nombre)))
}

#[allow(dead_code)]
pub fn comprobar_elegidas(raiz: &Path, elegidas: &[String]) -> Result<(), String> {
    for nombre in elegidas {
        if !administrada(raiz, nombre)? {
            return Err(format!(
                "«{nombre}» está elegida en configuracion.toml, pero Korunix no tiene su identidad técnica.\nConecta la unidad y usa «korunix almacenamiento adoptar» desde la interfaz o la CLI."
            ));
        }
    }

    Ok(())
}

fn hardware_con_unidad(
    texto: &str,
    nombre: &str,
    tecnica: &UnidadTecnica,
    uid: u32,
    gid: u32,
) -> Result<String, String> {
    if texto.lines().any(|linea| linea == clave_unidad(nombre)) {
        return Ok(texto.to_string());
    }

    let inicio = texto
        .find("  _module.args.unidadesDetectadas = {\n")
        .ok_or_else(|| {
            "No encontré la lista de unidades detectadas en hardware.nix.".to_string()
        })?;
    let resto = &texto[inicio..];
    let cierre_relativo = resto.find("\n  };\n").ok_or_else(|| {
        "La lista de unidades detectadas de hardware.nix está incompleta.".to_string()
    })?;
    let cierre = inicio + cierre_relativo;

    let bloque = format!(
        "\n    \"{}\" = {{\n      uuid = \"{}\";\n      sistemaArchivos = \"{}\";\n      ruta = \"{}\";\n      modelo = \"{}\";\n      capacidad = \"{}\";\n      transporte = \"{}\";\n      uid = {};\n      gid = {};\n    }};\n",
        escapar_nix(nombre),
        escapar_nix(&tecnica.uuid),
        escapar_nix(&tecnica.sistema_archivos),
        escapar_nix(&tecnica.ruta),
        escapar_nix(&tecnica.modelo),
        escapar_nix(&tecnica.capacidad),
        escapar_nix(&tecnica.transporte),
        uid,
        gid,
    );

    let mut nuevo = String::with_capacity(texto.len() + bloque.len());
    nuevo.push_str(&texto[..cierre]);
    nuevo.push_str(&bloque);
    nuevo.push_str(&texto[cierre..]);
    Ok(nuevo)
}

fn temporal_para(ruta: &Path) -> PathBuf {
    let nombre = ruta
        .file_name()
        .and_then(|valor| valor.to_str())
        .unwrap_or("archivo");
    let momento = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    ruta.with_file_name(format!(".{nombre}.korunix-{}-{momento}.tmp", process::id()))
}

fn escribir_atomico(ruta: &Path, contenido: &[u8]) -> Result<(), String> {
    let temporal = temporal_para(ruta);

    let resultado = (|| {
        fs::write(&temporal, contenido)
            .map_err(|error| format!("No pude guardar {}.\nDetalle: {error}", ruta.display()))?;

        if let Ok(datos) = fs::metadata(ruta) {
            fs::set_permissions(&temporal, datos.permissions()).map_err(|error| {
                format!(
                    "No pude conservar los permisos de {}.\nDetalle: {error}",
                    ruta.display()
                )
            })?;
        }

        fs::rename(&temporal, ruta).map_err(|error| {
            format!(
                "No pude terminar de guardar {}.\nDetalle: {error}",
                ruta.display()
            )
        })
    })();

    if resultado.is_err() {
        let _ = fs::remove_file(temporal);
    }

    resultado
}

fn programa_nix() -> PathBuf {
    if let Some(programa) = env::var_os("KORUNIX_NIX_BIN") {
        return programa.into();
    }

    if Path::new("/run/current-system/sw/bin/nix").is_file() {
        return "/run/current-system/sw/bin/nix".into();
    }

    "nix".into()
}

fn comprobar_nix(raiz: &Path) -> Result<(), String> {
    let salida = Command::new(programa_nix())
        .args([
            "eval",
            "--raw",
            ".#nixosConfigurations.korunix.config.system.build.toplevel.drvPath",
        ])
        .current_dir(raiz)
        .output()
        .map_err(|error| format!("No pude validar la unidad con Nix.\nDetalle: {error}"))?;

    if salida.status.success() {
        return Ok(());
    }

    let detalle = String::from_utf8_lossy(&salida.stderr).trim().to_string();

    Err(if detalle.is_empty() {
        "Nix no aceptó la unidad detectada.".to_string()
    } else {
        format!("Nix no aceptó la unidad detectada.\nDetalle: {detalle}")
    })
}

pub fn adoptar(raiz: &Path, nombre: &str) -> Result<bool, String> {
    if administrada(raiz, nombre)? {
        return Ok(false);
    }

    let unidades = leer()?;
    let unidad = unidades
        .into_iter()
        .find(|unidad| unidad.nombre == nombre)
        .ok_or_else(|| {
            format!(
                "No encuentro conectada la unidad «{nombre}». No voy a guardar una identidad que no puedo comprobar."
            )
        })?;

    let tecnica = unidad.tecnica.ok_or_else(|| {
        unidad.problema_adopcion.unwrap_or_else(|| {
            "No pude obtener una identidad técnica segura para esta unidad.".to_string()
        })
    })?;

    let ruta_configuracion = raiz.join("configuracion.toml");
    let ruta_hardware = raiz.join("hardware.nix");
    let configuracion_antes = fs::read(&ruta_configuracion).map_err(|error| {
        format!(
            "No pude leer {}.\nDetalle: {error}",
            ruta_configuracion.display()
        )
    })?;
    let hardware_antes = fs::read(&ruta_hardware)
        .map_err(|error| format!("No pude leer hardware.nix.\nDetalle: {error}"))?;
    let hardware_texto = String::from_utf8(hardware_antes.clone())
        .map_err(|_| "hardware.nix no es texto UTF-8 válido.".to_string())?;

    let datos_repo = fs::metadata(raiz)
        .map_err(|error| format!("No pude leer los permisos del repositorio.\nDetalle: {error}"))?;
    let uid = datos_repo.uid();
    let gid = datos_repo.gid();

    let hardware_nuevo = hardware_con_unidad(&hardware_texto, nombre, &tecnica, uid, gid)?;
    escribir_atomico(&ruta_hardware, hardware_nuevo.as_bytes())?;

    let restaurar = || {
        let _ = escribir_atomico(&ruta_configuracion, &configuracion_antes);
        let _ = escribir_atomico(&ruta_hardware, &hardware_antes);
    };

    let actual = match configuracion::leer(&ruta_configuracion) {
        Ok(configuracion) => configuracion,
        Err(error) => {
            restaurar();
            return Err(error);
        }
    };

    let mut disponibles = actual.almacenamiento.disponibles;

    if !disponibles.iter().any(|unidad| unidad == nombre) {
        disponibles.push(nombre.to_string());
    }

    if let Err(error) = configuracion::cambiar_almacenamiento(&ruta_configuracion, &disponibles) {
        restaurar();
        return Err(error);
    }

    if let Err(error) = comprobar_nix(raiz) {
        restaurar();
        return Err(format!(
            "{error}\nRestauré configuracion.toml y hardware.nix; no quedó una adopción parcial."
        ));
    }

    Ok(true)
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn una_eleccion_sin_identidad_tecnica_se_rechaza() {
        let momento = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let raiz = env::temp_dir().join(format!(
            "korunix-almacenamiento-prueba-{}-{momento}",
            process::id()
        ));

        fs::create_dir_all(&raiz).expect("debería crear la carpeta temporal");
        fs::write(
            raiz.join("hardware.nix"),
            r#"
{
  _module.args.unidadesDetectadas = {
    "ST3500413AS · 500 GB" = {
      uuid = "036F8E656FF00FB2";
    };
  };
}
"#,
        )
        .expect("debería escribir hardware.nix temporal");

        let error = comprobar_elegidas(&raiz, &["Disco externo · 2 TB".to_string()])
            .expect_err("una elección sin identidad técnica debería rechazarse");

        assert!(error.contains("Disco externo · 2 TB"));
        assert!(error.contains("identidad técnica"));

        fs::remove_dir_all(&raiz).expect("debería limpiar la carpeta temporal");
    }

    #[test]
    fn omite_el_disco_del_sistema_y_muestra_el_secundario() {
        let datos = br#"{
          "blockdevices": [
            {
              "size": "1000000000000",
              "type": "disk",
              "model": "NVME-SISTEMA",
              "tran": "nvme",
              "mountpoints": [null],
              "children": [
                {
                  "size": "900000000000",
                  "type": "part",
                  "fstype": "btrfs",
                  "uuid": "SISTEMA",
                  "label": null,
                  "mountpoints": ["/"]
                }
              ]
            },
            {
              "size": 500107862016,
              "type": "disk",
              "model": "ST3500413AS ",
              "tran": "sata",
              "mountpoints": [null],
              "children": [
                {
                  "size": 500107859968,
                  "type": "part",
                  "fstype": "ntfs",
                  "uuid": "036F8E656FF00FB2",
                  "label": null,
                  "mountpoints": ["/mnt/datos"]
                }
              ]
            }
          ]
        }"#;

        let unidades = leer_json(datos).expect("debería entender lsblk");

        assert_eq!(unidades.len(), 1);
        assert_eq!(unidades[0].nombre, "ST3500413AS · 500 GB");
        assert_eq!(unidades[0].detalle, "SATA · NTFS");
        assert!(!unidades[0].detalle.contains("036F8E656FF00FB2"));
        assert!(!unidades[0].detalle.contains("/mnt/datos"));
    }

    #[test]
    fn ventoy_elige_la_particion_grande_sin_preguntar_uuid() {
        let datos = br#"{
          "blockdevices": [{
            "size": 15569256448,
            "type": "disk",
            "model": "DataTraveler 2.0",
            "tran": "usb",
            "mountpoints": [null],
            "children": [
              {
                "size": 15535702016,
                "type": "part",
                "fstype": "exfat",
                "uuid": "BAF1-579A",
                "label": "Ventoy",
                "mountpoints": [null]
              },
              {
                "size": 33554432,
                "type": "part",
                "fstype": "vfat",
                "uuid": "EA6C-95B2",
                "label": "VTOYEFI",
                "mountpoints": [null]
              }
            ]
          }]
        }"#;

        let unidades = leer_json(datos).expect("debería entender Ventoy");
        let tecnica = unidades[0]
            .tecnica
            .as_ref()
            .expect("debería elegir la partición de datos");

        assert_eq!(unidades[0].nombre, "DataTraveler 2.0 · 16 GB");
        assert!(unidades[0].detalle.contains("Etiqueta: Ventoy"));
        assert_eq!(tecnica.sistema_archivos, "exfat");
        assert_eq!(tecnica.uuid, "BAF1-579A");
        assert_eq!(tecnica.ruta, "/mnt/korunix/baf1579a");
        assert!(!unidades[0].detalle.contains("BAF1-579A"));
    }

    #[test]
    fn dos_discos_iguales_se_distinguen_solo_si_hace_falta() {
        let datos = br#"{
          "blockdevices": [
            {
              "size": 16000000000,
              "type": "disk",
              "model": "DataTraveler 2.0",
              "serial": "AAAA111111",
              "tran": "usb",
              "mountpoints": [null],
              "children": [{
                "size": 15900000000,
                "type": "part",
                "fstype": "exfat",
                "uuid": "AAAA-0001",
                "label": "UNO",
                "mountpoints": [null]
              }]
            },
            {
              "size": 16000000000,
              "type": "disk",
              "model": "DataTraveler 2.0",
              "serial": "BBBB222222",
              "tran": "usb",
              "mountpoints": [null],
              "children": [{
                "size": 15900000000,
                "type": "part",
                "fstype": "exfat",
                "uuid": "BBBB-0002",
                "label": "DOS",
                "mountpoints": [null]
              }]
            }
          ]
        }"#;

        let unidades = leer_json(datos).expect("debería distinguir las dos unidades");

        assert_eq!(unidades.len(), 2);
        assert!(unidades
            .iter()
            .any(|unidad| unidad.nombre == "DataTraveler 2.0 · 16 GB · serie …111111"));
        assert!(unidades
            .iter()
            .any(|unidad| unidad.nombre == "DataTraveler 2.0 · 16 GB · serie …222222"));
    }

    #[test]
    fn no_adivina_entre_dos_particiones_grandes() {
        let datos = br#"{
          "blockdevices": [{
            "size": 100000000000,
            "type": "disk",
            "model": "DOS-PARTICIONES",
            "tran": "usb",
            "mountpoints": [null],
            "children": [
              {
                "size": 60000000000,
                "type": "part",
                "fstype": "exfat",
                "uuid": "AAAA",
                "label": "UNO",
                "mountpoints": [null]
              },
              {
                "size": 40000000000,
                "type": "part",
                "fstype": "ext4",
                "uuid": "BBBB",
                "label": "DOS",
                "mountpoints": [null]
              }
            ]
          }]
        }"#;

        let unidades = leer_json(datos).expect("debería mostrar el disco");

        assert!(unidades[0].tecnica.is_none());
        assert!(unidades[0]
            .problema_adopcion
            .as_deref()
            .unwrap_or("")
            .contains("varias particiones"));
    }

    #[test]
    fn agrega_la_identidad_tecnica_sin_cambiar_el_nombre_humano() {
        let hardware = r#"{
  _module.args.unidadesDetectadas = {
    "ST3500413AS · 500 GB" = {
      uuid = "ANTERIOR";
    };
  };
}
"#;
        let tecnica = UnidadTecnica {
            uuid: "BAF1-579A".to_string(),
            sistema_archivos: "exfat".to_string(),
            modelo: "DataTraveler 2.0".to_string(),
            capacidad: "16 GB".to_string(),
            transporte: "USB".to_string(),
            ruta: "/mnt/korunix/baf1579a".to_string(),
        };

        let nuevo = hardware_con_unidad(hardware, "DataTraveler 2.0 · 16 GB", &tecnica, 1000, 100)
            .expect("debería guardar la identidad");

        assert!(nuevo.contains("\"DataTraveler 2.0 · 16 GB\" = {"));
        assert!(nuevo.contains("uuid = \"BAF1-579A\";"));
        assert!(nuevo.contains("ruta = \"/mnt/korunix/baf1579a\";"));
    }
}
